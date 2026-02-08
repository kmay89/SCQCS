// verify.rs — VBW bundle verification
//
// Reads a witness bundle from disk, recomputes all hashes, verifies
// the Ed25519 signature, and checks policy compliance.
//
// WHAT IS REAL:
//   - SHA-256 hash verification of all component files
//   - Ed25519 signature verification against the public key in the manifest
//   - Output artifact hash verification (if files still exist on disk)
//   - Policy compliance checks (dirty tree, lockfile presence, mode match)
//
// WHAT IS NOT YET IMPLEMENTED (TODOs):
//   - Co-signature (attest) verification — only builder.ed25519.sig is checked.
//     Additional signatures in signatures/ are written by `attest` but not
//     validated by `verify`. This is a TODO for VBW v1.1.
//   - Cross-referencing source_commit_tree_hash against the local git repo
//     (verify currently trusts the hash in the manifest, not recomputing it)
//   - Schema validation of JSON files against the published schemas

use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

use crate::hash;
use crate::sign;
use crate::vbw::model::*;

#[derive(Debug, PartialEq)]
pub enum Verdict {
    Verified,
    VerifiedWithVariance(Vec<String>),
    Unverified(Vec<String>),
}

/// Parsed component files loaded once during verification.
struct ComponentData {
    environment: Option<Environment>,
    materials_lock: Option<MaterialsLock>,
    outputs: Option<Outputs>,
    policy: Option<Policy>,
}

/// Verify a VBW witness bundle.
pub fn run_verify(bundle_dir: &Path) -> Result<Verdict> {
    let mut errors: Vec<String> = Vec::new();
    let mut warnings: Vec<String> = Vec::new();

    // 1. Load manifest
    let manifest_path = bundle_dir.join("manifest.json");
    let manifest_json = fs::read_to_string(&manifest_path)
        .with_context(|| format!("reading {}", manifest_path.display()))?;
    let manifest: Manifest =
        serde_json::from_str(&manifest_json).context("parsing manifest.json")?;

    eprintln!("[vbw] Verifying build: {}", manifest.build_id);
    eprintln!("[vbw] Project: {}", manifest.project.name);
    eprintln!("[vbw] Git commit: {}", manifest.git.commit);

    // 2. Verify manifest hash
    let stored_hash_path = bundle_dir.join("hashes/manifest.sha256");
    let stored_hash = fs::read_to_string(&stored_hash_path)
        .with_context(|| format!("reading {}", stored_hash_path.display()))?
        .trim()
        .to_string();
    let computed_hash = hash::sha256_hex(manifest_json.as_bytes());

    if stored_hash != computed_hash {
        errors.push(format!(
            "Manifest hash mismatch: stored={}, computed={}",
            stored_hash, computed_hash
        ));
    } else {
        eprintln!("[vbw] Manifest hash: OK");
    }

    // 3. Verify builder signature
    let sig_path = bundle_dir.join("signatures/builder.ed25519.sig");
    let signature = fs::read_to_string(&sig_path)
        .with_context(|| format!("reading {}", sig_path.display()))?
        .trim()
        .to_string();

    match sign::verify(
        &manifest.builder_identity.public_key_ed25519,
        manifest_json.as_bytes(),
        &signature,
    ) {
        Ok(true) => eprintln!("[vbw] Builder signature: OK"),
        Ok(false) => errors.push("Builder signature INVALID".to_string()),
        Err(e) => errors.push(format!("Signature verification error: {}", e)),
    }

    // 4. Load component files once, verify hashes, and parse
    let mut components = ComponentData {
        environment: None,
        materials_lock: None,
        outputs: None,
        policy: None,
    };

    verify_and_parse_component(
        bundle_dir,
        "environment.json",
        &manifest.environment_hash,
        &mut errors,
        |data| {
            components.environment = serde_json::from_str(data).ok();
        },
    );
    verify_and_parse_component(
        bundle_dir,
        "materials.lock.json",
        &manifest.materials_lock_hash,
        &mut errors,
        |data| {
            components.materials_lock = serde_json::from_str(data).ok();
        },
    );
    verify_and_parse_component(
        bundle_dir,
        "outputs.json",
        &manifest.outputs_hash,
        &mut errors,
        |data| {
            components.outputs = serde_json::from_str(data).ok();
        },
    );

    // 5. Verify policy ref
    let policy_in_bundle = bundle_dir.join("policy.json");
    if policy_in_bundle.exists() {
        let policy_data = fs::read_to_string(&policy_in_bundle)?;
        let policy_hash = hash::sha256_hex(policy_data.as_bytes());
        if policy_hash != manifest.policy_ref.hash_sha256 {
            errors.push(format!(
                "Policy hash mismatch: manifest={}, computed={}",
                manifest.policy_ref.hash_sha256, policy_hash
            ));
        } else {
            eprintln!("[vbw] Policy hash: OK");
        }
        components.policy = serde_json::from_str(&policy_data).ok();
    }

    // 6. Verify output artifacts exist and match
    if let Some(ref outputs) = components.outputs {
        for artifact in &outputs.artifacts {
            let artifact_path = PathBuf::from(&artifact.path);
            if artifact_path.exists() {
                match hash::hash_file(&artifact_path) {
                    Ok(h) if h == artifact.sha256 => {}
                    Ok(h) => errors.push(format!(
                        "Artifact {} hash mismatch: expected={}, actual={}",
                        artifact.path, artifact.sha256, h
                    )),
                    Err(e) => errors.push(format!(
                        "Failed to hash artifact {}: {}",
                        artifact.path, e
                    )),
                }
            } else {
                warnings.push(format!(
                    "Artifact {} not found (may have been deployed)",
                    artifact.path
                ));
            }
        }
        eprintln!(
            "[vbw] Output artifacts: {} checked",
            outputs.artifacts.len()
        );
    }

    // 7. Check policy compliance (using already-parsed data)
    if let Some(ref policy) = components.policy {
        check_policy_compliance(
            &manifest,
            policy,
            components.environment.as_ref(),
            components.materials_lock.as_ref(),
            &mut warnings,
        );
    }

    // 8. Emit verdict
    if !errors.is_empty() {
        eprintln!();
        eprintln!("❌ UNVERIFIED — {} error(s):", errors.len());
        for e in &errors {
            eprintln!("   • {}", e);
        }
        Ok(Verdict::Unverified(errors))
    } else if !warnings.is_empty() {
        eprintln!();
        eprintln!(
            "⚠️  VERIFIED WITH VARIANCE — {} warning(s):",
            warnings.len()
        );
        for w in &warnings {
            eprintln!("   • {}", w);
        }
        Ok(Verdict::VerifiedWithVariance(warnings))
    } else {
        eprintln!();
        eprintln!("✅ VERIFIED");
        Ok(Verdict::Verified)
    }
}

fn verify_and_parse_component<F>(
    bundle_dir: &Path,
    filename: &str,
    expected: &str,
    errors: &mut Vec<String>,
    parse_fn: F,
) where
    F: FnOnce(&str),
{
    let path = bundle_dir.join(filename);
    match fs::read_to_string(&path) {
        Ok(data) => {
            let computed = hash::sha256_hex(data.as_bytes());
            if computed != expected {
                errors.push(format!(
                    "{} hash mismatch: manifest={}, computed={}",
                    filename, expected, computed
                ));
            } else {
                eprintln!("[vbw] {}: OK", filename);
            }
            parse_fn(&data);
        }
        Err(e) => errors.push(format!("Cannot read {}: {}", filename, e)),
    }
}

fn check_policy_compliance(
    manifest: &Manifest,
    policy: &Policy,
    environment: Option<&Environment>,
    materials_lock: Option<&MaterialsLock>,
    warnings: &mut Vec<String>,
) {
    if manifest.git.dirty {
        warnings.push("Build from dirty git tree".to_string());
    }

    if let Some(env) = environment {
        if env.reproducibility.mode != policy.requirements.reproducibility.mode {
            warnings.push(format!(
                "Environment mode {:?} differs from policy {:?}",
                env.reproducibility.mode, policy.requirements.reproducibility.mode
            ));
        }
    }

    if policy.requirements.materials.require_lockfile_hashes {
        if let Some(mat) = materials_lock {
            if mat.lockfiles.is_empty() {
                warnings.push("Policy requires lockfile hashes but none found".to_string());
            }
        }
    }
}

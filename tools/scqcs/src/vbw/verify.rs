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

    // 4. Verify component file hashes
    verify_component_hash(
        bundle_dir,
        "environment.json",
        &manifest.environment_hash,
        &mut errors,
    );
    verify_component_hash(
        bundle_dir,
        "materials.lock.json",
        &manifest.materials_lock_hash,
        &mut errors,
    );
    verify_component_hash(
        bundle_dir,
        "outputs.json",
        &manifest.outputs_hash,
        &mut errors,
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
    }

    // 6. Verify output artifacts exist and match
    let outputs_path = bundle_dir.join("outputs.json");
    if outputs_path.exists() {
        let outputs_json = fs::read_to_string(&outputs_path)?;
        let outputs: Outputs = serde_json::from_str(&outputs_json).context("parsing outputs.json")?;

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

    // 7. Check policy compliance
    let policy_path = bundle_dir.join("policy.json");
    if policy_path.exists() {
        let policy_data = fs::read_to_string(&policy_path)?;
        let policy: Policy = serde_json::from_str(&policy_data)?;
        check_policy_compliance(&manifest, &policy, &mut warnings);
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

fn verify_component_hash(
    bundle_dir: &Path,
    filename: &str,
    expected: &str,
    errors: &mut Vec<String>,
) {
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
        }
        Err(e) => errors.push(format!("Cannot read {}: {}", filename, e)),
    }
}

fn check_policy_compliance(manifest: &Manifest, policy: &Policy, warnings: &mut Vec<String>) {
    // Check dirty-tree policy
    if manifest.git.dirty {
        warnings.push("Build from dirty git tree".to_string());
    }

    // Check reproducibility mode
    let env_path = PathBuf::from("vbw/environment.json");
    if env_path.exists() {
        if let Ok(env_data) = fs::read_to_string(&env_path) {
            if let Ok(env) = serde_json::from_str::<Environment>(&env_data) {
                if env.reproducibility.mode != policy.requirements.reproducibility.mode {
                    warnings.push(format!(
                        "Environment mode {:?} differs from policy {:?}",
                        env.reproducibility.mode, policy.requirements.reproducibility.mode
                    ));
                }
            }
        }
    }

    // Check lockfile requirement
    if policy.requirements.materials.require_lockfile_hashes {
        let mat_path = PathBuf::from("vbw/materials.lock.json");
        if mat_path.exists() {
            if let Ok(mat_data) = fs::read_to_string(&mat_path) {
                if let Ok(mat) = serde_json::from_str::<MaterialsLock>(&mat_data) {
                    if mat.lockfiles.is_empty() {
                        warnings.push(
                            "Policy requires lockfile hashes but none found".to_string(),
                        );
                    }
                }
            }
        }
    }
}

// verify.rs — VBW bundle verification (strict, fail-closed)
//
// Reads a witness bundle from disk, recomputes all hashes from canonical
// bytes, verifies the Ed25519 signature, validates bundle completeness,
// and checks policy compliance.
//
// FAIL-CLOSED DESIGN:
//   - Missing required files → UNVERIFIED
//   - Unexpected files in the bundle → UNVERIFIED (strict bundle policy)
//   - Hash mismatch on any component → UNVERIFIED
//   - Invalid or missing signature → UNVERIFIED
//   - Path traversal attempts (.. or absolute paths) → UNVERIFIED
//   - Symlinks that escape the bundle → UNVERIFIED
//   - All failure paths return non-zero exit code
//
// SIGNING: Signature and manifest hash are verified against canonical
// manifest bytes (sorted keys, compact JSON — see canonical.rs),
// NOT against the pretty-printed file on disk.
//
// WHAT IS NOT YET IMPLEMENTED (TODOs):
//   - Co-signature (attest) verification — only builder.ed25519.sig is checked
//   - Cross-referencing source_commit_tree_hash against the local git repo
//   - Schema validation of JSON files against the published schemas

use anyhow::{Context, Result};
use std::collections::{BTreeSet, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

use crate::hash;
use crate::sign;
use crate::vbw::canonical;
use crate::vbw::model::*;

/// Maximum directory traversal depth to prevent symlink cycle DoS.
const MAX_WALK_DEPTH: usize = 16;

#[derive(Debug, PartialEq)]
pub enum Verdict {
    Verified,
    VerifiedWithVariance(Vec<String>),
    Unverified(Vec<String>),
}

/// The set of files that MUST exist in a valid VBW bundle.
const REQUIRED_FILES: &[&str] = &[
    "manifest.json",
    "environment.json",
    "materials.lock.json",
    "outputs.json",
    "transcript.txt",
    "policy.json",
    "signatures/builder.ed25519.sig",
    "hashes/manifest.sha256",
];

/// Parsed component files loaded once during verification.
struct ComponentData {
    environment: Option<Environment>,
    materials_lock: Option<MaterialsLock>,
    outputs: Option<Outputs>,
    policy: Option<Policy>,
}

/// Verify a VBW witness bundle (strict, fail-closed).
pub fn run_verify(bundle_dir: &Path) -> Result<Verdict> {
    let mut errors: Vec<String> = Vec::new();
    let mut warnings: Vec<String> = Vec::new();

    // 0. Validate bundle directory exists and is a directory
    if !bundle_dir.exists() {
        return Ok(Verdict::Unverified(vec![format!(
            "Bundle directory does not exist: {}",
            bundle_dir.display()
        )]));
    }
    if !bundle_dir.is_dir() {
        return Ok(Verdict::Unverified(vec![format!(
            "Bundle path is not a directory: {}",
            bundle_dir.display()
        )]));
    }

    // 1. Check for path safety: bundle_dir must be a real path (resolve symlinks)
    let canonical_bundle = bundle_dir
        .canonicalize()
        .with_context(|| format!("resolving bundle path {}", bundle_dir.display()))?;

    // 2. Check all required files exist
    for required in REQUIRED_FILES {
        let path = canonical_bundle.join(required);
        if !path.exists() {
            errors.push(format!("Required file missing: {}", required));
        }
    }
    if !errors.is_empty() {
        return emit_verdict(errors, warnings);
    }

    // 3. Check for unexpected files (strict bundle policy)
    check_unexpected_files(&canonical_bundle, &mut errors)?;
    if !errors.is_empty() {
        return emit_verdict(errors, warnings);
    }

    // 4. Path safety: check for symlinks that escape the bundle
    check_symlink_safety(&canonical_bundle, &mut errors)?;
    if !errors.is_empty() {
        return emit_verdict(errors, warnings);
    }

    // 5. Load and parse manifest
    let manifest_path = canonical_bundle.join("manifest.json");
    let manifest_json = fs::read_to_string(&manifest_path)
        .with_context(|| format!("reading {}", manifest_path.display()))?;
    let manifest: Manifest =
        serde_json::from_str(&manifest_json).context("parsing manifest.json")?;

    eprintln!("[vbw] Verifying build: {}", manifest.build_id);
    eprintln!("[vbw] Project: {}", manifest.project.name);
    eprintln!("[vbw] Git commit: {}", manifest.git.commit);

    // 6. Recompute canonical manifest bytes from parsed manifest
    //    This is the critical step: we don't trust the bytes on disk,
    //    we re-canonicalize from the parsed struct.
    let canonical_bytes = canonical::canonical_manifest_bytes(&manifest);
    let computed_hash = hash::sha256_hex(&canonical_bytes);

    // 7. Verify manifest hash
    let stored_hash_path = canonical_bundle.join("hashes/manifest.sha256");
    let stored_hash = fs::read_to_string(&stored_hash_path)
        .with_context(|| format!("reading {}", stored_hash_path.display()))?
        .trim()
        .to_string();

    if stored_hash != computed_hash {
        errors.push(format!(
            "Manifest hash mismatch: stored={}, computed={} (from canonical bytes)",
            stored_hash, computed_hash
        ));
    } else {
        eprintln!("[vbw] Manifest hash (canonical): OK");
    }

    // 8. Verify builder signature against canonical manifest bytes
    let sig_path = canonical_bundle.join("signatures/builder.ed25519.sig");
    let signature = fs::read_to_string(&sig_path)
        .with_context(|| format!("reading {}", sig_path.display()))?
        .trim()
        .to_string();

    match sign::verify(
        &manifest.builder_identity.public_key_ed25519,
        &canonical_bytes,
        &signature,
    ) {
        Ok(true) => eprintln!("[vbw] Builder signature (over canonical bytes): OK"),
        Ok(false) => errors.push(
            "Builder signature INVALID (verified against canonical manifest bytes)".to_string(),
        ),
        Err(e) => errors.push(format!("Signature verification error: {}", e)),
    }

    // 9. Load and verify component files
    let mut components = ComponentData {
        environment: None,
        materials_lock: None,
        outputs: None,
        policy: None,
    };

    verify_and_parse_component(
        &canonical_bundle,
        "environment.json",
        &manifest.environment_hash,
        &mut errors,
        &mut warnings,
        |data| serde_json::from_str::<Environment>(data).map(|v| {
            components.environment = Some(v);
        }),
    );
    verify_and_parse_component(
        &canonical_bundle,
        "materials.lock.json",
        &manifest.materials_lock_hash,
        &mut errors,
        &mut warnings,
        |data| serde_json::from_str::<MaterialsLock>(data).map(|v| {
            components.materials_lock = Some(v);
        }),
    );
    verify_and_parse_component(
        &canonical_bundle,
        "outputs.json",
        &manifest.outputs_hash,
        &mut errors,
        &mut warnings,
        |data| serde_json::from_str::<Outputs>(data).map(|v| {
            components.outputs = Some(v);
        }),
    );

    // 10. Verify policy reference
    let policy_in_bundle = canonical_bundle.join("policy.json");
    let policy_data = fs::read_to_string(&policy_in_bundle).context("reading policy.json")?;
    let policy_hash = hash::sha256_hex(policy_data.as_bytes());
    if policy_hash != manifest.policy_ref.hash_sha256 {
        errors.push(format!(
            "Policy hash mismatch: manifest={}, computed={}",
            manifest.policy_ref.hash_sha256, policy_hash
        ));
    } else {
        eprintln!("[vbw] Policy hash: OK");
    }
    match serde_json::from_str::<Policy>(&policy_data) {
        Ok(p) => components.policy = Some(p),
        Err(e) => warnings.push(format!(
            "policy.json passed hash check but failed to parse: {} (policy compliance checks skipped)",
            e
        )),
    }

    // 11. Verify output artifacts exist and match
    if let Some(ref outputs) = components.outputs {
        for artifact in &outputs.artifacts {
            let artifact_path = PathBuf::from(&artifact.path);

            // Path safety: reject absolute paths
            if artifact_path.is_absolute() {
                errors.push(format!(
                    "Artifact path is absolute: {} (path traversal rejected)",
                    artifact.path
                ));
                continue;
            }

            // Path safety: reject any path component that is ".." (traversal)
            // Uses proper path component parsing instead of naive string search
            // to avoid false positives on filenames like "my..file.txt"
            if artifact_path
                .components()
                .any(|c| c == std::path::Component::ParentDir)
            {
                errors.push(format!(
                    "Artifact path contains parent directory traversal: {} (rejected)",
                    artifact.path
                ));
                continue;
            }

            if artifact_path.exists() {
                // Resolve symlinks and verify the real path doesn't escape
                // the project directory via symlink indirection
                if let Ok(real_path) = artifact_path.canonicalize() {
                    if let Ok(cwd) = std::env::current_dir() {
                        if !real_path.starts_with(&cwd) {
                            errors.push(format!(
                                "Artifact {} resolves outside project directory (symlink escape rejected)",
                                artifact.path
                            ));
                            continue;
                        }
                    }
                }

                match hash::hash_file(&artifact_path) {
                    Ok(h) if h == artifact.sha256 => {}
                    Ok(h) => errors.push(format!(
                        "Artifact {} hash mismatch: expected={}, actual={}",
                        artifact.path, artifact.sha256, h
                    )),
                    Err(e) => {
                        errors.push(format!("Failed to hash artifact {}: {}", artifact.path, e))
                    }
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

    // 12. Check enforcement consistency
    if let Some(ref enforcement) = manifest.enforcement {
        if let Some(ref policy) = components.policy {
            if enforcement.mode_requested != policy.requirements.reproducibility.mode {
                errors.push(format!(
                    "Enforcement mode_requested ({:?}) does not match policy mode ({:?})",
                    enforcement.mode_requested, policy.requirements.reproducibility.mode
                ));
            }
            if !enforcement.mode_enforced {
                warnings.push(format!(
                    "Mode {:?} was requested but NOT enforced at build time (mode_enforced=false)",
                    enforcement.mode_requested
                ));
            }
        }
    }

    // 13. Check policy compliance
    if let Some(ref policy) = components.policy {
        check_policy_compliance(
            &manifest,
            policy,
            components.environment.as_ref(),
            components.materials_lock.as_ref(),
            &mut warnings,
        );
    }

    emit_verdict(errors, warnings)
}

/// Enumerate all files in the bundle and reject unexpected ones.
///
/// This is the strict bundle policy: only known files are allowed.
/// Extra files indicate tampering or tooling bugs.
fn check_unexpected_files(bundle_dir: &Path, errors: &mut Vec<String>) -> Result<()> {
    let mut allowed: BTreeSet<PathBuf> = BTreeSet::new();
    for f in REQUIRED_FILES {
        allowed.insert(bundle_dir.join(f));
    }
    // Allow the signatures/ and hashes/ directories themselves
    allowed.insert(bundle_dir.join("signatures"));
    allowed.insert(bundle_dir.join("hashes"));

    // Walk the bundle directory
    let actual_files = walk_dir(bundle_dir)?;
    for path in &actual_files {
        if path.is_dir() {
            // Allow known subdirectories
            if *path == bundle_dir.join("signatures") || *path == bundle_dir.join("hashes") {
                continue;
            }
            errors.push(format!(
                "Unexpected directory in bundle: {}",
                path.strip_prefix(bundle_dir).unwrap_or(path).display()
            ));
        } else if !allowed.contains(path) {
            // Allow additional co-signature files in signatures/ (from attest command).
            // Strictly require the *.ed25519.sig naming pattern to prevent arbitrary
            // data from being smuggled into the bundle via a .sig extension.
            if path.starts_with(bundle_dir.join("signatures")) {
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    if name.ends_with(".ed25519.sig")
                        && name.len() > ".ed25519.sig".len()
                        && name.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-' || c == '.')
                    {
                        // Co-signatures are allowed but not verified in v1.0
                        continue;
                    }
                }
            }
            errors.push(format!(
                "Unexpected file in bundle: {}",
                path.strip_prefix(bundle_dir).unwrap_or(path).display()
            ));
        }
    }
    Ok(())
}

/// Check that no symlinks in the bundle escape the bundle directory.
fn check_symlink_safety(bundle_dir: &Path, errors: &mut Vec<String>) -> Result<()> {
    let entries = walk_dir(bundle_dir)?;
    for entry in &entries {
        // Check if entry is a symlink
        let metadata = entry.symlink_metadata()?;
        if metadata.file_type().is_symlink() {
            let target = fs::read_link(entry)?;
            let resolved = if target.is_absolute() {
                target.clone()
            } else {
                entry.parent().unwrap_or(bundle_dir).join(&target)
            };
            let resolved_canonical = resolved.canonicalize().unwrap_or_else(|_| resolved.clone());
            if !resolved_canonical.starts_with(bundle_dir) {
                errors.push(format!(
                    "Symlink escapes bundle: {} -> {} (resolves outside {})",
                    entry.display(),
                    target.display(),
                    bundle_dir.display()
                ));
            }
        }
    }
    Ok(())
}

/// Recursively walk a directory and return all entries (files and dirs).
///
/// Protects against symlink cycle DoS attacks by:
///   1. Limiting recursion depth to MAX_WALK_DEPTH
///   2. Tracking visited directories by canonical path to detect cycles
fn walk_dir(dir: &Path) -> Result<Vec<PathBuf>> {
    let mut visited = HashSet::new();
    walk_dir_inner(dir, &mut visited, 0)
}

fn walk_dir_inner(
    dir: &Path,
    visited: &mut HashSet<PathBuf>,
    depth: usize,
) -> Result<Vec<PathBuf>> {
    if depth > MAX_WALK_DEPTH {
        anyhow::bail!(
            "Directory traversal exceeded maximum depth ({}) at {} — possible symlink cycle",
            MAX_WALK_DEPTH,
            dir.display()
        );
    }

    // Track visited directories by canonical path to detect symlink cycles
    if let Ok(canonical) = dir.canonicalize() {
        if !visited.insert(canonical) {
            anyhow::bail!(
                "Directory cycle detected at {} (already visited via symlink)",
                dir.display()
            );
        }
    }

    let mut results = Vec::new();
    for entry in fs::read_dir(dir).with_context(|| format!("reading dir {}", dir.display()))? {
        let entry = entry?;
        let path = entry.path();
        results.push(path.clone());
        if path.is_dir() {
            results.extend(walk_dir_inner(&path, visited, depth + 1)?);
        }
    }
    Ok(results)
}

fn verify_and_parse_component<F>(
    bundle_dir: &Path,
    filename: &str,
    expected: &str,
    errors: &mut Vec<String>,
    warnings: &mut Vec<String>,
    parse_fn: F,
) where
    F: FnOnce(&str) -> Result<(), serde_json::Error>,
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
            if let Err(e) = parse_fn(&data) {
                warnings.push(format!(
                    "{} passed hash check but failed to parse: {} (related checks skipped)",
                    filename, e
                ));
            }
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

fn emit_verdict(errors: Vec<String>, warnings: Vec<String>) -> Result<Verdict> {
    if !errors.is_empty() {
        eprintln!();
        eprintln!("UNVERIFIED — {} error(s):", errors.len());
        for e in &errors {
            eprintln!("   - {}", e);
        }
        Ok(Verdict::Unverified(errors))
    } else if !warnings.is_empty() {
        eprintln!();
        eprintln!("VERIFIED WITH VARIANCE — {} warning(s):", warnings.len());
        for w in &warnings {
            eprintln!("   - {}", w);
        }
        Ok(Verdict::VerifiedWithVariance(warnings))
    } else {
        eprintln!();
        eprintln!("VERIFIED");
        Ok(Verdict::Verified)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sign;
    use crate::vbw::canonical;
    use std::fs;

    /// Helper: create a minimal valid bundle for testing.
    fn create_test_bundle(dir: &Path) -> Manifest {
        let (sk, pk) = sign::keygen();

        // Use Mode C policy to match the enforcement in the test manifest
        let policy = Policy {
            policy_version: "1.0".to_string(),
            requirements: PolicyRequirements {
                network: NetworkRequirement {
                    allowed: true,
                    allowlist: Some(vec![]),
                },
                reproducibility: ReproducibilityRequirement {
                    mode: ReproducibilityMode::C_WITNESSED_ND,
                    require_source_date_epoch: Some(false),
                },
                materials: MaterialsRequirement {
                    require_lockfile_hashes: false,
                    require_vendor_archive_and_tree: Some(false),
                },
                signing: Some(SigningRequirement {
                    require_maintainer_cosign_for_release: Some(false),
                }),
            },
        };
        let policy_json = serde_json::to_string_pretty(&policy).unwrap();
        let policy_hash = hash::sha256_hex(policy_json.as_bytes());

        let env = Environment {
            os: OsInfo {
                name: "TestOS".to_string(),
                version: None,
                kernel: None,
                arch: None,
            },
            container: None,
            tools: vec![ToolInfo {
                name: "test".to_string(),
                version: "1.0".to_string(),
                path: None,
                invocation: None,
            }],
            env: None,
            locale: None,
            timezone: None,
            reproducibility: Reproducibility {
                mode: ReproducibilityMode::C_WITNESSED_ND,
                source_date_epoch: None,
                network: None,
            },
        };
        let env_json = serde_json::to_string_pretty(&env).unwrap();
        let env_hash = hash::sha256_hex(env_json.as_bytes());

        let materials = MaterialsLock {
            lockfiles: vec![],
            materials: vec![],
        };
        let mat_json = serde_json::to_string_pretty(&materials).unwrap();
        let mat_hash = hash::sha256_hex(mat_json.as_bytes());

        let outputs = Outputs { artifacts: vec![] };
        let out_json = serde_json::to_string_pretty(&outputs).unwrap();
        let out_hash = hash::sha256_hex(out_json.as_bytes());

        let manifest = Manifest {
            vbw_version: "1.0".to_string(),
            build_id: "test-verify-bundle".to_string(),
            created_at: "2026-01-01T00:00:00Z".to_string(),
            project: Project {
                name: "test".to_string(),
                repo_url: None,
                homepage: None,
            },
            git: GitRef {
                commit: "aabbccddee".to_string(),
                branch: Some("main".to_string()),
                tag: None,
                dirty: false,
            },
            source_commit_tree_hash: "a".repeat(64),
            source_worktree_hash: None,
            materials_lock_hash: mat_hash,
            environment_hash: env_hash,
            outputs_hash: out_hash,
            builder_identity: BuilderIdentity {
                key_id: "test@verify".to_string(),
                public_key_ed25519: pk,
                issuer: None,
            },
            policy_ref: PolicyRef {
                path: "vbw/policy.json".to_string(),
                hash_sha256: policy_hash,
            },
            enforcement: Some(Enforcement {
                mode_requested: ReproducibilityMode::C_WITNESSED_ND,
                mode_enforced: true,
                network_blocked: false,
                source_date_epoch_set: false,
                notes: None,
            }),
            notes: None,
            ext: None,
        };

        // Compute canonical bytes for signing
        let canonical_bytes = canonical::canonical_manifest_bytes(&manifest);
        let manifest_hash = hash::sha256_hex(&canonical_bytes);
        let signature = sign::sign(&sk, &canonical_bytes).unwrap();

        // Write bundle files
        fs::create_dir_all(dir.join("signatures")).unwrap();
        fs::create_dir_all(dir.join("hashes")).unwrap();

        let pretty_manifest = serde_json::to_string_pretty(&manifest).unwrap();
        fs::write(dir.join("manifest.json"), &pretty_manifest).unwrap();
        fs::write(dir.join("environment.json"), &env_json).unwrap();
        fs::write(dir.join("materials.lock.json"), &mat_json).unwrap();
        fs::write(dir.join("outputs.json"), &out_json).unwrap();
        fs::write(dir.join("transcript.txt"), "test transcript\n").unwrap();
        fs::write(dir.join("policy.json"), &policy_json).unwrap();
        fs::write(dir.join("signatures/builder.ed25519.sig"), &signature).unwrap();
        fs::write(dir.join("hashes/manifest.sha256"), &manifest_hash).unwrap();

        manifest
    }

    #[test]
    fn verify_valid_bundle() {
        let dir = tempfile::tempdir().unwrap();
        let bundle = dir.path().join("vbw");
        fs::create_dir(&bundle).unwrap();
        create_test_bundle(&bundle);

        let verdict = run_verify(&bundle).unwrap();
        assert_eq!(verdict, Verdict::Verified);
    }

    #[test]
    fn verify_fails_on_modified_manifest() {
        let dir = tempfile::tempdir().unwrap();
        let bundle = dir.path().join("vbw");
        fs::create_dir(&bundle).unwrap();
        create_test_bundle(&bundle);

        // Tamper with manifest
        let mut manifest_json = fs::read_to_string(bundle.join("manifest.json")).unwrap();
        manifest_json = manifest_json.replace("test", "tampered");
        fs::write(bundle.join("manifest.json"), &manifest_json).unwrap();

        let verdict = run_verify(&bundle).unwrap();
        match verdict {
            Verdict::Unverified(errors) => {
                assert!(
                    errors
                        .iter()
                        .any(|e| e.contains("hash mismatch") || e.contains("INVALID")),
                    "Expected hash mismatch or invalid signature error, got: {:?}",
                    errors
                );
            }
            _ => panic!("Expected Unverified, got {:?}", verdict),
        }
    }

    #[test]
    fn verify_fails_on_invalid_signature() {
        let dir = tempfile::tempdir().unwrap();
        let bundle = dir.path().join("vbw");
        fs::create_dir(&bundle).unwrap();
        create_test_bundle(&bundle);

        // Replace signature with a different one (sign with different key)
        let (other_sk, _) = sign::keygen();
        let manifest_json = fs::read_to_string(bundle.join("manifest.json")).unwrap();
        let manifest: Manifest = serde_json::from_str(&manifest_json).unwrap();
        let canonical_bytes = canonical::canonical_manifest_bytes(&manifest);
        let bad_sig = sign::sign(&other_sk, &canonical_bytes).unwrap();
        fs::write(bundle.join("signatures/builder.ed25519.sig"), &bad_sig).unwrap();

        let verdict = run_verify(&bundle).unwrap();
        match verdict {
            Verdict::Unverified(errors) => {
                assert!(
                    errors.iter().any(|e| e.contains("INVALID")),
                    "Expected invalid signature error, got: {:?}",
                    errors
                );
            }
            _ => panic!("Expected Unverified, got {:?}", verdict),
        }
    }

    #[test]
    fn verify_fails_on_component_hash_change() {
        let dir = tempfile::tempdir().unwrap();
        let bundle = dir.path().join("vbw");
        fs::create_dir(&bundle).unwrap();
        create_test_bundle(&bundle);

        // Tamper with environment.json
        fs::write(
            bundle.join("environment.json"),
            r#"{"os":{"name":"Tampered"},"tools":[{"name":"x","version":"0"}],"reproducibility":{"mode":"C_WITNESSED_ND"}}"#,
        )
        .unwrap();

        let verdict = run_verify(&bundle).unwrap();
        match verdict {
            Verdict::Unverified(errors) => {
                assert!(
                    errors
                        .iter()
                        .any(|e| e.contains("environment.json") && e.contains("hash mismatch")),
                    "Expected environment hash mismatch, got: {:?}",
                    errors
                );
            }
            _ => panic!("Expected Unverified, got {:?}", verdict),
        }
    }

    #[test]
    fn verify_fails_on_extra_file() {
        let dir = tempfile::tempdir().unwrap();
        let bundle = dir.path().join("vbw");
        fs::create_dir(&bundle).unwrap();
        create_test_bundle(&bundle);

        // Add an unexpected file
        fs::write(bundle.join("malicious.txt"), "pwned").unwrap();

        let verdict = run_verify(&bundle).unwrap();
        match verdict {
            Verdict::Unverified(errors) => {
                assert!(
                    errors.iter().any(|e| e.contains("Unexpected file")),
                    "Expected unexpected file error, got: {:?}",
                    errors
                );
            }
            _ => panic!("Expected Unverified, got {:?}", verdict),
        }
    }

    #[test]
    fn verify_fails_on_missing_file() {
        let dir = tempfile::tempdir().unwrap();
        let bundle = dir.path().join("vbw");
        fs::create_dir(&bundle).unwrap();
        create_test_bundle(&bundle);

        // Remove a required file
        fs::remove_file(bundle.join("transcript.txt")).unwrap();

        let verdict = run_verify(&bundle).unwrap();
        match verdict {
            Verdict::Unverified(errors) => {
                assert!(
                    errors.iter().any(|e| e.contains("Required file missing")),
                    "Expected missing file error, got: {:?}",
                    errors
                );
            }
            _ => panic!("Expected Unverified, got {:?}", verdict),
        }
    }

    #[test]
    fn verify_fails_on_symlink_escape() {
        let dir = tempfile::tempdir().unwrap();
        let bundle = dir.path().join("vbw");
        fs::create_dir(&bundle).unwrap();
        create_test_bundle(&bundle);

        // Create a symlink that points outside the bundle
        let symlink_path = bundle.join("escape_link");
        // Use /etc/passwd as target (exists on all Unix)
        #[cfg(unix)]
        {
            std::os::unix::fs::symlink("/etc/passwd", &symlink_path).unwrap();
        }

        let verdict = run_verify(&bundle).unwrap();
        match verdict {
            Verdict::Unverified(errors) => {
                assert!(
                    errors
                        .iter()
                        .any(|e| e.contains("Unexpected file") || e.contains("Symlink escapes")),
                    "Expected symlink or unexpected file error, got: {:?}",
                    errors
                );
            }
            _ => {
                #[cfg(unix)]
                panic!("Expected Unverified, got {:?}", verdict);
            }
        }
    }

    #[test]
    fn verify_allows_cosignature_files() {
        let dir = tempfile::tempdir().unwrap();
        let bundle = dir.path().join("vbw");
        fs::create_dir(&bundle).unwrap();
        create_test_bundle(&bundle);

        // Add a co-signature file (should be allowed)
        fs::write(
            bundle.join("signatures/maintainer_org.ed25519.sig"),
            "base64sigdata",
        )
        .unwrap();

        let verdict = run_verify(&bundle).unwrap();
        // Should still verify (co-sigs are allowed but not checked)
        assert!(
            matches!(
                verdict,
                Verdict::Verified | Verdict::VerifiedWithVariance(_)
            ),
            "Co-signature files should be allowed, got {:?}",
            verdict
        );
    }

    #[test]
    fn verify_nonexistent_bundle_dir() {
        let verdict = run_verify(Path::new("/nonexistent/path/vbw")).unwrap();
        match verdict {
            Verdict::Unverified(errors) => {
                assert!(errors.iter().any(|e| e.contains("does not exist")));
            }
            _ => panic!("Expected Unverified for nonexistent dir"),
        }
    }
}

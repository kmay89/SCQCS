// build.rs — VBW build workflow
//
// Orchestrates the build pipeline: load policy, capture environment,
// detect lockfiles, gather git state, run the build command, hash outputs,
// assemble the manifest, sign it, and write the bundle to vbw/.
//
// SIGNING: The Ed25519 signature covers canonical manifest bytes (sorted
// keys, compact JSON — see canonical.rs), NOT the pretty-printed file on
// disk. The stored hash in hashes/manifest.sha256 is also over canonical bytes.
//
// WHAT IS REAL:
//   - Cryptographic hashing (SHA-256) of all files and artifacts
//   - Ed25519 signing of canonical manifest bytes
//   - Git commit/branch/dirty detection
//   - Source tree hashing via git ls-tree
//   - Lockfile detection and hashing
//   - Environment capture (OS, tools, container detection)
//   - Build command execution with interleaved transcript capture
//   - Enforcement honesty: manifest records what was actually enforced
//
// WHAT IS NOT YET IMPLEMENTED (TODOs):
//   - Build-time policy enforcement (Mode A network blocking, etc.)
//   - Vendor tarball hashing (archive_sha256 + extracted_tree_hash)
//   - Individual dependency hash verification from lockfiles

use anyhow::{Context, Result};
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::thread;

use crate::hash;
use crate::sign;
use crate::vbw::canonical;
use crate::vbw::model::*;

/// Lockfile names to auto-detect in the project root.
const LOCKFILE_NAMES: &[&str] = &[
    "package-lock.json",
    "yarn.lock",
    "pnpm-lock.yaml",
    "Cargo.lock",
    "go.sum",
    "Gemfile.lock",
    "poetry.lock",
    "composer.lock",
    "Pipfile.lock",
];

/// Run the full VBW build workflow.
pub fn run_build(
    build_cmd: &[String],
    project_name: Option<&str>,
    output_dir: Option<&str>,
    keyfile: Option<&Path>,
    key_id: Option<&str>,
    policy_path: Option<&str>,
) -> Result<()> {
    let vbw_dir = PathBuf::from("vbw");
    let dist_dir = PathBuf::from(output_dir.unwrap_or("dist"));

    // 1. Load or auto-generate policy
    let policy_file = policy_path
        .map(PathBuf::from)
        .unwrap_or_else(|| vbw_dir.join("policy.json"));
    let policy = load_or_create_policy(&policy_file)?;
    let policy_json = serde_json::to_string_pretty(&policy)?;
    let policy_hash = hash::sha256_hex(policy_json.as_bytes());

    // 2. Check mode enforcement honesty and warn loudly
    let enforcement = compute_enforcement(&policy);
    if !enforcement.mode_enforced {
        eprintln!(
            "[vbw] WARNING: Requested mode {:?} but enforcement is NOT implemented.",
            enforcement.mode_requested
        );
        eprintln!("[vbw] WARNING: The manifest will record mode_enforced=false.");
        if let Some(ref notes) = enforcement.notes {
            eprintln!("[vbw] WARNING: {}", notes);
        }
    }

    // 3. Load signing key
    let secret_key = sign::load_secret_key(keyfile)?;
    let public_key = sign::public_key_from_secret(&secret_key)?;
    let resolved_key_id = key_id.unwrap_or("builder@local").to_string();

    // 4. Capture environment
    let environment = capture_environment(&policy)?;
    let env_json = serde_json::to_string_pretty(&environment)?;
    let env_hash = hash::sha256_hex(env_json.as_bytes());

    // 5. Detect and hash lockfiles → materials_lock
    let materials_lock = detect_materials()?;
    let mat_json = serde_json::to_string_pretty(&materials_lock)?;
    let mat_hash = hash::sha256_hex(mat_json.as_bytes());

    // 6. Git info
    let git_info = crate::git::get_git_info().context("getting git info")?;

    // 7. Source commit tree hash
    let source_commit_tree_hash =
        crate::git::source_commit_tree_hash(&git_info.commit).context("source tree hash")?;

    // 8. Source worktree hash (if dirty)
    let source_worktree_hash = if git_info.dirty {
        Some(crate::git::source_worktree_hash().context("worktree hash")?)
    } else {
        None
    };

    // 9. Run build command, capture interleaved transcript
    eprintln!("[vbw] Running build: {}", build_cmd.join(" "));
    let transcript = run_build_command(build_cmd)?;

    // 10. Collect outputs from dist/
    let outputs = collect_outputs(&dist_dir)?;
    let out_json = serde_json::to_string_pretty(&outputs)?;
    let out_hash = hash::sha256_hex(out_json.as_bytes());

    // 11. Determine project name
    let proj_name = project_name
        .map(|s| s.to_string())
        .or_else(|| {
            std::env::current_dir()
                .ok()
                .and_then(|p| p.file_name().map(|n| n.to_string_lossy().to_string()))
        })
        .unwrap_or_else(|| "unknown".to_string());

    // 12. Build manifest
    let build_id = uuid::Uuid::new_v4().to_string();
    let created_at = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);

    let manifest = Manifest {
        vbw_version: "1.0".to_string(),
        build_id,
        created_at,
        project: Project {
            name: proj_name,
            repo_url: None,
            homepage: None,
        },
        git: GitRef {
            commit: git_info.commit,
            branch: git_info.branch,
            tag: git_info.tag,
            dirty: git_info.dirty,
        },
        source_commit_tree_hash,
        source_worktree_hash,
        materials_lock_hash: mat_hash,
        environment_hash: env_hash,
        outputs_hash: out_hash,
        builder_identity: BuilderIdentity {
            key_id: resolved_key_id,
            public_key_ed25519: public_key,
            issuer: None,
        },
        policy_ref: PolicyRef {
            path: policy_file.to_string_lossy().to_string(),
            hash_sha256: policy_hash,
        },
        enforcement: Some(enforcement),
        notes: None,
        ext: None,
    };

    // 13. Compute canonical bytes, sign, and hash
    //     The signature covers canonical manifest bytes (sorted keys, compact).
    //     The pretty-printed file on disk is for human readability only.
    let canonical_bytes = canonical::canonical_manifest_bytes(&manifest);
    let manifest_hash = hash::sha256_hex(&canonical_bytes);
    let signature = sign::sign(&secret_key, &canonical_bytes)?;

    // Write pretty manifest for human readability
    let manifest_json_pretty = serde_json::to_string_pretty(&manifest)?;

    // 14. Write all files
    fs::create_dir_all(vbw_dir.join("signatures"))?;
    fs::create_dir_all(vbw_dir.join("hashes"))?;

    fs::write(vbw_dir.join("manifest.json"), &manifest_json_pretty)?;
    fs::write(vbw_dir.join("environment.json"), &env_json)?;
    fs::write(vbw_dir.join("materials.lock.json"), &mat_json)?;
    fs::write(vbw_dir.join("outputs.json"), &out_json)?;
    fs::write(vbw_dir.join("transcript.txt"), &transcript)?;
    fs::write(vbw_dir.join("policy.json"), &policy_json)?;
    fs::write(vbw_dir.join("signatures/builder.ed25519.sig"), &signature)?;
    fs::write(vbw_dir.join("hashes/manifest.sha256"), &manifest_hash)?;

    eprintln!("[vbw] Witness bundle written to vbw/");
    eprintln!("[vbw] Signature covers canonical manifest bytes (sorted keys, compact JSON).");
    eprintln!("[vbw] Build ID: {}", manifest.build_id);
    eprintln!("[vbw] Manifest hash (canonical): {}", manifest_hash);
    eprintln!("[vbw] Artifacts: {} file(s)", outputs.artifacts.len());

    Ok(())
}

/// Compute enforcement flags based on what VBW v1.0 can actually enforce.
fn compute_enforcement(policy: &Policy) -> Enforcement {
    let mode = &policy.requirements.reproducibility.mode;
    let sde_set = std::env::var("SOURCE_DATE_EPOCH").is_ok();

    match mode {
        ReproducibilityMode::A_DETERMINISTIC => Enforcement {
            mode_requested: mode.clone(),
            mode_enforced: false,
            network_blocked: false,
            source_date_epoch_set: sde_set,
            notes: Some(
                "VBW v1.0: Mode A requested but network isolation, container pinning, \
                 and SOURCE_DATE_EPOCH enforcement are not implemented. \
                 The mode is a declaration only."
                    .to_string(),
            ),
        },
        ReproducibilityMode::B_LOCKED_NETWORK => Enforcement {
            mode_requested: mode.clone(),
            mode_enforced: false,
            network_blocked: false,
            source_date_epoch_set: sde_set,
            notes: Some(
                "VBW v1.0: Mode B requested but dependency-source verification \
                 is not implemented. Lockfile hashes are recorded but the tool \
                 does not verify that the build only fetched from those lockfiles."
                    .to_string(),
            ),
        },
        ReproducibilityMode::C_WITNESSED_ND => Enforcement {
            mode_requested: mode.clone(),
            // Mode C is honestly enforceable: it makes no reproducibility promises.
            mode_enforced: true,
            network_blocked: false,
            source_date_epoch_set: sde_set,
            notes: None,
        },
    }
}

fn load_or_create_policy(path: &Path) -> Result<Policy> {
    if path.exists() {
        let data = fs::read_to_string(path)
            .with_context(|| format!("reading policy {}", path.display()))?;
        let policy: Policy = serde_json::from_str(&data).with_context(|| "parsing policy.json")?;
        Ok(policy)
    } else {
        eprintln!("[vbw] No policy found, generating default (Mode B)");
        let policy = Policy::default_policy();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(&policy)?;
        fs::write(path, &json)?;
        Ok(policy)
    }
}

/// Capture the current build environment.
///
/// NOTE: This implementation targets Unix/Linux and CI runners (GitHub Actions,
/// Docker). OS detection uses `uname` and tool paths use `which`. On non-Unix
/// systems the OS fields will fall back to "unknown".
fn capture_environment(policy: &Policy) -> Result<Environment> {
    let os_name = get_cmd_output("uname", &["-s"]).unwrap_or_else(|_| "unknown".into());
    let os_version = get_cmd_output("uname", &["-r"]).ok();
    let kernel = get_cmd_output("uname", &["-v"]).ok();
    let arch = get_cmd_output("uname", &["-m"]).ok();

    let mut tools: Vec<ToolInfo> = Vec::new();

    // Detect common tools
    for (name, flag) in &[
        ("node", "--version"),
        ("npm", "--version"),
        ("cargo", "--version"),
        ("rustc", "--version"),
        ("gcc", "--version"),
        ("python3", "--version"),
        ("go", "version"),
    ] {
        if let Ok(version) = get_cmd_output(name, &[flag]) {
            let version_line = version.lines().next().unwrap_or(&version).to_string();
            tools.push(ToolInfo {
                name: name.to_string(),
                version: version_line,
                path: which_cmd(name).ok(),
                invocation: None,
            });
        }
    }

    // At least one tool required
    if tools.is_empty() {
        tools.push(ToolInfo {
            name: "sh".to_string(),
            version: get_cmd_output("sh", &["--version"]).unwrap_or_else(|_| "unknown".into()),
            path: Some("/bin/sh".to_string()),
            invocation: None,
        });
    }

    let container = detect_container();
    let mode = policy.requirements.reproducibility.mode.clone();
    let network_allowed = policy.requirements.network.allowed;
    let allowlist = policy.requirements.network.allowlist.clone();

    Ok(Environment {
        os: OsInfo {
            name: os_name,
            version: os_version,
            kernel,
            arch,
        },
        container,
        tools,
        env: None,
        locale: std::env::var("LANG").ok(),
        timezone: std::env::var("TZ").ok(),
        reproducibility: Reproducibility {
            mode,
            source_date_epoch: std::env::var("SOURCE_DATE_EPOCH")
                .ok()
                .and_then(|v| v.parse().ok()),
            network: Some(NetworkPolicy {
                allowed: network_allowed,
                allowlist,
            }),
        },
    })
}

/// Detect if we're running inside a container or CI environment.
///
/// This is heuristic-based, not authoritative. Returns None for bare-metal.
/// GitHub Actions is reported with type "none" because it's a VM, not a container,
/// but we include it so the environment record reflects CI context.
fn detect_container() -> Option<ContainerInfo> {
    // Check for /.dockerenv (standard Docker marker file)
    if Path::new("/.dockerenv").exists() {
        return Some(ContainerInfo {
            container_type: "docker".to_string(),
            image: std::env::var("CONTAINER_IMAGE").ok(),
            image_digest: std::env::var("CONTAINER_IMAGE_DIGEST")
                .unwrap_or_else(|_| "unknown".to_string()),
        });
    }

    // Check /proc/self/cgroup for container indicators
    if let Ok(cgroup) = fs::read_to_string("/proc/self/cgroup") {
        if cgroup.contains("docker") || cgroup.contains("containerd") {
            return Some(ContainerInfo {
                container_type: "docker".to_string(),
                image: None,
                image_digest: "unknown".to_string(),
            });
        }
    }

    // Check for GitHub Actions container
    if std::env::var("GITHUB_ACTIONS").is_ok() {
        return Some(ContainerInfo {
            container_type: "none".to_string(),
            image: Some("github-actions-runner".to_string()),
            image_digest: std::env::var("ImageOS").unwrap_or_else(|_| "unknown".to_string()),
        });
    }

    None
}

fn detect_materials() -> Result<MaterialsLock> {
    let mut lockfiles = Vec::new();
    let mut materials = Vec::new();

    for name in LOCKFILE_NAMES {
        let path = Path::new(name);
        if path.exists() {
            let file_hash = hash::hash_file(path)?;
            lockfiles.push(LockfileEntry {
                path: name.to_string(),
                sha256: file_hash.clone(),
            });
            materials.push(MaterialEntry {
                name: name.to_string(),
                kind: lockfile_kind(name).to_string(),
                source: None,
                sha256: file_hash,
                archive_sha256: None,
                extracted_tree_hash: None,
            });
        }
    }

    Ok(MaterialsLock {
        lockfiles,
        materials,
    })
}

/// Map lockfile name to a material kind for the schema.
///
/// The schema allows: "npm", "git", "tarball", "file".
/// We use "npm" for JS ecosystem locks and "file" for everything else.
fn lockfile_kind(name: &str) -> &str {
    match name {
        "package-lock.json" | "yarn.lock" | "pnpm-lock.yaml" => "npm",
        "Cargo.lock" => "file",
        "go.sum" => "file",
        "Gemfile.lock" => "file",
        "poetry.lock" | "Pipfile.lock" => "file",
        "composer.lock" => "file",
        _ => "file",
    }
}

/// Run the user's build command, capturing interleaved stdout and stderr
/// with timestamps for forensic value.
///
/// Each line is tagged with a stream identifier and ISO-8601 timestamp:
///   [2026-01-01T00:00:00.123Z] [stdout] line contents here
///   [2026-01-01T00:00:00.456Z] [stderr] warning: something
///
/// Lines from both streams are collected via a channel and written in
/// arrival order, which approximates true interleaving.
fn run_build_command(cmd: &[String]) -> Result<String> {
    if cmd.is_empty() {
        anyhow::bail!("No build command specified");
    }

    let mut child = Command::new(&cmd[0])
        .args(&cmd[1..])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .with_context(|| format!("spawning build command: {}", cmd[0]))?;

    let (tx, rx) = mpsc::channel::<String>();

    // Spawn a reader thread for stdout
    let stdout = child.stdout.take().expect("stdout was piped");
    let tx_out = tx.clone();
    let stdout_thread = thread::spawn(move || {
        let reader = BufReader::new(stdout);
        for line in reader.lines().map_while(Result::ok) {
            let ts = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true);
            let tagged = format!("[{}] [stdout] {}", ts, line);
            eprint!("{}\r\n", line);
            let _ = tx_out.send(tagged);
        }
    });

    // Spawn a reader thread for stderr
    let stderr = child.stderr.take().expect("stderr was piped");
    let tx_err = tx;
    let stderr_thread = thread::spawn(move || {
        let reader = BufReader::new(stderr);
        for line in reader.lines().map_while(Result::ok) {
            let ts = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true);
            let tagged = format!("[{}] [stderr] {}", ts, line);
            eprint!("{}\r\n", line);
            let _ = tx_err.send(tagged);
        }
    });

    // Collect lines in arrival order (approximately interleaved)
    // We must drop the transmitters by joining threads before collecting.
    stdout_thread.join().expect("stdout reader thread panicked");
    stderr_thread.join().expect("stderr reader thread panicked");

    let mut transcript = String::new();
    for line in rx.iter() {
        transcript.push_str(&line);
        transcript.push('\n');
    }

    let status = child.wait().context("waiting for build command")?;
    if !status.success() {
        anyhow::bail!(
            "Build command failed with exit code: {}",
            status.code().unwrap_or(-1)
        );
    }

    Ok(transcript)
}

fn collect_outputs(dist_dir: &Path) -> Result<Outputs> {
    let mut artifacts = Vec::new();

    if dist_dir.exists() {
        collect_artifacts(dist_dir, dist_dir, &mut artifacts)?;
    } else {
        eprintln!(
            "[vbw] Warning: output directory {} does not exist",
            dist_dir.display()
        );
    }

    Ok(Outputs { artifacts })
}

fn collect_artifacts(root: &Path, dir: &Path, out: &mut Vec<Artifact>) -> Result<()> {
    let mut entries: Vec<_> = fs::read_dir(dir)
        .with_context(|| format!("reading {}", dir.display()))?
        .filter_map(|e| e.ok())
        .collect();
    entries.sort_by_key(|e| e.file_name());

    for entry in entries {
        let path = entry.path();
        if path.is_dir() {
            collect_artifacts(root, &path, out)?;
        } else {
            let rel = path
                .strip_prefix(root)
                .unwrap()
                .to_string_lossy()
                .replace('\\', "/");
            let meta = fs::metadata(&path)?;
            let file_hash = hash::hash_file(&path)?;

            out.push(Artifact {
                path: format!("{}/{}", root.display(), rel),
                sha256: file_hash,
                size_bytes: meta.len(),
                mime: guess_mime(&rel),
                build_id: None,
                notes: None,
            });
        }
    }
    Ok(())
}

fn guess_mime(path: &str) -> Option<String> {
    let ext = path.rsplit('.').next()?;
    Some(
        match ext {
            "html" | "htm" => "text/html",
            "css" => "text/css",
            "js" | "mjs" => "application/javascript",
            "json" => "application/json",
            "xml" => "application/xml",
            "svg" => "image/svg+xml",
            "png" => "image/png",
            "jpg" | "jpeg" => "image/jpeg",
            "gif" => "image/gif",
            "webp" => "image/webp",
            "woff" => "font/woff",
            "woff2" => "font/woff2",
            "ttf" => "font/ttf",
            "pdf" => "application/pdf",
            "wasm" => "application/wasm",
            "map" => "application/json",
            "txt" => "text/plain",
            "ico" => "image/x-icon",
            _ => return None,
        }
        .to_string(),
    )
}

fn get_cmd_output(cmd: &str, args: &[&str]) -> Result<String> {
    let output = Command::new(cmd)
        .args(args)
        .output()
        .with_context(|| format!("running {} {}", cmd, args.join(" ")))?;
    if !output.status.success() {
        anyhow::bail!("{} failed", cmd);
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn which_cmd(name: &str) -> Result<String> {
    get_cmd_output("which", &[name])
}

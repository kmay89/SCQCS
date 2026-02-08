// main.rs — SCQCS CLI entry point
//
// Currently the only top-level command is `vbw` (Verified Build Witness).
// The CLI is structured to allow future commands under the `scqcs` namespace.

mod cli;
mod git;
mod hash;
mod sign;
mod vbw;

use anyhow::{Context, Result};
use clap::Parser;
use std::fs;
use std::path::{Path, PathBuf};

use cli::{Cli, Commands, VbwAction};

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Vbw { action } => match action {
            VbwAction::Keygen { output } => cmd_keygen(output),
            VbwAction::Build {
                project,
                output_dir,
                keyfile,
                key_id,
                policy,
                cmd,
            } => vbw::build::run_build(
                &cmd,
                project.as_deref(),
                Some(&output_dir),
                keyfile.as_deref(),
                key_id.as_deref(),
                policy.as_deref(),
            ),
            VbwAction::Verify { bundle } => {
                let verdict = vbw::verify::run_verify(&bundle)?;
                match verdict {
                    vbw::verify::Verdict::Verified => std::process::exit(0),
                    vbw::verify::Verdict::VerifiedWithVariance(_) => std::process::exit(0),
                    vbw::verify::Verdict::Unverified(_) => std::process::exit(1),
                }
            }
            VbwAction::Attest {
                bundle,
                keyfile,
                key_id,
            } => cmd_attest(&bundle, keyfile.as_deref(), key_id.as_deref()),
        },
    }
}

fn cmd_keygen(output: Option<PathBuf>) -> Result<()> {
    let (sk, pk) = sign::keygen();
    let dir = output.unwrap_or_else(|| PathBuf::from("."));
    fs::create_dir_all(&dir)?;

    let sk_path = dir.join("vbw-builder.sk");
    let pk_path = dir.join("vbw-builder.pk");

    fs::write(&sk_path, &sk)?;
    // Restrict secret key file permissions to owner-only (0600) on Unix.
    // Prevents other users on the system from reading the signing key.
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&sk_path, fs::Permissions::from_mode(0o600))?;
    }
    fs::write(&pk_path, &pk)?;

    eprintln!("Ed25519 keypair generated:");
    eprintln!("  Secret key: {}", sk_path.display());
    eprintln!("  Public key: {}", pk_path.display());
    eprintln!();
    eprintln!("Public key (base64): {}", pk);
    eprintln!();
    eprintln!("SECURITY: Copy the secret key value to a secure location (e.g. CI secret),");
    eprintln!("then verify the .sk file permissions are restricted.");
    eprintln!("  SCQCS_VBW_ED25519_SK_B64=<contents of {}>", sk_path.display());

    Ok(())
}

fn cmd_attest(
    bundle: &Path,
    keyfile: Option<&std::path::Path>,
    key_id: Option<&str>,
) -> Result<()> {
    let secret_key = sign::load_secret_key(keyfile)?;
    let public_key = sign::public_key_from_secret(&secret_key)?;
    let resolved_key_id = key_id.unwrap_or("maintainer@local");

    // Read manifest, parse, and sign canonical bytes (consistent with build + verify)
    let manifest_path = bundle.join("manifest.json");
    let manifest_json = fs::read_to_string(&manifest_path)?;
    let manifest: vbw::model::Manifest =
        serde_json::from_str(&manifest_json).context("parsing manifest.json")?;
    let canonical_bytes = vbw::canonical::canonical_manifest_bytes(&manifest);
    let signature = sign::sign(&secret_key, &canonical_bytes)?;

    // Write co-signature
    let sig_dir = bundle.join("signatures");
    fs::create_dir_all(&sig_dir)?;

    // Sanitize key_id for use as a filename: whitelist alphanumeric, hyphen,
    // underscore, and dot. Replace all other characters (including path separators,
    // shell metacharacters, and control characters) with underscore.
    let sanitized_id: String = resolved_key_id
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.' {
                c
            } else {
                '_'
            }
        })
        .collect();

    // Reject empty or dot-only filenames that could cause filesystem issues
    let sanitized_id = if sanitized_id.is_empty()
        || sanitized_id == "."
        || sanitized_id == ".."
        || sanitized_id.starts_with('.')
    {
        format!("key_{}", sanitized_id)
    } else {
        sanitized_id
    };

    let sig_filename = format!("{}.ed25519.sig", sanitized_id);
    let sig_path = sig_dir.join(&sig_filename);
    fs::write(&sig_path, &signature)?;

    eprintln!("[vbw] Attestation added:");
    eprintln!("  Key ID: {}", resolved_key_id);
    eprintln!("  Public key: {}", public_key);
    eprintln!(
        "  Signature (over canonical manifest bytes): {}",
        sig_path.display()
    );

    Ok(())
}

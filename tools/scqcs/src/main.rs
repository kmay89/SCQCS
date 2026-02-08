// main.rs — SCQCS CLI entry point
//
// Currently the only top-level command is `vbw` (Verified Build Witness).
// The CLI is structured to allow future commands under the `scqcs` namespace.

mod cli;
mod git;
mod hash;
mod sign;
mod vbw;

use anyhow::Result;
use clap::Parser;
use std::fs;
use std::path::PathBuf;

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
    fs::write(&pk_path, &pk)?;

    eprintln!("Ed25519 keypair generated:");
    eprintln!("  Secret key: {}", sk_path.display());
    eprintln!("  Public key: {}", pk_path.display());
    eprintln!();
    eprintln!("Public key (base64): {}", pk);
    eprintln!();
    eprintln!("To use in CI, set the secret key as:");
    eprintln!("  SCQCS_VBW_ED25519_SK_B64={}", sk);

    Ok(())
}

fn cmd_attest(
    bundle: &PathBuf,
    keyfile: Option<&std::path::Path>,
    key_id: Option<&str>,
) -> Result<()> {
    let secret_key = sign::load_secret_key(keyfile)?;
    let public_key = sign::public_key_from_secret(&secret_key)?;
    let resolved_key_id = key_id.unwrap_or("maintainer@local");

    // Read and sign the manifest
    let manifest_path = bundle.join("manifest.json");
    let manifest_json = fs::read_to_string(&manifest_path)?;
    let signature = sign::sign(&secret_key, manifest_json.as_bytes())?;

    // Write co-signature
    let sig_dir = bundle.join("signatures");
    fs::create_dir_all(&sig_dir)?;

    let sig_filename = format!(
        "{}.ed25519.sig",
        resolved_key_id.replace(['@', '/', '\\'], "_")
    );
    let sig_path = sig_dir.join(&sig_filename);
    fs::write(&sig_path, &signature)?;

    eprintln!("[vbw] Attestation added:");
    eprintln!("  Key ID: {}", resolved_key_id);
    eprintln!("  Public key: {}", public_key);
    eprintln!("  Signature: {}", sig_path.display());

    Ok(())
}

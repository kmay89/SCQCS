// cli.rs — Command-line interface definitions (clap derive)
//
// Defines the top-level `scqcs` command and the `vbw` subcommand tree:
//   scqcs vbw keygen   — generate Ed25519 keypair
//   scqcs vbw build    — run build + generate witness bundle
//   scqcs vbw verify   — verify a witness bundle
//   scqcs vbw attest   — add a co-signature to an existing bundle

use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "scqcs")]
#[command(about = "SCQCS CLI — Verified Build Witness tooling")]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Verified Build Witness commands
    Vbw {
        #[command(subcommand)]
        action: VbwAction,
    },
}

#[derive(Subcommand)]
pub enum VbwAction {
    /// Generate an Ed25519 keypair for build signing
    Keygen {
        /// Output directory for key files (default: current directory)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Run a build command and generate a witness bundle
    Build {
        /// Project name (default: current directory name)
        #[arg(long)]
        project: Option<String>,

        /// Output directory for build artifacts (default: dist/)
        #[arg(long, default_value = "dist")]
        output_dir: String,

        /// Path to Ed25519 secret key file
        #[arg(long)]
        keyfile: Option<PathBuf>,

        /// Key identifier string
        #[arg(long)]
        key_id: Option<String>,

        /// Path to policy.json (default: vbw/policy.json)
        #[arg(long)]
        policy: Option<String>,

        /// Build command (everything after --)
        #[arg(last = true, required = true)]
        cmd: Vec<String>,
    },

    /// Verify a witness bundle
    Verify {
        /// Path to the VBW bundle directory
        #[arg(long, default_value = "vbw")]
        bundle: PathBuf,
    },

    /// Add a maintainer co-signature to a bundle
    Attest {
        /// Path to the VBW bundle directory
        #[arg(long, default_value = "vbw")]
        bundle: PathBuf,

        /// Path to Ed25519 secret key file
        #[arg(long)]
        keyfile: Option<PathBuf>,

        /// Key identifier for the attestor
        #[arg(long)]
        key_id: Option<String>,
    },
}

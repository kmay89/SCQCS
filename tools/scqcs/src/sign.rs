// sign.rs — Ed25519 key generation, signing, and verification
//
// Uses the `ed25519-dalek` crate (v2) for all cryptographic operations.
// Keys are 32-byte seeds, signatures are 64 bytes, all encoded as
// standard base64 for storage and transport.
//
// REAL: This is real Ed25519 cryptography using OS-provided randomness.
// Keys generated here are production-grade.

use anyhow::{bail, Context, Result};
use base64::engine::general_purpose::STANDARD as B64;
use base64::Engine;
use ed25519_dalek::{Signer, SigningKey, Verifier, VerifyingKey};
use std::path::Path;
use zeroize::Zeroize;

/// Generate a new Ed25519 keypair using OS randomness.
/// Returns (secret_key_base64, public_key_base64).
pub fn keygen() -> (String, String) {
    let mut csprng = rand::rngs::OsRng;
    let signing_key = SigningKey::generate(&mut csprng);
    let verifying_key = signing_key.verifying_key();
    (
        B64.encode(signing_key.to_bytes()),
        B64.encode(verifying_key.to_bytes()),
    )
}

/// Sign arbitrary data with an Ed25519 secret key.
/// The secret key is a base64-encoded 32-byte seed.
/// Returns the signature as base64.
///
/// Intermediate secret key bytes are zeroized after use to limit
/// the window during which key material exists in memory.
pub fn sign(secret_key_b64: &str, data: &[u8]) -> Result<String> {
    let mut sk_bytes = B64
        .decode(secret_key_b64)
        .context("decoding secret key base64")?;

    if sk_bytes.len() != 32 {
        let len = sk_bytes.len();
        sk_bytes.zeroize();
        anyhow::bail!("secret key must be 32 bytes, but was {} bytes", len);
    }

    let mut sk_array = [0u8; 32];
    sk_array.copy_from_slice(&sk_bytes);
    sk_bytes.zeroize();

    let signing_key = SigningKey::from_bytes(&sk_array);
    let sig = signing_key.sign(data);
    sk_array.zeroize();

    Ok(B64.encode(sig.to_bytes()))
}

/// Verify an Ed25519 signature.
/// Returns Ok(true) if valid, Ok(false) if the signature doesn't match.
/// Returns Err only if the key or signature bytes are malformed.
pub fn verify(public_key_b64: &str, data: &[u8], signature_b64: &str) -> Result<bool> {
    let pk_bytes = B64
        .decode(public_key_b64)
        .context("decoding public key base64")?;
    let pk_array: [u8; 32] = pk_bytes
        .try_into()
        .map_err(|_| anyhow::anyhow!("public key must be 32 bytes"))?;
    let verifying_key =
        VerifyingKey::from_bytes(&pk_array).context("invalid Ed25519 public key")?;

    let sig_bytes = B64
        .decode(signature_b64)
        .context("decoding signature base64")?;
    let sig_array: [u8; 64] = sig_bytes
        .try_into()
        .map_err(|_| anyhow::anyhow!("signature must be 64 bytes"))?;
    let signature = ed25519_dalek::Signature::from_bytes(&sig_array);

    Ok(verifying_key.verify(data, &signature).is_ok())
}

/// Load the builder's secret key from one of two sources (checked in order):
///   1. SCQCS_VBW_ED25519_SK_B64 environment variable (preferred for CI)
///   2. --keyfile path on disk (for local development)
///
/// Returns the base64-encoded secret key string.
pub fn load_secret_key(keyfile: Option<&Path>) -> Result<String> {
    if let Ok(key) = std::env::var("SCQCS_VBW_ED25519_SK_B64") {
        if !key.is_empty() {
            return Ok(key);
        }
    }

    if let Some(path) = keyfile {
        let contents = std::fs::read_to_string(path)
            .with_context(|| format!("reading keyfile {}", path.display()))?;
        return Ok(contents.trim().to_string());
    }

    bail!(
        "No signing key found. Set SCQCS_VBW_ED25519_SK_B64 env var \
         or pass --keyfile <path>"
    );
}

/// Derive the public key from a secret key.
/// Both are base64-encoded.
///
/// Intermediate secret key bytes are zeroized after use.
pub fn public_key_from_secret(secret_key_b64: &str) -> Result<String> {
    let mut sk_bytes = B64
        .decode(secret_key_b64)
        .context("decoding secret key base64")?;

    if sk_bytes.len() != 32 {
        let len = sk_bytes.len();
        sk_bytes.zeroize();
        anyhow::bail!("secret key must be 32 bytes, but was {} bytes", len);
    }

    let mut sk_array = [0u8; 32];
    sk_array.copy_from_slice(&sk_bytes);
    sk_bytes.zeroize();

    let signing_key = SigningKey::from_bytes(&sk_array);
    let verifying_key = signing_key.verifying_key();
    sk_array.zeroize();

    Ok(B64.encode(verifying_key.to_bytes()))
}

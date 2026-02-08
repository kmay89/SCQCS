use anyhow::{bail, Context, Result};
use base64::engine::general_purpose::STANDARD as B64;
use base64::Engine;
use ed25519_dalek::{Signer, SigningKey, Verifier, VerifyingKey};
use std::path::Path;

/// Generate a new Ed25519 keypair. Returns (secret_key_b64, public_key_b64).
pub fn keygen() -> (String, String) {
    let mut csprng = rand::rngs::OsRng;
    let signing_key = SigningKey::generate(&mut csprng);
    let verifying_key = signing_key.verifying_key();
    (
        B64.encode(signing_key.to_bytes()),
        B64.encode(verifying_key.to_bytes()),
    )
}

/// Sign data with an Ed25519 secret key (base64-encoded 32-byte seed).
pub fn sign(secret_key_b64: &str, data: &[u8]) -> Result<String> {
    let sk_bytes = B64
        .decode(secret_key_b64)
        .context("decoding secret key base64")?;
    let sk_array: [u8; 32] = sk_bytes
        .try_into()
        .map_err(|_| anyhow::anyhow!("secret key must be 32 bytes"))?;
    let signing_key = SigningKey::from_bytes(&sk_array);
    let sig = signing_key.sign(data);
    Ok(B64.encode(sig.to_bytes()))
}

/// Verify an Ed25519 signature (base64) against a public key (base64).
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

/// Load a secret key from either the environment variable or a keyfile path.
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

/// Derive the public key (base64) from a secret key (base64).
pub fn public_key_from_secret(secret_key_b64: &str) -> Result<String> {
    let sk_bytes = B64
        .decode(secret_key_b64)
        .context("decoding secret key base64")?;
    let sk_array: [u8; 32] = sk_bytes
        .try_into()
        .map_err(|_| anyhow::anyhow!("secret key must be 32 bytes"))?;
    let signing_key = SigningKey::from_bytes(&sk_array);
    let verifying_key = signing_key.verifying_key();
    Ok(B64.encode(verifying_key.to_bytes()))
}

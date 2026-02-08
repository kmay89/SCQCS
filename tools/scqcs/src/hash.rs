// hash.rs — SHA-256 hashing utilities
//
// This module provides the core hashing primitives used throughout VBW.
// All hashes are SHA-256, output as lowercase hex strings (64 characters).
//
// These are real cryptographic hashes using the RustCrypto `sha2` crate,
// not placeholders or demos.

use anyhow::{Context, Result};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::Path;

/// Compute the SHA-256 digest of a byte slice and return it as a 64-char hex string.
pub fn sha256_hex(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hex_encode(&hasher.finalize())
}

/// Read a file from disk and return its SHA-256 hex digest.
///
/// Reads the entire file into memory. For very large files (multi-GB),
/// a streaming approach would be better, but for build artifacts this is fine.
pub fn hash_file(path: &Path) -> Result<String> {
    let data = fs::read(path).with_context(|| format!("reading {}", path.display()))?;
    Ok(sha256_hex(&data))
}

/// Convert raw bytes to a lowercase hex string.
/// Used by both this module and git.rs.
pub(crate) fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

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
use std::io::{BufReader, Read};
use std::path::Path;

/// Compute the SHA-256 digest of a byte slice and return it as a 64-char hex string.
pub fn sha256_hex(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hex_encode(&hasher.finalize())
}

/// Read a file from disk using streaming I/O and return its SHA-256 hex digest.
///
/// Uses a buffered reader with 64 KiB chunks to avoid loading the entire file
/// into memory. Safe for files of any size.
pub fn hash_file(path: &Path) -> Result<String> {
    let file = fs::File::open(path).with_context(|| format!("opening {}", path.display()))?;
    let mut reader = BufReader::with_capacity(64 * 1024, file);
    let mut hasher = Sha256::new();
    let mut buf = [0u8; 64 * 1024];
    loop {
        let n = reader
            .read(&mut buf)
            .with_context(|| format!("reading {}", path.display()))?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    Ok(hex_encode(&hasher.finalize()))
}

/// Convert raw bytes to a lowercase hex string.
/// Used by both this module and git.rs.
pub(crate) fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn sha256_known_vector() {
        // SHA-256("") = e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855
        assert_eq!(
            sha256_hex(b""),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }

    #[test]
    fn sha256_hello() {
        // SHA-256("hello") = 2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824
        assert_eq!(
            sha256_hex(b"hello"),
            "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824"
        );
    }

    #[test]
    fn hash_file_streaming_matches_in_memory() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("testfile.bin");

        // Write a multi-MB file (3 MiB of repeating pattern)
        let pattern: Vec<u8> = (0..256u16).map(|i| (i % 256) as u8).collect();
        {
            let mut f = std::fs::File::create(&path).unwrap();
            for _ in 0..(3 * 1024 * 1024 / pattern.len()) {
                f.write_all(&pattern).unwrap();
            }
        }

        // Hash via streaming
        let streaming_hash = hash_file(&path).unwrap();

        // Hash via loading all into memory
        let data = std::fs::read(&path).unwrap();
        let memory_hash = sha256_hex(&data);

        assert_eq!(
            streaming_hash, memory_hash,
            "Streaming and in-memory hashing must produce identical results"
        );
    }

    #[test]
    fn hash_file_empty() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("empty.bin");
        std::fs::write(&path, b"").unwrap();

        assert_eq!(
            hash_file(&path).unwrap(),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }
}

use anyhow::{Context, Result};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::Path;

/// Compute SHA-256 hex digest of a byte slice.
pub fn sha256_hex(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hex_encode(&hasher.finalize())
}

/// Compute SHA-256 hex digest of a file.
pub fn hash_file(path: &Path) -> Result<String> {
    let data = fs::read(path).with_context(|| format!("reading {}", path.display()))?;
    Ok(sha256_hex(&data))
}

/// Compute a canonical tree hash for a directory.
///
/// Walks all files (sorted lexicographically by relative path), hashing each
/// file's contents, then hashes the concatenation of `"<relative_path>\0<sha256>\n"`.
#[allow(dead_code)]
pub fn hash_tree_canonical(root: &Path) -> Result<String> {
    let mut entries: Vec<(String, String)> = Vec::new();
    collect_files(root, root, &mut entries)?;
    entries.sort_by(|a, b| a.0.cmp(&b.0));

    let mut hasher = Sha256::new();
    for (rel_path, file_hash) in &entries {
        hasher.update(rel_path.as_bytes());
        hasher.update(b"\0");
        hasher.update(file_hash.as_bytes());
        hasher.update(b"\n");
    }
    Ok(hex_encode(&hasher.finalize()))
}

#[allow(dead_code)]
fn collect_files(root: &Path, dir: &Path, out: &mut Vec<(String, String)>) -> Result<()> {
    for entry in fs::read_dir(dir).with_context(|| format!("reading dir {}", dir.display()))? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            // Skip .git directories
            if path.file_name().map_or(false, |n| n == ".git") {
                continue;
            }
            collect_files(root, &path, out)?;
        } else {
            let rel = path
                .strip_prefix(root)
                .unwrap()
                .to_string_lossy()
                .replace('\\', "/");
            let h = hash_file(&path)?;
            out.push((rel, h));
        }
    }
    Ok(())
}

fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

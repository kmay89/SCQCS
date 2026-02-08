use anyhow::{bail, Context, Result};
use std::process::Command;

/// Information about the current git state.
pub struct GitInfo {
    pub commit: String,
    pub branch: Option<String>,
    pub tag: Option<String>,
    pub dirty: bool,
}

/// Gather git commit, branch, tag, and dirty status.
pub fn get_git_info() -> Result<GitInfo> {
    let commit = run_git(&["rev-parse", "HEAD"])
        .context("getting git commit")?
        .trim()
        .to_string();

    let branch = run_git(&["rev-parse", "--abbrev-ref", "HEAD"])
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| s != "HEAD");

    let tag = run_git(&["describe", "--tags", "--exact-match", "HEAD"])
        .ok()
        .map(|s| s.trim().to_string());

    let status = run_git(&["status", "--porcelain"]).context("checking dirty status")?;
    let dirty = !status.trim().is_empty();

    Ok(GitInfo {
        commit,
        branch,
        tag,
        dirty,
    })
}

/// Compute a SHA-256 hash of the canonical source tree at a given commit.
///
/// Uses `git ls-tree -r <commit>` to enumerate all tracked blobs, then hashes
/// the sorted list of `"<mode> <type> <object_hash>\t<path>\n"` entries.
pub fn source_commit_tree_hash(commit: &str) -> Result<String> {
    let output = run_git(&["ls-tree", "-r", commit]).context("git ls-tree")?;
    let hash = crate::hash::sha256_hex(output.as_bytes());
    Ok(hash)
}

/// Compute a SHA-256 hash of the worktree (including unstaged changes).
///
/// Uses `git ls-files -z` to get all tracked files, sorts them, and hashes
/// each file's content from the working tree.
pub fn source_worktree_hash() -> Result<String> {
    let output = run_git(&["ls-files", "-z"]).context("git ls-files")?;
    let mut files: Vec<&str> = output.split('\0').filter(|s| !s.is_empty()).collect();
    files.sort();

    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    for file in &files {
        let path = std::path::Path::new(file);
        if path.exists() {
            let contents = std::fs::read(path)
                .with_context(|| format!("reading worktree file {}", file))?;
            let file_hash = crate::hash::sha256_hex(&contents);
            hasher.update(file.as_bytes());
            hasher.update(b"\0");
            hasher.update(file_hash.as_bytes());
            hasher.update(b"\n");
        }
    }
    let result = hasher.finalize();
    Ok(crate::hash::hex_encode(&result))
}

fn run_git(args: &[&str]) -> Result<String> {
    let output = Command::new("git")
        .args(args)
        .output()
        .context("spawning git")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("git {} failed: {}", args.join(" "), stderr.trim());
    }
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

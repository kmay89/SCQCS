// git.rs — Git state detection and source tree hashing
//
// Shells out to the `git` CLI to gather commit info and compute
// canonical source tree hashes. Requires `git` on PATH.
//
// REAL: All git operations produce real data from the actual repository.
// No mocking or simulation.

use anyhow::{bail, Context, Result};
use std::process::Command;

/// Snapshot of the current git state at build time.
pub struct GitInfo {
    /// Full 40-char SHA-1 commit hash.
    pub commit: String,
    /// Current branch name, or None if in detached HEAD state.
    pub branch: Option<String>,
    /// Exact tag on this commit, or None if untagged.
    pub tag: Option<String>,
    /// True if there are uncommitted changes (staged or unstaged).
    pub dirty: bool,
}

/// Gather git commit, branch, tag, and dirty status from the working directory.
pub fn get_git_info() -> Result<GitInfo> {
    let commit = run_git(&["rev-parse", "HEAD"])
        .context("getting git commit")?
        .trim()
        .to_string();

    let branch = run_git(&["rev-parse", "--abbrev-ref", "HEAD"])
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| s != "HEAD"); // Detached HEAD returns literal "HEAD"

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

/// Compute a SHA-256 hash of the committed source tree.
///
/// Runs `git ls-tree -r <commit>` which outputs one line per tracked file:
///   `<mode> <type> <object_hash>\t<path>`
///
/// The output is already sorted by git. We hash the entire text block.
/// This means two commits with identical tracked files produce identical hashes.
pub fn source_commit_tree_hash(commit: &str) -> Result<String> {
    let output = run_git(&["ls-tree", "-r", commit]).context("git ls-tree")?;
    let hash = crate::hash::sha256_hex(output.as_bytes());
    Ok(hash)
}

/// Compute a SHA-256 hash of the working tree (includes uncommitted changes).
///
/// Only computed when git reports a dirty tree. Lists all tracked files via
/// `git ls-files -z`, reads each file from the working directory (not the
/// index), and hashes the concatenation of `"<path>\0<file_sha256>\n"`.
///
/// NOTE: Untracked files are NOT included — only files git already knows about.
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

/// Run a git command and return stdout as a String.
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

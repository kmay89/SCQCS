// model.rs — Serde data structures for all VBW JSON files
//
// These structs map 1:1 to the JSON Schemas in schemas/vbw/.
// They are used for both serialization (build) and deserialization (verify).
//
// REAL: These are the actual types written to and read from vbw/ bundles.
// They are not demo types — they define the wire format.
//
// NOTE on MaterialEntry.kind: The JSON schema constrains kind to
// enum ["npm", "git", "tarball", "file"], but the Rust struct uses String
// for forward-compatibility. Validation against the schema is the
// responsibility of external tooling, not this code.

use serde::{Deserialize, Serialize};

// ── Manifest ────────────────────────────────────────────────────────────────
// The root document of a witness bundle. Contains hashes of all other files,
// git state, builder identity, and the policy reference. This is the file
// that gets signed.
//
// SIGNING: The Ed25519 signature covers canonical_manifest_bytes(&manifest),
// NOT the pretty-printed JSON on disk. See canonical.rs for the canonical form.

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Manifest {
    pub vbw_version: String,
    pub build_id: String,
    pub created_at: String,
    pub project: Project,
    pub git: GitRef,
    /// SHA-256 of `git ls-tree -r <commit>` output.
    pub source_commit_tree_hash: String,
    /// SHA-256 of worktree file contents. Only present when git.dirty is true.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_worktree_hash: Option<String>,
    /// SHA-256 of materials.lock.json (the file contents, not the lockfiles).
    pub materials_lock_hash: String,
    /// SHA-256 of environment.json.
    pub environment_hash: String,
    /// SHA-256 of outputs.json.
    pub outputs_hash: String,
    pub builder_identity: BuilderIdentity,
    pub policy_ref: PolicyRef,
    /// Records what the build tool actually enforced vs. what was requested.
    /// Always present in bundles produced by VBW v1.0+.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enforcement: Option<Enforcement>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
    /// Extension point for custom fields. Not used by VBW v1.0.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ext: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repo_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub homepage: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitRef {
    pub commit: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub branch: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tag: Option<String>,
    pub dirty: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuilderIdentity {
    /// Human-readable identifier (e.g. "builder@ci", "alice@example.com").
    pub key_id: String,
    /// Base64-encoded Ed25519 public key (44 characters with padding).
    pub public_key_ed25519: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub issuer: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyRef {
    pub path: String,
    pub hash_sha256: String,
}

// ── Enforcement ─────────────────────────────────────────────────────────────
// Records what the build tool actually enforced at build time.
// This is critical for honesty: if Mode A was requested but the tool
// cannot enforce network isolation, this struct says so explicitly.

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Enforcement {
    /// The reproducibility mode that was requested by policy.
    pub mode_requested: ReproducibilityMode,
    /// Whether the requested mode's constraints were actually enforced.
    /// false for Mode A and Mode B in VBW v1.0 (enforcement not implemented).
    pub mode_enforced: bool,
    /// Whether network access was actually blocked during the build.
    pub network_blocked: bool,
    /// Whether SOURCE_DATE_EPOCH was set in the build environment.
    pub source_date_epoch_set: bool,
    /// Human-readable explanation of enforcement gaps.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
}

// ── Environment ─────────────────────────────────────────────────────────────
// Captures the build machine state: OS, tools, container info, and
// reproducibility settings.

#[derive(Debug, Serialize, Deserialize)]
pub struct Environment {
    pub os: OsInfo,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub container: Option<ContainerInfo>,
    pub tools: Vec<ToolInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub env: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub locale: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timezone: Option<String>,
    pub reproducibility: Reproducibility,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OsInfo {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kernel: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arch: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ContainerInfo {
    #[serde(rename = "type")]
    pub container_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image: Option<String>,
    pub image_digest: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ToolInfo {
    pub name: String,
    pub version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub invocation: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Reproducibility {
    pub mode: ReproducibilityMode,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_date_epoch: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub network: Option<NetworkPolicy>,
}

/// The three reproducibility modes defined by the VBW spec.
///
/// Mode A attempts network namespace isolation via `unshare -rn` on Linux.
/// Mode B verifies lockfile integrity before and after the build.
/// Mode C makes no reproducibility promises and is trivially enforceable.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[allow(non_camel_case_types)]
pub enum ReproducibilityMode {
    A_DETERMINISTIC,
    B_LOCKED_NETWORK,
    C_WITNESSED_ND,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NetworkPolicy {
    pub allowed: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allowlist: Option<Vec<String>>,
}

// ── Outputs ─────────────────────────────────────────────────────────────────
// Lists every artifact produced by the build, with SHA-256 hashes and sizes.

#[derive(Debug, Serialize, Deserialize)]
pub struct Outputs {
    pub artifacts: Vec<Artifact>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Artifact {
    pub path: String,
    pub sha256: String,
    pub size_bytes: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mime: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub build_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
}

// ── Policy ──────────────────────────────────────────────────────────────────
// Defines what the build SHOULD do. The build command records the policy;
// the verify command checks compliance after the fact.
//
// Build-time enforcement: Mode A attempts network namespace isolation,
// Mode B checks lockfile integrity before/after build.

#[derive(Debug, Serialize, Deserialize)]
pub struct Policy {
    pub policy_version: String,
    pub requirements: PolicyRequirements,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PolicyRequirements {
    pub network: NetworkRequirement,
    pub reproducibility: ReproducibilityRequirement,
    pub materials: MaterialsRequirement,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signing: Option<SigningRequirement>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NetworkRequirement {
    pub allowed: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allowlist: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ReproducibilityRequirement {
    pub mode: ReproducibilityMode,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub require_source_date_epoch: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MaterialsRequirement {
    pub require_lockfile_hashes: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub require_vendor_archive_and_tree: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SigningRequirement {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub require_maintainer_cosign_for_release: Option<bool>,
    /// Trusted cosigner public keys for co-signature verification.
    /// During verify, each co-signature file in signatures/ is checked
    /// against the matching key_id in this list.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trusted_cosigner_keys: Option<Vec<TrustedCosignerKey>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TrustedCosignerKey {
    /// Human-readable identifier matching the co-signature filename.
    pub key_id: String,
    /// Base64-encoded Ed25519 public key.
    pub public_key_ed25519: String,
}

impl Policy {
    /// Generate a sensible default policy (Mode B, locked network).
    /// Used when no policy.json exists yet.
    pub fn default_policy() -> Self {
        Policy {
            policy_version: "1.0".to_string(),
            requirements: PolicyRequirements {
                network: NetworkRequirement {
                    allowed: true,
                    allowlist: Some(vec![]),
                },
                reproducibility: ReproducibilityRequirement {
                    mode: ReproducibilityMode::B_LOCKED_NETWORK,
                    require_source_date_epoch: Some(false),
                },
                materials: MaterialsRequirement {
                    require_lockfile_hashes: true,
                    require_vendor_archive_and_tree: Some(false),
                },
                signing: Some(SigningRequirement {
                    require_maintainer_cosign_for_release: Some(false),
                    trusted_cosigner_keys: None,
                }),
            },
        }
    }
}

// ── Materials Lock ──────────────────────────────────────────────────────────
// Records which lockfiles were present and their hashes.
//
// TODO: Vendor tarball support (archive_sha256 + extracted_tree_hash)
// is defined in the schema but not yet populated by the build command.
// These fields will always be None in VBW v1.0.

#[derive(Debug, Serialize, Deserialize)]
pub struct MaterialsLock {
    pub lockfiles: Vec<LockfileEntry>,
    pub materials: Vec<MaterialEntry>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LockfileEntry {
    pub path: String,
    pub sha256: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MaterialEntry {
    pub name: String,
    /// One of: "npm", "git", "tarball", "file" (per schema).
    /// Currently only "npm" and "file" are used by auto-detection.
    pub kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    pub sha256: String,
    /// SHA-256 of vendor archive as-downloaded. TODO: Not yet populated.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub archive_sha256: Option<String>,
    /// Canonical hash of extracted vendor archive. TODO: Not yet populated.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extracted_tree_hash: Option<String>,
}

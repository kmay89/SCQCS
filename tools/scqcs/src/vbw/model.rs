use serde::{Deserialize, Serialize};

// ── Manifest ────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize)]
pub struct Manifest {
    pub vbw_version: String,
    pub build_id: String,
    pub created_at: String,
    pub project: Project,
    pub git: GitRef,
    pub source_commit_tree_hash: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_worktree_hash: Option<String>,
    pub materials_lock_hash: String,
    pub environment_hash: String,
    pub outputs_hash: String,
    pub builder_identity: BuilderIdentity,
    pub policy_ref: PolicyRef,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ext: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Project {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repo_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub homepage: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GitRef {
    pub commit: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub branch: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tag: Option<String>,
    pub dirty: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BuilderIdentity {
    pub key_id: String,
    pub public_key_ed25519: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub issuer: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PolicyRef {
    pub path: String,
    pub hash_sha256: String,
}

// ── Environment ─────────────────────────────────────────────────────────────

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
}

impl Policy {
    /// Generate a sensible default policy (Mode B, locked network).
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
                }),
            },
        }
    }
}

// ── Materials Lock ──────────────────────────────────────────────────────────

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
    pub kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    pub sha256: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub archive_sha256: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extracted_tree_hash: Option<String>,
}

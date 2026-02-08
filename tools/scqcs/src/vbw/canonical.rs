// canonical.rs — Deterministic JSON serialization for manifest signing
//
// Implements canonical JSON: sorted object keys, compact format (no whitespace),
// standard JSON string escaping. This is equivalent to JCS (RFC 8785) for our
// use case (no floating-point normalization needed since the manifest contains
// only strings, integers, booleans, and null).
//
// RULE: The Ed25519 signature and manifest SHA-256 hash are ALWAYS computed
// over canonical_manifest_bytes(), never over the pretty-printed file on disk.
// Both `build` and `verify` use this same function.

use serde::Serialize;
use serde_json::Value;

/// Serialize a manifest struct to canonical JSON bytes.
///
/// The canonical form is: sorted object keys at every level, compact
/// (no whitespace), standard JSON escaping. This is the byte sequence
/// that gets signed and hashed.
pub fn canonical_manifest_bytes<T: Serialize>(manifest: &T) -> Vec<u8> {
    let value = serde_json::to_value(manifest).expect("manifest must be serializable to Value");
    canonical_json(&value).into_bytes()
}

/// Produce canonical JSON from a serde_json::Value.
///
/// - Objects: keys sorted lexicographically, no whitespace
/// - Arrays: elements in order, no whitespace
/// - Strings: standard JSON escaping (via serde_json)
/// - Numbers/bools/null: standard JSON representation
pub fn canonical_json(value: &Value) -> String {
    let mut out = String::new();
    write_canonical(value, &mut out);
    out
}

fn write_canonical(value: &Value, out: &mut String) {
    match value {
        Value::Null => out.push_str("null"),
        Value::Bool(b) => out.push_str(if *b { "true" } else { "false" }),
        Value::Number(n) => out.push_str(&n.to_string()),
        Value::String(s) => {
            // Use serde_json's string escaping for correctness
            out.push_str(&serde_json::to_string(s).expect("string serialization cannot fail"));
        }
        Value::Array(arr) => {
            out.push('[');
            for (i, v) in arr.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                write_canonical(v, out);
            }
            out.push(']');
        }
        Value::Object(map) => {
            // Sort keys lexicographically for deterministic output.
            // serde_json::Map preserves insertion order but does NOT
            // guarantee sorted order, so we must sort explicitly.
            let mut keys: Vec<&String> = map.keys().collect();
            keys.sort();

            out.push('{');
            for (i, key) in keys.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                // Key as JSON string
                out.push_str(&serde_json::to_string(*key).expect("key serialization cannot fail"));
                out.push(':');
                write_canonical(&map[*key], out);
            }
            out.push('}');
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hash;
    use crate::vbw::model::*;

    /// Build a deterministic test manifest for golden-vector testing.
    fn test_manifest() -> Manifest {
        Manifest {
            vbw_version: "1.0".to_string(),
            build_id: "test-build-00000000".to_string(),
            created_at: "2026-01-01T00:00:00Z".to_string(),
            project: Project {
                name: "golden-test".to_string(),
                repo_url: None,
                homepage: None,
            },
            git: GitRef {
                commit: "aabbccddee".to_string(),
                branch: Some("main".to_string()),
                tag: None,
                dirty: false,
            },
            source_commit_tree_hash: "a".repeat(64),
            source_worktree_hash: None,
            materials_lock_hash: "b".repeat(64),
            environment_hash: "c".repeat(64),
            outputs_hash: "d".repeat(64),
            builder_identity: BuilderIdentity {
                key_id: "test@golden".to_string(),
                public_key_ed25519: "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=".to_string(),
                issuer: None,
            },
            policy_ref: PolicyRef {
                path: "vbw/policy.json".to_string(),
                hash_sha256: "e".repeat(64),
            },
            notes: None,
            ext: None,
            enforcement: None,
        }
    }

    #[test]
    fn canonical_bytes_are_deterministic() {
        let m = test_manifest();
        let bytes1 = canonical_manifest_bytes(&m);
        let bytes2 = canonical_manifest_bytes(&m);
        assert_eq!(
            bytes1, bytes2,
            "canonical bytes must be identical across calls"
        );
    }

    #[test]
    fn canonical_bytes_golden_hash() {
        // This is a golden test: if the canonical format ever changes,
        // this test MUST be updated intentionally, not silently.
        let m = test_manifest();
        let bytes = canonical_manifest_bytes(&m);
        let hash = hash::sha256_hex(&bytes);

        // The canonical JSON for this manifest (keys sorted, compact):
        let expected_json = canonical_json(&serde_json::to_value(&m).unwrap());
        // Verify it's compact (no newlines, no spaces after colons/commas)
        assert!(
            !expected_json.contains('\n'),
            "canonical JSON must not contain newlines"
        );
        assert!(
            !expected_json.contains(": "),
            "canonical JSON must not have space after colon"
        );
        assert!(
            !expected_json.contains(", "),
            "canonical JSON must not have space after comma"
        );

        // Golden hash — if this changes, canonicalization broke.
        // To update: run test, get actual hash, verify the canonical JSON
        // is correct, then update this constant.
        let golden = hash::sha256_hex(expected_json.as_bytes());
        assert_eq!(hash, golden, "canonical hash must match golden vector");

        // Hard-coded golden hash. If canonicalization or struct field order
        // changes, this test fails, forcing intentional review.
        assert_eq!(
            hash, "9641ebc924afa024809871ac2e3c94d177e8e5823d4ecb42f681d0f188b6516b",
            "canonical hash must match hardcoded golden vector"
        );
    }

    #[test]
    fn canonical_omits_none_fields() {
        let m = test_manifest();
        let json = canonical_json(&serde_json::to_value(&m).unwrap());
        // source_worktree_hash is None and has skip_serializing_if, so
        // it should NOT appear in the output.
        assert!(
            !json.contains("source_worktree_hash"),
            "None fields with skip_serializing_if must be omitted"
        );
        assert!(!json.contains("notes"), "None notes must be omitted");
        assert!(!json.contains("ext"), "None ext must be omitted");
    }

    #[test]
    fn canonical_sorts_object_keys() {
        let json_str = r#"{"z":1,"a":2,"m":3}"#;
        let value: Value = serde_json::from_str(json_str).unwrap();
        let canonical = canonical_json(&value);
        assert_eq!(canonical, r#"{"a":2,"m":3,"z":1}"#);
    }

    #[test]
    fn canonical_handles_nested_objects() {
        let json_str = r#"{"b":{"z":1,"a":2},"a":true}"#;
        let value: Value = serde_json::from_str(json_str).unwrap();
        let canonical = canonical_json(&value);
        assert_eq!(canonical, r#"{"a":true,"b":{"a":2,"z":1}}"#);
    }

    #[test]
    fn canonical_handles_arrays() {
        let json_str = r#"[3,1,2]"#;
        let value: Value = serde_json::from_str(json_str).unwrap();
        let canonical = canonical_json(&value);
        // Arrays preserve order
        assert_eq!(canonical, r#"[3,1,2]"#);
    }

    #[test]
    fn canonical_escapes_strings() {
        let json_str = r#"{"key":"hello \"world\"\nnewline"}"#;
        let value: Value = serde_json::from_str(json_str).unwrap();
        let canonical = canonical_json(&value);
        assert_eq!(canonical, r#"{"key":"hello \"world\"\nnewline"}"#);
    }

    #[test]
    fn canonical_pretty_roundtrip() {
        // Verify that pretty-printing and re-parsing produces the same
        // canonical output (this is the core guarantee for verify).
        let m = test_manifest();
        let pretty = serde_json::to_string_pretty(&m).unwrap();
        let reparsed: Manifest = serde_json::from_str(&pretty).unwrap();
        let bytes_original = canonical_manifest_bytes(&m);
        let bytes_reparsed = canonical_manifest_bytes(&reparsed);
        assert_eq!(
            bytes_original, bytes_reparsed,
            "canonical bytes must survive pretty-print roundtrip"
        );
    }

    #[test]
    fn sign_and_verify_canonical_bytes() {
        let m = test_manifest();
        let canonical_bytes = canonical_manifest_bytes(&m);

        let (sk, pk) = crate::sign::keygen();
        let sig = crate::sign::sign(&sk, &canonical_bytes).unwrap();

        // Verify against canonical bytes succeeds
        assert!(crate::sign::verify(&pk, &canonical_bytes, &sig).unwrap());

        // Verify against pretty bytes MUST fail (different bytes)
        let pretty_bytes = serde_json::to_string_pretty(&m).unwrap();
        assert!(
            !crate::sign::verify(&pk, pretty_bytes.as_bytes(), &sig).unwrap(),
            "signature over canonical bytes must not verify against pretty-printed bytes"
        );
    }
}

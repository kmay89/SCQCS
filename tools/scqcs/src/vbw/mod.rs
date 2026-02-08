// vbw/ — Verified Build Witness core logic
//
// model.rs  — Data structures (serde) matching the JSON schemas
// build.rs  — Build command: run build, capture environment, generate bundle
// verify.rs — Verify command: check hashes, signature, policy compliance

pub mod build;
pub mod canonical;
pub mod model;
pub mod verify;

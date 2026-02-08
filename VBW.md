# Verified Build Witness (VBW) v1.0

**Cryptographic proof that your build is what it claims to be.**

VBW creates tamper-evident records proving that specific source code, built with a specific toolchain, in a specific environment, produced specific artifacts. Every build ships with a signed witness bundle that anyone can independently verify.

---

## Status

VBW v1.0 is a **working implementation** — the CLI builds, signs, and verifies real bundles. The core pipeline (hashing, signing, verification) is production-grade cryptography.

**What works today:**
- Ed25519 key generation, signing, and verification (real, not demo)
- SHA-256 hashing of all source trees, lockfiles, and output artifacts (real)
- Git commit/branch/dirty detection (real)
- Build transcript capture (real, with a limitation noted below)
- Full verify pipeline: hash checks, signature verification, policy compliance
- GitHub Actions integration

**What is not yet implemented (TODOs):**
- Build-time policy enforcement (Mode A does not block network; Mode B does not verify dependency sources)
- Co-signature (attest) verification during `verify` (signatures are written but not checked)
- Vendor tarball hashing (`archive_sha256` / `extracted_tree_hash` fields are always empty)
- Source tree hash re-verification during `verify` (the stored hash is checked for integrity but not recomputed from the local repo)
- Schema validation of bundle JSON against the published schemas

**Known limitations:**
- Build transcript captures stdout fully, then stderr (not interleaved)
- Environment capture requires Unix (`uname`, `which`) — falls back to "unknown" on other platforms
- Container detection is heuristic (checks `/.dockerenv`, `/proc/self/cgroup`)

---

## Why VBW Exists

When you download software, you trust that the binary matches the source code. But how do you *prove* it? VBW answers six questions about every build:

| Question | VBW Answer |
|----------|------------|
| What exact source was built? | Git commit hash + canonical source tree hash |
| What tools compiled it? | Compiler/runtime versions captured in `environment.json` |
| What OS/container ran the build? | OS, kernel, architecture, container digest |
| Can this build be reproduced? | Reproducibility mode recorded (see note below) |
| Has the output been tampered with? | SHA-256 hashes of every artifact in `outputs.json` |
| Who attested to all of this? | Ed25519 signature over the manifest |

> **Note on reproducibility:** VBW v1.0 *records* the reproducibility mode but does not *enforce* it. Selecting Mode A does not actually block network access during the build. Enforcement is planned for a future version. The mode is an honest declaration by the builder, verified against policy at audit time.

---

## Quick Start

### Prerequisites

- [Rust toolchain](https://rustup.rs/) (1.70+)
- Git

### 1. Build the CLI

```bash
cd tools/scqcs
cargo build --release
```

The binary is at `tools/scqcs/target/release/scqcs`.

### 2. Generate a Signing Key

```bash
./tools/scqcs/target/release/scqcs vbw keygen --output ~/.scqcs
```

Output:
```
Ed25519 keypair generated:
  Secret key: /home/you/.scqcs/vbw-builder.sk
  Public key: /home/you/.scqcs/vbw-builder.pk

Public key (base64): <your-public-key>

To use in CI, set the secret key as:
  SCQCS_VBW_ED25519_SK_B64=<your-secret-key>
```

Keep `vbw-builder.sk` secret. Share `vbw-builder.pk` publicly.

### 3. Run a Witnessed Build

```bash
./tools/scqcs/target/release/scqcs vbw build \
  --keyfile ~/.scqcs/vbw-builder.sk \
  --project my-project \
  --output-dir dist \
  -- npm run build
```

This runs `npm run build`, then generates a `vbw/` directory with the full witness bundle.

### 4. Verify the Bundle

```bash
./tools/scqcs/target/release/scqcs vbw verify --bundle vbw
```

Output:
```
[vbw] Verifying build: a1b2c3d4-...
[vbw] Project: my-project
[vbw] Git commit: abc1234...
[vbw] Manifest hash: OK
[vbw] Builder signature: OK
[vbw] environment.json: OK
[vbw] materials.lock.json: OK
[vbw] outputs.json: OK
[vbw] Policy hash: OK
[vbw] Output artifacts: 12 checked

VERIFIED
```

---

## What's in a Witness Bundle

After a build, the `vbw/` directory contains:

```
vbw/
  manifest.json               # The signed statement — links everything together
  environment.json             # OS, compiler versions, container digest
  materials.lock.json          # Dependency lockfile hashes
  outputs.json                 # Artifact paths, SHA-256 hashes, sizes
  transcript.txt               # Full build log (stdout/stderr, sequential)
  policy.json                  # Build policy requirements
  signatures/
    builder.ed25519.sig        # Ed25519 signature over manifest.json
  hashes/
    manifest.sha256            # SHA-256 of manifest.json
```

### manifest.json

The core document. It doesn't contain data directly — it contains *hashes* of all other files, creating a single signed root of trust.

```json
// ILLUSTRATIVE EXAMPLE — hashes are shortened placeholders, not real values
{
  "vbw_version": "1.0",
  "build_id": "a1b2c3d4-e5f6-...",
  "created_at": "2026-02-08T12:00:00Z",
  "project": { "name": "my-project" },
  "git": {
    "commit": "abc1234def5678...",
    "branch": "main",
    "dirty": false
  },
  "source_commit_tree_hash": "aabbccdd...(64 hex chars total)...",
  "materials_lock_hash": "11223344...(64 hex chars total)...",
  "environment_hash": "55667788...(64 hex chars total)...",
  "outputs_hash": "99aabbcc...(64 hex chars total)...",
  "builder_identity": {
    "key_id": "builder@ci",
    "public_key_ed25519": "Base64EncodedEd25519PublicKey44chars="
  },
  "policy_ref": {
    "path": "vbw/policy.json",
    "hash_sha256": "ddeeff00...(64 hex chars total)..."
  }
}
```

### environment.json

Captures the build machine state. This is real data captured from the OS at build time.

```json
// ILLUSTRATIVE EXAMPLE — values will differ on your machine
{
  "os": {
    "name": "Linux",
    "version": "6.5.0-44-generic",
    "kernel": "#44-Ubuntu SMP ...",
    "arch": "x86_64"
  },
  "tools": [
    { "name": "node", "version": "v20.11.0", "path": "/usr/bin/node" },
    { "name": "npm", "version": "10.2.4", "path": "/usr/bin/npm" }
  ],
  "reproducibility": {
    "mode": "B_LOCKED_NETWORK",
    "network": { "allowed": true, "allowlist": [] }
  }
}
```

### outputs.json

Every artifact is hashed and measured. These are real SHA-256 hashes of the actual files.

```json
// ILLUSTRATIVE EXAMPLE — hashes and sizes are placeholders
{
  "artifacts": [
    {
      "path": "dist/index.html",
      "sha256": "aabbccdd...(64 hex chars)...",
      "size_bytes": 15234,
      "mime": "text/html"
    },
    {
      "path": "dist/main.js",
      "sha256": "eeff0011...(64 hex chars)...",
      "size_bytes": 42891,
      "mime": "application/javascript"
    }
  ]
}
```

---

## CLI Reference

### `scqcs vbw keygen`

Generate an Ed25519 keypair for signing builds.

```bash
scqcs vbw keygen [--output <dir>]
```

| Option | Default | Description |
|--------|---------|-------------|
| `--output` | `.` (current dir) | Directory to write key files into |

Produces two files:
- `vbw-builder.sk` — secret key (keep private, use in CI as a secret)
- `vbw-builder.pk` — public key (distribute freely)

### `scqcs vbw build`

Run a build command and generate a witness bundle.

```bash
scqcs vbw build [options] -- <build-command...>
```

| Option | Default | Description |
|--------|---------|-------------|
| `--project` | Directory name | Project name in the manifest |
| `--output-dir` | `dist` | Where build artifacts live |
| `--keyfile` | — | Path to Ed25519 secret key file |
| `--key-id` | `builder@local` | Human-readable key identifier |
| `--policy` | `vbw/policy.json` | Path to policy file |

The signing key can also be provided via the `SCQCS_VBW_ED25519_SK_B64` environment variable (preferred for CI).

**What happens during build:**

1. Loads or auto-generates `policy.json`
2. Snapshots the environment (OS, tools, container)
3. Detects lockfiles (`package-lock.json`, `Cargo.lock`, `go.sum`, etc.)
4. Records git commit, branch, dirty status
5. Computes canonical source tree hash via `git ls-tree`
6. Runs your build command, capturing the full transcript
7. Hashes every artifact in the output directory
8. Assembles the manifest referencing all component hashes
9. Signs the manifest with the builder's Ed25519 key
10. Writes everything to `vbw/`

### `scqcs vbw verify`

Verify a witness bundle's integrity and signatures.

```bash
scqcs vbw verify [--bundle <dir>]
```

| Option | Default | Description |
|--------|---------|-------------|
| `--bundle` | `vbw` | Path to the witness bundle directory |

**Verification checks:**

1. Recomputes `manifest.sha256` and compares to stored hash
2. Verifies Ed25519 signature against the public key in the manifest
3. Loads each component file, recomputes its hash, compares to manifest
4. Checks that output artifacts exist and match `outputs.json` hashes
5. Validates policy compliance (dirty tree warning, mode mismatch, lockfile presence)

**What verify does NOT check (TODOs):**
- Co-signatures from `attest` are not verified
- Source tree hash is not recomputed from the local git repo
- JSON files are not validated against the published schemas

**Exit codes:**
- `0` — Verified (or verified with variance)
- `1` — Unverified (integrity failure)

**Three verdicts:**

| Verdict | Meaning |
|---------|---------|
| VERIFIED | All hashes match, signature valid, policy satisfied |
| VERIFIED WITH VARIANCE | Signature and hashes OK, but warnings (e.g., dirty tree, missing lockfiles) |
| UNVERIFIED | Hash mismatch, bad signature, or missing files |

### `scqcs vbw attest`

Add a maintainer co-signature to an existing bundle.

```bash
scqcs vbw attest [--bundle <dir>] [--keyfile <path>] [--key-id <id>]
```

Use this when a second person (a maintainer, auditor, or release manager) independently reviews the bundle and wants to add their own signature.

```bash
scqcs vbw attest --bundle vbw --keyfile ~/.scqcs/maintainer.sk --key-id "maintainer@org"
```

This writes a new file: `vbw/signatures/maintainer_org.ed25519.sig`

> **Note:** `verify` does not yet check co-signatures — it only verifies the builder signature. Co-signature verification is a TODO for VBW v1.1.

---

## Reproducibility Modes

VBW defines three levels of build reproducibility. In v1.0, these are **recorded as declarations** — they describe the builder's intent but are not actively enforced by the tool.

### Mode A: Deterministic

The strictest mode. Declares that identical inputs produce identical outputs, byte-for-byte.

- **Intent:** No network access, pinned toolchain, `SOURCE_DATE_EPOCH` set
- **Reality in v1.0:** The mode is recorded but the tool does not block network access or enforce timestamp normalization. The builder is making a promise that auditors can check manually.

### Mode B: Locked Network (Default)

A practical middle ground. Declares that network access is only used for fetching locked, hashed dependencies.

- **Intent:** Dependencies come from lockfiles with recorded hashes
- **Reality in v1.0:** The tool records lockfile hashes but does not verify that the build only fetched from those lockfiles. It's a record of what lockfiles existed, not a guarantee the build respected them.

### Mode C: Witnessed Non-Deterministic

Provenance and integrity without a reproducibility guarantee.

- Full network access allowed
- Build may not be reproducible
- Still records what happened: who built it, what tools, what outputs
- Useful for complex builds that can't (yet) be made deterministic

---

## Setting Up CI (GitHub Actions)

The included workflow at `.github/workflows/vbw-build.yml` automates VBW for every push to `main`.

### Step 1: Generate a Key

On your local machine:

```bash
scqcs vbw keygen --output /tmp/vbw-keys
cat /tmp/vbw-keys/vbw-builder.sk
```

### Step 2: Add the Secret to GitHub

1. Go to your repo on GitHub
2. **Settings** > **Secrets and variables** > **Actions**
3. Click **New repository secret**
4. Name: `VBW_BUILDER_SK_B64`
5. Value: paste the contents of `vbw-builder.sk`

### Step 3: The Workflow

The workflow in this repo does the following:

```yaml
# .github/workflows/vbw-build.yml
name: VBW Build (SCQCS site)
on:
  push:
    branches: ["main"]

jobs:
  build_vbw:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
        with:
          fetch-depth: 0          # Full history for git tree hashing

      - name: Install Rust toolchain
        uses: dtolnay/rust-toolchain@stable

      - name: Build scqcs CLI tool
        run: cargo build --release
        working-directory: tools/scqcs

      - name: VBW bundle
        env:
          SCQCS_VBW_ED25519_SK_B64: ${{ secrets.VBW_BUILDER_SK_B64 }}
        run: |
          ./tools/scqcs/target/release/scqcs vbw build \
            --project scqcs-site \
            --output-dir dist \
            -- echo "Static site copied to dist/"

      - uses: actions/upload-artifact@v4
        with:
          name: scqcs-site-dist-and-vbw
          path: |
            dist/**
            vbw/**
```

> **Note:** This workflow will fail until you add the `VBW_BUILDER_SK_B64` secret to GitHub. Without it, the `scqcs vbw build` command will error with "No signing key found."

After each push, the witness bundle is uploaded as a build artifact alongside the site.

### Step 4: Verify Locally

Download the artifact from the Actions tab, then:

```bash
scqcs vbw verify --bundle vbw
```

---

## Adapting VBW for Your Project

VBW works with any build system. Here are common setups:

### Node.js / npm

```bash
scqcs vbw build --output-dir dist -- npm run build
```

VBW auto-detects `package-lock.json` and records its hash.

### Rust / Cargo

```bash
scqcs vbw build --output-dir target/release -- cargo build --release
```

VBW auto-detects `Cargo.lock` and records its hash.

### Go

```bash
scqcs vbw build --output-dir bin -- go build -o bin/myapp ./cmd/myapp
```

VBW auto-detects `go.sum` and records its hash.

### Static Sites (No Build Step)

If your site is already built (pure HTML/CSS/JS), use a copy step as the build command:

```bash
mkdir -p dist && cp -r *.html *.css *.js dist/
scqcs vbw build --output-dir dist -- echo "Static site copied"
```

This is what the SCQCS site itself uses. The build command (`echo`) is trivial — VBW still captures the full environment and hashes all output artifacts.

### Custom Output Directory

The `--output-dir` flag tells VBW where to find artifacts to hash:

```bash
scqcs vbw build --output-dir build/release -- make release
```

---

## Policy Configuration

The policy file controls what the build is *expected* to do. VBW auto-generates a default if none exists.

> **Important:** In v1.0, policy is checked at verify time only. The build command does not enforce policy constraints (it won't block network access for Mode A, for example). Policy enforcement at build time is a TODO.

### Default Policy (Mode B)

```json
{
  "policy_version": "1.0",
  "requirements": {
    "network": {
      "allowed": true,
      "allowlist": []
    },
    "reproducibility": {
      "mode": "B_LOCKED_NETWORK",
      "require_source_date_epoch": false
    },
    "materials": {
      "require_lockfile_hashes": true,
      "require_vendor_archive_and_tree": false
    },
    "signing": {
      "require_maintainer_cosign_for_release": false
    }
  }
}
```

### Strict Policy (Mode A)

For declaring maximum reproducibility intent:

```json
{
  "policy_version": "1.0",
  "requirements": {
    "network": {
      "allowed": false,
      "allowlist": []
    },
    "reproducibility": {
      "mode": "A_DETERMINISTIC",
      "require_source_date_epoch": true
    },
    "materials": {
      "require_lockfile_hashes": true,
      "require_vendor_archive_and_tree": true
    },
    "signing": {
      "require_maintainer_cosign_for_release": true
    }
  }
}
```

> **Note:** Setting `"allowed": false` records the intent but does not block network. Setting `"require_vendor_archive_and_tree": true` will produce a verify warning since vendor tarball hashing is not yet implemented.

To use a custom policy, save it and pass it via `--policy`:

```bash
scqcs vbw build --policy strict-policy.json -- npm run build
```

---

## How Verification Works

Verification rebuilds the chain of trust from the inside out:

```
manifest.json
  |
  |-- hashes/manifest.sha256       Does the stored hash match the file?
  |-- signatures/builder.ed25519   Does the signature match the public key?
  |
  |-- environment_hash             Recompute hash of environment.json, compare
  |-- materials_lock_hash          Recompute hash of materials.lock.json, compare
  |-- outputs_hash                 Recompute hash of outputs.json, compare
  |-- policy_ref.hash_sha256       Recompute hash of policy.json, compare
  |
  outputs.json
    |-- artifact[0].sha256         Does dist/index.html still match?
    |-- artifact[1].sha256         Does dist/main.js still match?
    ...
```

If any hash doesn't match, the verdict is **UNVERIFIED**. This catches:
- Modified artifacts (someone changed a file after the build)
- Modified metadata (someone altered the environment or materials record)
- Forged signatures (someone tried to re-sign with a different key)

**What verification does not catch:**
- Tampering that occurred *during* the build (compromised build environment)
- A compromised signing key used to produce a valid-but-malicious bundle
- Builds that violated their declared reproducibility mode

---

## Project Layout

```
tools/scqcs/
  Cargo.toml                    # Rust project manifest
  src/
    main.rs                     # CLI entry point, keygen + attest commands
    cli.rs                      # clap command definitions
    hash.rs                     # SHA-256 hashing utilities
    git.rs                      # Git state detection and tree hashing
    sign.rs                     # Ed25519 key generation, signing, verification
    vbw/
      mod.rs                    # Module declarations
      model.rs                  # Serde structs matching all JSON schemas
      build.rs                  # Build workflow (13-step pipeline)
      verify.rs                 # Verification workflow (8-step pipeline)

schemas/vbw/
  manifest-1.0.schema.json     # JSON Schema for manifest.json
  environment-1.0.schema.json  # JSON Schema for environment.json
  outputs-1.0.schema.json      # JSON Schema for outputs.json
  policy-1.0.schema.json       # JSON Schema for policy.json
  materials-lock-1.0.schema.json  # JSON Schema for materials.lock.json

.github/workflows/
  vbw-build.yml                 # CI workflow for automated VBW bundles
```

---

## JSON Schemas

All VBW files conform to JSON Schemas published in `schemas/vbw/`. These schemas can be used by editors for autocomplete and validation, or by external tools that consume VBW bundles.

| Schema | Validates |
|--------|-----------|
| `manifest-1.0.schema.json` | `vbw/manifest.json` |
| `environment-1.0.schema.json` | `vbw/environment.json` |
| `outputs-1.0.schema.json` | `vbw/outputs.json` |
| `policy-1.0.schema.json` | `vbw/policy.json` |
| `materials-lock-1.0.schema.json` | `vbw/materials.lock.json` |

> **Note:** The CLI does not validate bundle files against these schemas. The schemas are published for external tooling and documentation. Runtime schema validation is a TODO.

---

## Security Model

### What VBW Proves (real, implemented)

- The build artifacts match the hashes recorded at build time (SHA-256)
- The manifest was signed by the holder of the declared Ed25519 key
- The environment, dependencies, and policy files have not been modified since signing
- The git commit and dirty status were accurately recorded at build time

### What VBW Does Not Prove

- That the source code is free of vulnerabilities
- That the signing key hasn't been compromised
- That the build environment wasn't itself compromised
- That the build is reproducible (the mode is a declaration, not a proof)
- That dependencies were actually fetched from lockfile-specified sources

VBW is one layer in a defense-in-depth strategy. It answers "what happened during this build?" with cryptographic certainty, but it doesn't answer "should this build be trusted?" — that's a policy decision for humans.

### Key Management

| Method | When to Use |
|--------|-------------|
| `--keyfile path/to/key.sk` | Local development |
| `SCQCS_VBW_ED25519_SK_B64` env var | CI/CD pipelines |

The secret key is a 32-byte Ed25519 seed, base64-encoded. Never commit it to the repository. In CI, store it as a repository secret.

---

## Lockfile Auto-Detection

VBW automatically detects and hashes these lockfiles if they exist in the project root:

| Lockfile | Ecosystem |
|----------|-----------|
| `package-lock.json` | npm |
| `yarn.lock` | Yarn |
| `pnpm-lock.yaml` | pnpm |
| `Cargo.lock` | Rust/Cargo |
| `go.sum` | Go |
| `Gemfile.lock` | Ruby/Bundler |
| `poetry.lock` | Python/Poetry |
| `composer.lock` | PHP/Composer |
| `Pipfile.lock` | Python/Pipenv |

---

## Platform Support

VBW targets Unix/Linux environments and CI runners (GitHub Actions, Docker). Environment detection uses `uname` for OS information and `which` for tool paths. On non-Unix systems, OS fields will report "unknown" but the core signing and verification workflow functions correctly on any platform where Rust compiles.

---

## All TODOs in One Place

For quick reference, every TODO mentioned in this document and in the code:

| TODO | Where | Priority |
|------|-------|----------|
| Build-time policy enforcement (network blocking, epoch, etc.) | `build.rs`, `model.rs` | High |
| Co-signature verification in `verify` | `verify.rs` | High |
| Source tree hash re-verification during `verify` | `verify.rs` | Medium |
| Vendor tarball hashing (`archive_sha256`, `extracted_tree_hash`) | `build.rs`, `model.rs` | Medium |
| Runtime JSON schema validation | `verify.rs` | Low |
| Richer material kind values in schema (cargo, go, ruby) | `build.rs`, schema | Low |
| Interleaved stdout/stderr transcript capture | `build.rs` | Low |
| Transparency log integration | Roadmap (VBW-2) | Future |
| Multi-builder consensus (N-of-M signatures) | Roadmap (VBW-2) | Future |
| OIDC identity binding | Roadmap (VBW-2) | Future |
| SBOM integration | Roadmap (VBW-2) | Future |

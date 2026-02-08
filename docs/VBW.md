# Verified Build Witness (VBW) v1.0

**Cryptographic proof that your build is what it claims to be.**

VBW creates tamper-evident records proving that specific source code, built with a specific toolchain, in a specific environment, produced specific artifacts. Every build ships with a signed witness bundle that anyone can independently verify.

---

## The Problem: You Can't Prove What Built Your Software

When someone downloads your software, they trust that the binary matches the source code. But how would they verify that? Today, most projects ship artifacts with no proof of origin. The gap between "source code on GitHub" and "binary on your machine" is a black box — and attackers know it. Supply chain attacks (SolarWinds, codecov, event-stream) exploit exactly this gap.

## How Teams Normally Handle This

The industry has responded with several approaches. Here's what conventional best practice looks like:

| Approach | What It Does | Limitations |
|----------|-------------|-------------|
| **Sigstore / cosign** | Signs artifacts using ephemeral OIDC keys; records signatures in a transparency log | Requires an external transparency log service (Rekor). Signatures prove *who* signed, not *how* it was built. No environment or dependency capture. |
| **SLSA Framework** | Defines 4 levels of supply chain security maturity; provenance metadata via in-toto attestations | Provenance is a separate attestation stored in a registry, not alongside the artifact. Achieving Level 3+ requires a hardened build platform. Complex to adopt. |
| **in-toto** | Defines a layout of expected build steps; each step produces a signed "link" attestation | Powerful but heavyweight — requires defining a full layout of steps, functionaries, and inspection rules before you start. Designed for multi-party pipelines, not single-team builds. |
| **Reproducible Builds** | Ensures identical source produces byte-identical output | Extremely difficult to achieve in practice. Many build tools embed timestamps, paths, or randomness. Proves *reproducibility* but not *provenance*. |
| **GPG-signed tags/releases** | Developer signs a git tag or release archive with their GPG key | Proves who signed, not what environment built it. No artifact hashing, no dependency capture, no build transcript. Key management is painful. |

These are good tools. VBW doesn't replace them — it addresses a gap they leave open.

## What VBW Does Differently

VBW is a **build-time witness** — it wraps your existing build command, captures everything that happened, and packages the evidence into a single portable bundle that ships alongside your artifacts. No external services required.

Here's what makes it distinct:

### 1. The witness bundle is self-contained and ships with your code

Conventional tools store provenance in a separate registry, transparency log, or attestation store. VBW produces a `vbw/` directory that lives right next to your build output. Anyone with the bundle can verify it — no network calls, no registry lookups, no accounts.

### 2. It captures the full build context, not just a signature

Most signing tools answer "who signed this?" VBW answers six questions: **what source** was built, **what tools** compiled it, **what OS/container** ran the build, **what dependencies** were locked, **what artifacts** were produced, and **who attested** to all of it. Every answer is hashed and signed together.

### 3. It works with any build system, right now

No layout files to define. No build platform to migrate to. No SLSA level to achieve first. Run `scqcs vbw build -- npm run build` and you get a complete witness bundle. Works with npm, Cargo, Go, Make, or a plain `cp` command.

### 4. Progressive reproducibility — declare what you can, prove what you do

Instead of requiring full determinism (which most projects can't achieve), VBW defines three modes:
- **Mode A** (Deterministic) — no network, pinned tools, byte-identical output
- **Mode B** (Locked Network) — dependencies from lockfiles, practical for most teams
- **Mode C** (Witnessed Non-Deterministic) — full provenance without a reproducibility guarantee

You pick the mode that matches your reality. The bundle honestly records which mode was declared.

### 5. Human-readable, auditable, no special tooling to inspect

Every file in the bundle is plain JSON. You can `cat manifest.json` and read it. You can diff two bundles with standard tools. You can write your own verifier in any language. The format is not a binary blob or a protobuf — it's designed for humans and machines equally.

## Why This Matters

If you ship software and can't answer "prove this binary came from that commit" — VBW gives you that answer in one command. If you already use Sigstore or SLSA, VBW complements them by capturing the build context those tools don't record.

The goal is not to replace the ecosystem. It's to make build provenance **accessible enough that small teams actually use it** instead of treating it as a someday problem.

---

## Status

VBW v1.0 is a **working implementation** — the CLI builds, signs, and verifies real bundles. The core pipeline (hashing, signing, verification) is production-grade cryptography.

**What works today:**
- Ed25519 key generation, signing, and verification (real, not demo)
- SHA-256 hashing of all source trees, lockfiles, and output artifacts (real, streaming for large files)
- Canonical JSON signing: signature covers deterministic canonical manifest bytes (sorted keys, compact JSON), not the pretty-printed file on disk
- Git commit/branch/dirty detection (real)
- Build transcript capture with interleaved stdout/stderr and ISO-8601 timestamps
- Strict fail-closed verify pipeline: hash checks, signature verification, bundle completeness, unexpected file detection, path traversal rejection, symlink escape detection
- Enforcement honesty: manifest records what was actually enforced vs. requested. Mode A attempts network namespace isolation via `unshare -rn`; Mode B checks lockfile integrity before/after build.
- GitHub Actions integration

**What is not yet implemented (TODOs):**
- Vendor tarball hashing (`archive_sha256` / `extracted_tree_hash` fields are always empty)
- Source tree hash re-verification during `verify` (the stored hash is checked for integrity but not recomputed from the local repo)
- Schema validation of bundle JSON against the published schemas
- Individual dependency artifact verification from lockfiles. **Lockfiles are hashed; individual dependency artifact verification is future work.**

**Known limitations:**
- Environment capture requires Unix (`uname`, `which`) — falls back to "unknown" on other platforms
- Container detection is heuristic (checks `/.dockerenv`, `/proc/self/cgroup`)

---

## At a Glance: What a VBW Bundle Answers

| Question | How VBW Answers It |
|----------|------------|
| What exact source was built? | Git commit hash + canonical source tree hash |
| What tools compiled it? | Compiler/runtime versions captured in `environment.json` |
| What OS/container ran the build? | OS, kernel, architecture, container digest |
| Can this build be reproduced? | Reproducibility mode recorded (Mode A/B/C) |
| Has the output been tampered with? | SHA-256 hashes of every artifact in `outputs.json` |
| Who attested to all of this? | Ed25519 signature over canonical manifest bytes |

> **Note on reproducibility:** VBW attempts to enforce reproducibility modes at build time. Mode A uses `unshare -rn` for network namespace isolation and sets `SOURCE_DATE_EPOCH`. Mode B snapshots lockfile hashes before and after the build to detect modifications. If enforcement partially fails (e.g., user namespaces unavailable for Mode A), the manifest honestly records `mode_enforced=false` with a note explaining what could not be enforced.

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
  transcript.txt               # Full build log (interleaved stdout/stderr with timestamps)
  policy.json                  # Build policy requirements
  signatures/
    builder.ed25519.sig        # Ed25519 signature over canonical manifest bytes
  hashes/
    manifest.sha256            # SHA-256 of canonical manifest bytes
```

> **Canonical signing:** The signature and manifest hash are computed over *canonical manifest bytes* (sorted keys, compact JSON), not the pretty-printed `manifest.json` file on disk. The file on disk is human-readable; verification re-canonicalizes the parsed manifest to check the signature. This ensures byte-level signing stability regardless of JSON formatting.

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

**Verification checks (strict, fail-closed):**

1. Validates bundle directory exists and is a real directory
2. Checks all required files are present (manifest, environment, materials, outputs, transcript, policy, signature, hash)
3. Rejects unexpected files in the bundle (strict bundle policy — extra files are an error)
4. Checks for symlinks that escape the bundle directory
5. Parses manifest, re-canonicalizes to canonical bytes (sorted keys, compact JSON)
6. Recomputes manifest hash from canonical bytes and compares to `hashes/manifest.sha256`
7. Verifies Ed25519 signature against canonical manifest bytes
8. Loads each component file, recomputes its SHA-256 hash, compares to manifest reference
9. Verifies co-signatures against `trusted_cosigner_keys` from the policy. If `require_maintainer_cosign_for_release` is true, at least one valid co-signature must be present.
10. Checks output artifacts exist and match `outputs.json` hashes (with path traversal rejection)
11. Validates enforcement consistency (mode_requested matches policy mode)
12. Validates policy compliance (dirty tree warning, mode mismatch, lockfile presence)

**What verify does NOT check (TODOs):**
- Source tree hash is not recomputed from the local git repo
- JSON files are not validated against the published schemas

**Exit codes:**
- `0` — Verified (or verified with variance)
- `1` — Unverified (integrity failure, missing files, unexpected files, bad signature)

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

> **Note:** `verify` checks co-signatures against `trusted_cosigner_keys` listed in the policy. If the policy sets `require_maintainer_cosign_for_release: true`, at least one valid co-signature must be present. Co-signer public keys must be declared in the policy for verification to succeed.

---

## Reproducibility Modes

VBW defines three levels of build reproducibility with active enforcement.

### Mode A: Deterministic

The strictest mode. Declares that identical inputs produce identical outputs, byte-for-byte.

- **Intent:** No network access, pinned toolchain, `SOURCE_DATE_EPOCH` set
- **Enforcement:** VBW attempts network namespace isolation via `unshare -rn` (Linux user namespaces) and sets `SOURCE_DATE_EPOCH` if not already present. If network isolation succeeds and `SOURCE_DATE_EPOCH` is set, the manifest records `mode_enforced=true`. If `unshare` fails (e.g., user namespaces disabled), the manifest records `mode_enforced=false` with a diagnostic note.

### Mode B: Locked Network (Default)

A practical middle ground. Declares that network access is only used for fetching locked, hashed dependencies.

- **Intent:** Dependencies come from lockfiles with recorded hashes
- **Enforcement:** VBW snapshots all lockfile hashes (package-lock.json, Cargo.lock, etc.) before the build and compares them after the build completes. If any lockfile was modified during the build, `mode_enforced=false` is recorded. If lockfiles are unchanged, `mode_enforced=true`.

### Mode C: Witnessed Non-Deterministic

Provenance and integrity without a reproducibility guarantee.

- Full network access allowed
- Build may not be reproducible
- Still records what happened: who built it, what tools, what outputs
- Useful for complex builds that can't (yet) be made deterministic
- `mode_enforced=true` because Mode C makes no reproducibility promises that need enforcement.

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

> **Note:** Policy is checked at both build time (enforcement) and verify time (compliance). The build command enforces Mode A (network isolation) and Mode B (lockfile integrity). The verify command checks co-signatures against `trusted_cosigner_keys` in the policy.

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

- The build artifacts match the hashes recorded at build time (SHA-256, streaming for large files)
- The manifest was signed by the holder of the declared Ed25519 key (signature covers canonical manifest bytes)
- The environment, dependencies, and policy files have not been modified since signing
- The git commit and dirty status were accurately recorded at build time
- The bundle has not been tampered with (strict verification rejects unexpected files, symlink escapes, path traversal)
- The enforcement field honestly records what was actually enforced vs. requested

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
| Source tree hash re-verification during `verify` | `verify.rs` | Medium |
| Vendor tarball hashing (`archive_sha256`, `extracted_tree_hash`) | `build.rs`, `model.rs` | Medium |
| Individual dependency artifact verification from lockfiles | `build.rs` | Medium |
| Runtime JSON schema validation | `verify.rs` | Low |
| Richer material kind values in schema (cargo, go, ruby) | `build.rs`, schema | Low |
| Transparency log integration | Roadmap (VBW-2) | Future |
| Multi-builder consensus (N-of-M signatures) | Roadmap (VBW-2) | Future |
| OIDC identity binding | Roadmap (VBW-2) | Future |
| SBOM integration | Roadmap (VBW-2) | Future |

### Completed in this version

| Done | Where | Notes |
|------|-------|-------|
| Canonical JSON signing (RFC 8785-equivalent) | `canonical.rs` | Signature covers sorted-key, compact JSON bytes |
| Strict fail-closed verify | `verify.rs` | Missing files, extra files, symlink escapes all rejected |
| Enforcement honesty (mode_enforced flag) | `model.rs`, `build.rs` | Manifest records what was actually enforced |
| Interleaved stdout/stderr transcript capture | `build.rs` | Timestamped, threaded, arrival-order |
| Streaming SHA-256 for large files | `hash.rs` | 64 KiB buffered reads, constant memory |
| Path traversal rejection | `verify.rs` | Rejects `..` in artifact paths, absolute paths, escaping symlinks |

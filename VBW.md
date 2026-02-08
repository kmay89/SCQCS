# Verified Build Witness (VBW) v1.0

**Cryptographic proof that your build is what it claims to be.**

VBW creates tamper-evident records proving that specific source code, built with a specific toolchain, in a specific environment, produced specific artifacts. Every build ships with a signed witness bundle that anyone can independently verify.

---

## Why VBW Exists

When you download software, you trust that the binary matches the source code. But how do you *prove* it? VBW answers six questions about every build:

| Question | VBW Answer |
|----------|------------|
| What exact source was built? | Git commit hash + canonical source tree hash |
| What tools compiled it? | Compiler/runtime versions captured in `environment.json` |
| What OS/container ran the build? | OS, kernel, architecture, container digest |
| Can this build be reproduced? | Reproducibility mode (deterministic / locked / witnessed) |
| Has the output been tampered with? | SHA-256 hashes of every artifact in `outputs.json` |
| Who attested to all of this? | Ed25519 signature over the manifest |

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
  transcript.txt               # Full build log (stdout/stderr)
  policy.json                  # Build policy requirements
  signatures/
    builder.ed25519.sig        # Ed25519 signature over manifest.json
  hashes/
    manifest.sha256            # SHA-256 of manifest.json
```

### manifest.json

The core document. It doesn't contain data directly — it contains *hashes* of all other files, creating a single signed root of trust.

```json
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
  "source_commit_tree_hash": "e3b0c44298fc1c14...",
  "materials_lock_hash": "d7a8fbb307d7809469...",
  "environment_hash": "9f86d081884c7d659a...",
  "outputs_hash": "2c26b46b68ffc68ff9...",
  "builder_identity": {
    "key_id": "builder@ci",
    "public_key_ed25519": "Base64EncodedPublicKey..."
  },
  "policy_ref": {
    "path": "vbw/policy.json",
    "hash_sha256": "hash-of-policy-file..."
  }
}
```

### environment.json

Captures everything about where the build ran:

```json
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

Every artifact is hashed and measured:

```json
{
  "artifacts": [
    {
      "path": "dist/index.html",
      "sha256": "e3b0c44298fc1c149afbf4c8...",
      "size_bytes": 15234,
      "mime": "text/html"
    },
    {
      "path": "dist/main.js",
      "sha256": "d7a8fbb307d7809469ca9abcb...",
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
5. Validates policy compliance

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

---

## Reproducibility Modes

VBW supports three levels of build reproducibility:

### Mode A: Deterministic

The strictest mode. The exact same inputs must produce the exact same outputs, byte-for-byte.

- No network access during build
- Pinned toolchain versions
- `SOURCE_DATE_EPOCH` set to normalize timestamps
- If you rebuild from the same commit, you get identical hashes

### Mode B: Locked Network (Default)

A practical middle ground. Network access is allowed, but only for fetching locked, hashed dependencies.

- Dependencies must come from lockfiles with recorded hashes
- Toolchain versions are captured but not pinned
- Good for projects using `npm ci`, `cargo build`, or similar

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

If your site is already built (pure HTML/CSS/JS), use a copy step:

```bash
mkdir -p dist && cp -r *.html *.css *.js dist/
scqcs vbw build --output-dir dist -- echo "Static site copied"
```

### Custom Output Directory

The `--output-dir` flag tells VBW where to find artifacts to hash:

```bash
scqcs vbw build --output-dir build/release -- make release
```

---

## Policy Configuration

The policy file controls what the build is *required* to do. VBW auto-generates a default if none exists.

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

For maximum reproducibility:

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
  |-- hashes/manifest.sha256      (does the stored hash match the file?)
  |-- signatures/builder.ed25519  (does the signature match the public key?)
  |
  |-- environment_hash            (recompute hash of environment.json, compare)
  |-- materials_lock_hash         (recompute hash of materials.lock.json, compare)
  |-- outputs_hash                (recompute hash of outputs.json, compare)
  |-- policy_ref.hash_sha256      (recompute hash of policy.json, compare)
  |
  outputs.json
    |-- artifact[0].sha256        (does dist/index.html still match?)
    |-- artifact[1].sha256        (does dist/main.js still match?)
    ...
```

If any hash doesn't match, the verdict is **UNVERIFIED**. This catches:
- Modified artifacts (someone changed a file after the build)
- Modified metadata (someone altered the environment or materials record)
- Forged signatures (someone tried to re-sign with a different key)

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
      build.rs                  # Build workflow (14-step pipeline)
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

---

## Security Model

### What VBW Proves

- The build artifacts were produced by the claimed source commit
- The builder possessed the signing key at build time
- The environment and dependencies match what was recorded
- No file in the bundle has been modified since signing

### What VBW Does Not Prove

- That the source code is free of vulnerabilities
- That the signing key hasn't been compromised
- That the build environment wasn't itself compromised
- That the build is reproducible (unless using Mode A)

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

VBW targets Unix/Linux environments and CI runners (GitHub Actions, Docker). Environment detection uses `uname` for OS information and `which` for tool paths. On non-Unix systems, OS fields will report "unknown" but the core signing and verification workflow functions correctly on any platform.

---

## Roadmap (VBW-2)

Future enhancements planned for VBW v2:

- Transparency log integration (append-only public ledger of witness bundles)
- Vendor tarball support (hash both archive and extracted tree)
- Multi-builder consensus (require N-of-M builder signatures)
- OIDC identity binding (tie builder keys to CI identity tokens)
- SBOM integration (link witness bundles to software bill of materials)

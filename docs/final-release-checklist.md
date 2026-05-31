# Final Release Checklist

This checklist is the final release readiness gate for the pre-1.0
launch-readiness line.

Run it from a clean checkout before tagging a release:

```bash
scripts/check-final-release.sh
```

## Full End-To-End Workflow Validation

The final release gate runs `scripts/check.sh`, which covers formatting, shell
and Python syntax, benchmark report regeneration, release workflow invariants,
Rust checks, the `wasm32-unknown-unknown` core build, tests, install smoke, and
clippy.

It also runs the researcher-facing workflow through `scripts/launch-demo.sh`
against the release binary.

## Public Contract Freeze

The public contract candidates remain:

- CLI command list and output policy in `docs/cli-contract.md`
- error codes in `docs/error-codes.md`
- schemas in `schemas/`
- stabilization candidates in `docs/public-contract-1.0-candidates.md`

Before tagging, check whether any CLI flag, JSON field, schema, error code, or
package manifest field changed without a matching test and doc update.

## Dependency Policy

The final release gate runs `scripts/check-dependency-policy.py` through
`scripts/check.sh`. Before tagging, review any `Cargo.toml` or `Cargo.lock`
change against [docs/dependency-policy.md](dependency-policy.md), including:

- whether `biors-core` still has only its approved normal dependencies
- whether optional Candle, MCP, Python, and WASM dependencies remain isolated in
  their package-specific crates
- whether `cargo tree --locked -p biors-core --duplicates` and
  `cargo tree --locked -p biors --duplicates` remain clean
- why any new dependency is needed and whether it affects the default CLI/core
  install path

## Breaking Change Cleanup

The current blocker queue is the local pre-release audit ledger at
`docs/pre-release-audit-main-2026-05-30.md`. That file is intentionally not
published with the repository, but release prep must treat every open item in it
as unresolved until it is either fixed, explicitly deferred in a public doc, or
removed because code-level inspection proved it obsolete.

If a breaking cleanup is discovered, do not hide it in release prep. Land it as
a focused implementation commit with tests, update the contract docs, then rerun
the final release gate.

## Benchmark Artifact Coverage

The final release gate runs `scripts/check-benchmark-docs.sh` through
`scripts/check.sh`. Before tagging, confirm committed benchmark artifacts cover
the features promoted in README, docs, release notes, and package metadata.

Do not make performance claims for a feature unless a committed benchmark JSON
artifact and regenerated markdown report cover that surface. If a promoted
feature has only smoke coverage or no numeric artifact, document it as a
non-claim.

## Release Artifact Contents

Tagged release jobs must inspect package contents before upload or publish:

- Python wheels must include `LICENSE-APACHE` and `LICENSE-MIT`.
- The Python sdist must include `LICENSE-APACHE` and `LICENSE-MIT`.
- The npm WASM package dry-run must include `LICENSE-APACHE` and `LICENSE-MIT`.
- Binary tarballs must include `biors`, `README.md`, `LICENSE-APACHE`, and
  `LICENSE-MIT`.

The release workflow enforces these checks with
`scripts/check-release-artifact-contents.py`.

## Registry Preflight

The tag workflow runs `scripts/check-registry-versions.py` before publishing
crates, Python distributions, or the npm package. Before tagging, verify the
target version is unpublished on every registry and that the workflow still runs
the registry check before any publish job.

## Version Tag

Use annotated tags that point at the release-prep commit:

```bash
git tag -a vX.Y.Z -m "Release vX.Y.Z"
git push --no-verify origin vX.Y.Z
```

The tag push triggers `.github/workflows/release.yml`.

## Binary Release Test

The final release gate builds the release binary locally:

```bash
cargo build --locked --release -p biors
BIORS_BIN=target/release/biors sh scripts/launch-demo.sh
```

Tagged releases also build and attach:

- `biors-vX.Y.Z-x86_64-unknown-linux-gnu.tar.gz`
- `biors-vX.Y.Z-aarch64-apple-darwin.tar.gz`

## Install Flow Final Test

The final release gate runs:

```bash
scripts/check-install-smoke.sh
```

That installs `biors` from the local package path with `--locked`, verifies the
reported version, and runs a tokenization command from the installed binary.

## GitHub Release Dry Run

GitHub release creation is side-effectful, so the local dry run verifies the
release workflow instead of creating a draft release:

```bash
python3 scripts/check-release-workflow.py
```

This checks crates publish order, release creation order, and binary artifact
packaging configuration.

## npm Trusted Publishing

The WASM package release job uses npm trusted publishing. Before tagging a
release that changes `packages/rust/biors-wasm`, verify the npm package
settings for `@bio-rs/biors-wasm` allow GitHub Actions publishing from:

- GitHub organization/user: `bio-rs`
- GitHub repository: `bio-rs`
- Workflow filename: `release.yml`
- Allowed action: `npm publish`

## Public Demo Dry Run

Run the installed or release binary through:

```bash
sh scripts/launch-demo.sh
sh scripts/record-cli-demo.sh
```

For source checkouts before install:

```bash
sh scripts/launch-demo.sh --cargo
sh scripts/record-cli-demo.sh --cargo
```

The browser playground remains intentionally deferred; no browser release gate is
required for this phase.

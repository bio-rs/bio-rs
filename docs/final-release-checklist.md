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

## Breaking Change Cleanup

No known breaking cleanup is deferred for the current pre-1.0 contract set.

If a breaking cleanup is discovered, do not hide it in release prep. Land it as
a focused implementation commit with tests, update the contract docs, then rerun
the final release gate.

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

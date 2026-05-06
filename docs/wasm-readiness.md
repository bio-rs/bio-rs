# WASM Readiness

bio-rs keeps the core crate usable without CLI file-system assumptions.

## Separation Audit

- `biors-core` exposes string, byte, and buffered-reader APIs for FASTA,
  tokenization, workflow, package contracts, and verification helpers.
- Local file I/O lives in the `biors` CLI input layer or in package artifact
  helpers that explicitly take a base directory.
- `scripts/check.sh` builds `biors-core` for `wasm32-unknown-unknown` so
  accidental platform-specific dependencies are caught.

## Panic-Free API Review

Public sequence validation and tokenization APIs return structured validation
reports for invalid direct byte input. They do not assume that a caller-created
`ProteinSequence` contains valid UTF-8.

The CLI still rejects invalid UTF-8 FASTA input as `io.read_failed` because
FASTA reader paths are UTF-8 text contracts.

## Optional Proof Of Concept

The current proof is compile-level: `cargo check -p biors-core --target
wasm32-unknown-unknown --all-features`. A runtime WASM package is intentionally
deferred until the core input and package contracts settle further.

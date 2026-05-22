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

## WASM API Design

The browser-safe WASM API surface is documented in [docs/wasm-api.md](wasm-api.md).
It covers:

- `npm install @bio-rs/core-wasm`
- Initialization and module loading
- FASTA parsing, validation, tokenization, model input, workflow
- Package manifest validation and runtime bridge planning
- TypeScript interfaces for all exported types
- Browser limitations (no filesystem, no external process)

The actual `@bio-rs/core-wasm` npm package is planned for v0.45.0.
The v0.43.0 release documents the intended API surface as a design contract.

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

- `npm install @bio-rs/biors-wasm`
- Initialization and module loading
- FASTA parsing, validation, tokenization, model input, workflow
- TypeScript interfaces for all exported types
- Browser limitations (no filesystem, no external process)

The `biors-wasm` package source is implemented in-repo with TypeScript
definitions. Tag releases build the package with `wasm-pack`, run WASM tests,
and publish it through npm trusted publishing.

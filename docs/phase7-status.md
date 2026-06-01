# Phase 7 Runtime And Interface Status

Phase 7 is implemented through the `0.47.x` line as the runtime and external
interface layer for bio-rs, but release-readiness is tracked separately from
implementation status.

Excluded work for this phase means only:

- landing pages and browser playground UI
- the future `1.0.0` launch event and stable-contract launch polish

WASM/JavaScript bindings, Python bindings, MCP tooling, service contracts,
runtime planning, and backend packaging are in scope.

## Shipped Surface

| Version | Slice | Current status | Evidence |
|---|---|---|---|
| `0.38.0` | Execution abstraction | Implemented | `biors_core::runtime::{Backend, BackendConfig, BackendCapabilities, ExecutionContext, ExecutionResult}` and `docs/backend-architecture.md` |
| `0.39.0` | External process backend | Implemented | `ExternalProcessBackend`, timeout/stdout/stderr limits, process I/O tests, and security notes in `docs/backend-architecture.md` |
| `0.40.0` | Optional Candle backend | Implemented | `biors-backend-candle`, linear-probe tests/bench, and `docs/candle-backend.md` |
| `0.41.0` | Model artifact metadata and compatibility | Implemented | package manifest metadata, artifact hashes, runtime compatibility checks, and package bridge reports |
| `0.42.0` | Backend compatibility matrix | Implemented | package compatibility matrix and reproducibility/report linkage in package bridge outputs |
| `0.43.0` | External interface API review | Implemented, needs contract hardening before 1.0 | `docs/rust-api.md`, `docs/python-api.md`, and `docs/wasm-api.md` |
| `0.44.0` | Python binding candidate | Implemented, release workflow configured, needs contract hardening | `packages/rust/biors-python`, Python API tests, and `docs/python-api.md` |
| `0.45.0` | WASM/JS API candidate | Implemented, release workflow configured, needs contract hardening | `packages/rust/biors-wasm`, `index.d.ts`, and `docs/wasm-api.md` |
| `0.46.0` | Agent-callable tool interface | Implemented, needs contract hardening | `biors-mcp-server` crate, MCP tests, and crates.io package |
| `0.47.0` | Service interface design | Implemented as offline contract | `biors service contract`, `biors_core::service`, and `schemas/service-interface-output.v0.json` |

## Release-Readiness Boundaries

The shipped Phase 7 surface is intended for real preprocessing and package
integration work before model inference, but this document does not claim every
binding and integration surface is fully researcher-grade. The active
pre-release audit ledger is `docs/pre-release-audit-main-2026-05-30.md`; that
file is local release-prep state and is not part of the published project docs.

Current implemented guarantees:

- deterministic JSON contracts for CLI, package, runtime, and service surfaces
- stable FASTA parsing, validation, tokenization, model-input construction, and
  reproducibility provenance
- package checksum, fixture, metadata, runtime bridge, compatibility, and diff
  checks for local model artifacts
- native Rust and CLI release verification through `scripts/check.sh`
- crates.io publication for `biors`, `biors-core`, `biors-backend-candle`, and
  `biors-mcp-server`

External binding surfaces are covered by this repository's tag release workflow
and by the support matrix in `docs/sequence-kind-support.md`:

- `biors-python` PyO3 wheels/source distribution on PyPI
- `@bio-rs/biors-wasm` package with TypeScript definitions on npm; future tag
  releases publish it from GitHub Actions through npm trusted publishing
- `biors-mcp-server` package and JSON tool contracts

Do not describe the entire project as fully researcher-grade without also
naming the remaining protein-first package skeleton/conversion limitations in
the support matrix.

## Performance Status

The committed FASTA benchmark is a historical public baseline, not current
`0.47.5` throughput evidence. Current benchmark artifacts cover fixed-length
model-input construction, binding/MCP/package smoke surfaces, and nucleotide
workflow regression guards, but no new public throughput claim is made until
the FASTA benchmark artifact is regenerated for the current release candidate.

## Deferred

The following remain outside this Phase 7 delivery:

- landing page or browser playground UI
- hosted web workflow
- `1.0.0` stable launch process
- pretrained model-specific inference backend coverage beyond the current
  optional Candle linear-probe backend and runtime planning contracts
- chemistry, structure, and no-code workflow surfaces

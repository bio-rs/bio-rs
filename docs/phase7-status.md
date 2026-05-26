# Phase 7 Runtime And Interface Status

Phase 7 is implemented through the `0.47.x` line as the runtime and external
interface layer for bio-rs.

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
| `0.43.0` | External interface API review | Implemented | `docs/rust-api.md`, `docs/python-api.md`, and `docs/wasm-api.md` |
| `0.44.0` | Python binding candidate | Implemented as local PyO3 crate | `packages/rust/biors-python`, Python API tests, and `docs/python-api.md` |
| `0.45.0` | WASM/JS API candidate | Implemented as WASM crate/package source | `packages/rust/biors-wasm`, `index.d.ts`, and `docs/wasm-api.md` |
| `0.46.0` | Agent-callable tool interface | Implemented | `biors-mcp-server` crate, MCP tests, and crates.io package |
| `0.47.0` | Service interface design | Implemented as offline contract | `biors service contract`, `biors_core::service`, and `schemas/service-interface-output.v0.json` |

## Researcher-Grade Criteria

The shipped Phase 7 surface is intended for real preprocessing and package
integration work before model inference. Current guarantees:

- deterministic JSON contracts for CLI, package, runtime, service, and MCP
  surfaces
- stable FASTA parsing, validation, tokenization, model-input construction, and
  reproducibility provenance
- package checksum, fixture, metadata, runtime bridge, compatibility, and diff
  checks for local model artifacts
- native Rust and CLI release verification through `scripts/check.sh`
- crates.io publication for `biors`, `biors-core`, `biors-backend-candle`, and
  `biors-mcp-server`

Candidate surfaces that are implemented in-repo but not yet independently
published by this repository's release workflow:

- `biors-python` PyO3 package source and tests
- `@bio-rs/biors-wasm` package source and TypeScript definitions

Those bindings are suitable for local integration testing and downstream package
work, but their registry publication should use dedicated PyPI/npm release
workflows before they are described as independently published artifacts.

## Performance Status

The committed FASTA benchmark is the latest recorded public baseline. It should
remain the source for numeric throughput claims until rerun and committed with
new environment metadata. The `0.47.1` patch reduces unnecessary allocation in
valid protein validation and fixed-length model-input construction; no new
throughput claim is made until the benchmark artifact is regenerated.

## Deferred

The following remain outside this Phase 7 delivery:

- landing page or browser playground UI
- hosted web workflow
- `1.0.0` stable launch process
- pretrained model-specific inference backend coverage beyond the current
  optional Candle linear-probe backend and runtime planning contracts
- chemistry, structure, and no-code workflow surfaces

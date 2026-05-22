# Phase 7 Runtime Compatibility 0.42 Design

## Scope

`0.42.0` completes the backend compatibility slice that follows the released
runtime contracts, external process backend, Candle adapter, and package bridge
checks in `0.38.0` through `0.41.0`.

This release must:

- finish the in-progress package bridge `BackendConfig` exposure
- define the supported backend compatibility matrix from implemented package
  and backend contracts
- connect package compatibility decisions to reproducibility and benchmark
  reporting without making unsupported performance claims
- keep the default CLI free of optional runtime execution dependencies

This release does not add Python bindings, WASM/JS packaging, service mode,
agent-callable tool manifests, or browser launch surfaces.

## Architecture

`biors-core::runtime` remains the source of backend execution contracts.
`biors-core::package` remains the source of package manifest and package bridge
planning contracts.

The package bridge report becomes the deterministic join point between:

- manifest runtime declarations
- model artifact format and metadata
- a stable `BackendConfig` identity usable by downstream execution layers
- backend compatibility checks and blocking issues

The compatibility matrix is derived from implemented backend and package
contracts instead of a disconnected roadmap table. Matrix entries must name the
model format, runtime backend, target, execution provider, input/output contract
status, and reproducibility evidence available today.

## Data Flow

1. A package manifest declares model artifact, runtime backend, target, and
   runtime version.
2. `plan_runtime_bridge` validates the manifest and checks the backend/model
   pair.
3. The report returns a `BackendConfig` with a stable backend id, provider,
   version, and model artifact reference.
4. CLI package bridge output serializes the same report contract and schema.
5. Documentation and report artifacts use the same compatibility mapping for
   researchers deciding whether a package can be routed to an implemented
   backend.

## Error Handling

Compatibility remains preflight behavior. Unsupported backend/model/target
pairs become deterministic compatibility failures and blocking bridge issues.
This release does not silently execute optional backends, download artifacts, or
infer unsupported runtime capabilities.

## Reproducibility And Benchmarks

The matrix must distinguish:

- checksum and fixture evidence already carried by package manifests
- deterministic backend declarations from runtime capabilities
- benchmark evidence that exists for a backend-specific workload

Benchmark links and baselines are metadata about reproducible evidence. They are
not global claims about every biological model or input distribution.

## Testing

The release is test-first:

- retain the currently failing bridge report tests as the red baseline
- add focused core and CLI assertions for backend configuration and matrix
  report output
- update JSON schema contract tests when CLI output changes
- run the repository check gate before release preparation

## Release Shape

`0.42.0` stays one release slice:

1. feature commit for compatibility report, matrix, tests, schemas, and docs
2. release-prep commit updating versioned metadata
3. merge to `main`
4. tag and crates.io/GitHub release workflow verification

Follow-up versions own the later external interface review and binding/service
surfaces.

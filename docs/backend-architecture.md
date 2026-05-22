# Backend Architecture

Phase 7 starts with an execution abstraction in `biors-core::runtime`.

The `0.38.0` scope is intentionally narrow:

- define the `Backend` trait
- define `BackendConfig`
- define `BackendCapabilities`
- define `ExecutionContext`
- define `ExecutionResult`
- define `BackendExecutionError`
- define preflight compatibility reporting before execution
- keep concrete backend implementations out of the default build

This version does not run model inference. It creates the contract that future
backends must implement.

## Runtime Contracts

`Backend` owns three responsibilities:

- expose stable backend identity through `BackendConfig`
- expose supported input/output behavior through `BackendCapabilities`
- execute one `ExecutionContext` into one `ExecutionResult`

`ExecutionContext.payload` and `ExecutionResult.payload` are byte vectors
because each backend owns its wire format. The surrounding fields carry the
stable bio-rs contract:

- `trace_id`: optional caller-provided correlation ID
- `input_format`: stable input payload format
- `requested_output_format`: optional desired output payload format
- `output_format`: actual result payload format
- `metadata`: small string key/value metadata safe to log or echo in reports

`BackendCapabilities` is deliberately declarative. It can be used before
execution for compatibility checks without loading a model or invoking an
external process.

`BackendCapabilities::compatibility_report` reports every known blocker before
execution:

- unsupported input payload format
- unsupported requested output payload format
- payload larger than the backend-declared byte limit

`Backend::execute_checked` uses the same compatibility check and returns a
stable `BackendExecutionError` before backend-specific execution starts. Backend
authors should expose the checked path to callers unless they have already
performed equivalent validation at a higher layer.

## Crate Split Review

`crates/biors-runtime` is not introduced in `0.38.0`.

Rationale:

- the abstraction is still small and coupled to `biors-core` model-input and
  package compatibility work
- there is no independently published runtime implementation yet
- adding a crate now would add release coordination without reducing dependency
  weight for users

The code is isolated under `packages/rust/biors-core/src/runtime.rs` so a future
split can move the contracts without dragging package, CLI, or benchmark code
into a backend crate.

Revisit the split when at least one concrete backend exists outside
`biors-core`, such as an external process backend or an optional Candle backend.

## Backend Boundaries

Concrete backend work belongs in later versions:

- `0.39.0`: external process invocation, stdout/stderr/result parsing, timeout
  and sandbox policy
- `0.40.0`: optional Candle integration outside the default build
- `0.41.0`: model artifact metadata and backend compatibility checks

Until those versions land, package runtime bridge reports remain planning and
compatibility surfaces, not proof that inference was executed.

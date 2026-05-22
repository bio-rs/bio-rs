# Backend Architecture

Phase 7 starts with execution contracts and guarded local backend integration in
`biors-core::runtime`.

The `0.38.0` scope introduced the core contracts:

- define the `Backend` trait
- define `BackendConfig`
- define `BackendCapabilities`
- define `ExecutionContext`
- define `ExecutionResult`
- define `BackendExecutionError`
- define preflight compatibility reporting before execution
- keep concrete backend implementations out of the default build

The `0.39.0` scope adds a guarded external-process backend for adapters that
already exist as local tools, scripts, or research binaries:

- `ExternalProcessBackend`
- `ExternalProcessConfig`
- direct process execution without shell interpolation
- JSON stdin/stdout protocol for `ExecutionContext` and `ExecutionResult`
- timeout enforcement
- bounded stdout and stderr capture
- non-zero exit, invalid output, and process I/O error codes
- process timing and byte-count metadata on successful results

This still does not add a built-in model inference engine. The external process
backend is a controlled adapter boundary for researchers who already have a
local executable that can consume bio-rs model-input JSON and return a stable
bio-rs execution result.

The `0.40.0` scope adds an optional Candle backend crate:

- `biors-backend-candle`
- CPU safetensors loading through Candle
- a deterministic linear-probe adapter for `ModelInput` JSON
- a Candle-specific Criterion benchmark
- no Candle dependency in `biors-core`
- no backend-enabled CLI binary artifact by default

The `0.41.0` scope connects package manifests to the runtime planning layer:

- optional model artifact metadata on package `model`
- manifest support for ONNX/WebGPU and safetensors/Candle CPU compatibility
  pairs
- structured `package bridge` compatibility checks
- explicit blocking issues when a model format, backend, and target do not
  match
- no runtime launch command or backend-enabled CLI binary artifact

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

## External Process Backend

`ExternalProcessBackend` runs one configured child process per execution. The
child receives a serialized `ExecutionContext` on stdin and must write a
serialized `ExecutionResult` to stdout.

The backend intentionally avoids shell execution. `ExternalProcessConfig.program`
and `ExternalProcessConfig.args` are passed directly to the operating system, so
paths and arguments are not interpreted through shell quoting, expansion, pipes,
or redirection.

`ExternalProcessConfig` defaults are conservative:

- parent environment is not inherited
- only explicitly listed environment variables are passed to the child
- one execution has a wall-clock timeout
- stdout and stderr are drained with byte limits
- stderr content is not copied into process-exit error messages

The invocation contract is intentionally small:

- stdin: one JSON `ExecutionContext`
- stdout: one JSON `ExecutionResult`
- stderr: unstructured diagnostic output, counted but not copied into stable
  errors
- exit status: zero means stdout is parsed; non-zero maps to
  `runtime.process_exit_failed`

If a child exits before reading stdin, the backend reports the process exit
failure instead of masking it behind a broken-pipe write. Stream limit failures
are reported before JSON parsing or process-exit details so oversized output is
classified as a resource-policy failure.

## Observability Draft

Successful results receive wrapper metadata:

- `external_process.elapsed_millis`
- `external_process.stdout_bytes`
- `external_process.stderr_bytes`
- `external_process.exit_status`

These values are intended for trace correlation and operational diagnostics.
They are not benchmark claims.

The caller-supplied `ExecutionContext.trace_id` is propagated through successful
results when the child omits a trace identifier. Failed executions return stable
`BackendExecutionError` codes that include the backend id for log correlation.

## Security Boundary

The external process backend is a process-control boundary, not a full sandbox.
It prevents shell interpolation, constrains inherited environment by default,
enforces timeout, and bounds captured output.

Current resource policy:

- wall-clock timeout is enforced per execution
- stdout and stderr bytes are bounded and drained before classification
- CPU, memory, GPU, network, and file descriptor limits are not enforced by
  bio-rs

Current environment and file-access policy:

- parent environment is not inherited unless explicitly configured
- child environment variables must be explicitly listed when inheritance is off
- `current_dir` only selects the child working directory; it is not a file
  sandbox
- bio-rs does not implement seccomp, container isolation, chroot, network
  filtering, filesystem allowlists, or GPU policy

Run untrusted adapters inside an OS sandbox, container, virtual machine, or
cluster policy before exposing them to user-supplied biological data. Treat
stdout and stderr from external tools as untrusted process output.

## Crate Split Review

`crates/biors-runtime` is still not introduced in `0.40.0`.

Rationale:

- the runtime surface is still coupled to `biors-core` model-input and package
  compatibility work
- the dependency-heavy Candle integration is isolated in `biors-backend-candle`
- adding a crate now would add release coordination without reducing dependency
  weight for users

The code is isolated under `packages/rust/biors-core/src/runtime/` so a future
split can move the contracts and adapters without dragging package, CLI, or
benchmark code into a backend crate.

Revisit the split when at least one concrete backend needs to live outside
`biors-core`, such as an optional Candle backend or a provider-specific adapter
with heavier dependencies.

## Backend Boundaries

Concrete backend work belongs in later versions:

- later phases may add backend-specific CLI wiring once package
  compatibility and artifact contracts are stable

Package runtime bridge reports remain planning and compatibility surfaces.
Candle execution is available only through the optional backend crate and is not
wired into the default CLI binary.

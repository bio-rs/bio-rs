# Candle Backend

`biors-backend-candle` is an optional runtime backend crate for local Candle
inference experiments. It is separate from `biors-core` so the default parser,
tokenizer, package, and CLI build stays dependency-light.

The first backend adapter is a deterministic CPU linear-probe model:

- load `safetensors` weights with Candle
- read bio-rs `ModelInput` JSON through `ExecutionContext`
- embed unmasked token IDs
- mean-pool token embeddings with `attention_mask`
- apply a projection weight and optional bias
- return JSON scores through `ExecutionResult`

This is intended for small local probes, smoke tests, and research pipelines
that need a Rust-native inference boundary before heavier model families are
added. It is not a pretrained ESM, ProtBERT, ONNX, WebGPU, or hosted inference
runtime.

## Weight Contract

`CandleBackend::from_safetensors` expects a `CandleBackendConfig` naming these
tensors:

- `embedding_tensor`: rank-2 floating tensor shaped `[vocab_size, hidden_dim]`
- `projection_weight_tensor`: rank-2 floating tensor shaped
  `[hidden_dim, output_dim]`
- `projection_bias_tensor`: optional rank-1 floating tensor shaped
  `[output_dim]`

Token IDs in each `ModelInputRecord.input_ids` must be less than
`vocab_size`. Tokens with attention-mask value `0` are ignored before pooling.
Records with no unmasked tokens fail with a stable runtime execution error.

## Runtime Contract

The backend uses the Phase 7 runtime interfaces from `biors-core::runtime`:

- input format: `biors.model-input.v0+json`
- output format: `biors.candle.linear-probe.v0+json`
- backend provider: `candle`
- deterministic: `true`
- streaming: `false`

Successful results include metadata:

- `candle.device`
- `candle.elapsed_millis`
- `candle.output_records`

Errors from payload parsing, tensor loading, shape validation, token ID checks,
and Candle execution are mapped into `runtime.execution_failed` with the
configured backend id.

## Device And Feature Policy

The `0.40.0` backend supports CPU execution. CUDA, Metal, Accelerate, MKL, and
other device-specific Candle features remain out of the default workspace build.
Those should be added behind explicit crate features only after platform CI and
artifact policy are defined.

## Core And CLI Separation

`biors-core` does not depend on Candle. The `biors` CLI does not link this
backend by default and no backend-enabled binary artifact is produced in
`0.40.0`.

Rationale:

- keep the default CLI install small and portable
- avoid requiring researchers to compile platform-specific inference backends
  for preprocessing-only workflows
- preserve the runtime trait boundary for Python, WASM, service, and external
  process adapters

## ONNX / WebGPU Review

Existing package manifests still describe `onnx-webgpu` as a planning target.
The Candle crate does not reinterpret those manifests and does not add a browser
runtime bridge. Backend compatibility mapping between Candle, ONNX, WebGPU, and
future service runtimes belongs in the compatibility matrix phase.

## Benchmark

The crate includes a Candle-specific Criterion benchmark:

```bash
cargo bench -p biors-backend-candle --bench candle_linear_probe
```

The benchmark runs a generated CPU linear probe over 32 model-input records with
128 tokens each. It is a backend smoke benchmark, not a claim about pretrained
protein language model performance.

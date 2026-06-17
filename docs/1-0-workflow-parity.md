# 1.0 Workflow Parity

This matrix records how the local bio-AI 1.0 workflows behave across exposed
surfaces. Parity means the same success/failure semantics, stable codes, and
JSON contracts where a surface exposes the workflow. A missing capability is
recorded as not exposed instead of treated as a test failure.

| Workflow | CLI | Rust core | Python | WASM | MCP | service |
| --- | --- | --- | --- | --- | --- | --- |
| protein validate/tokenize/model-input/workflow | Exposed through `fasta validate`, `seq validate`, `tokenize`, `model-input`, and `workflow`; parity test compares CLI workflow output with Rust core. | Source of truth through sequence, tokenizer, model-input, and workflow modules. | Exposed through `validate_fasta_input_with_kind`, `tokenize_fasta_records`, `build_model_inputs_checked`, and `prepare_workflow_from_fasta`; Python tests cover schema parity and stable errors. | Exposed through `validateFasta`, `tokenize`, `buildModelInput*`, and `runWorkflow`; WASM tests cover shared workflow schema parity. | Exposed through `validate`, `tokenize`, and `workflow`; MCP parity test compares workflow output with Rust core. | not exposed; service only exposes batch sequence validation, not tokenization or model-input generation. |
| invalid sequence | Exposed through JSON errors and `workflow` `model_ready=false` readiness issues; parity test compares CLI and Rust core readiness code. | Source of truth for validation issues and `sequence.not_model_ready`. | Exposed through Python validation, tokenization diagnostics, and model-input errors. | Exposed through WASM validation/workflow errors and readiness issues. | Exposed through MCP invalid params for invalid FASTA and workflow readiness issues for non-model-ready records. | Exposed for batch validation only; tokenization/model-input recovery is unsupported on this surface. |
| package validate/bridge | Exposed through `package validate` and `package bridge`; parity test compares CLI bridge fields with Rust core. | Source of truth through package validation and runtime bridge planning. | Exposed through package JSON helper functions and artifact validation helpers. | not exposed; package manifest validation and bridge planning are unsupported on this surface. | `package_validate` and `package_validate_fields` are exposed; bridge planning is not exposed. | not exposed; package validation and runtime bridge planning are unsupported on this surface. |
| service batch validation | not exposed as CLI workflow parity; CLI only starts the local HTTP server and reports service contracts. | Source of truth through service batch validation request handling and sequence validation semantics. | not exposed. | not exposed. | not exposed. | Exposed through `POST /v0/batch/sequence/validate`; service tests compare route behavior with checked schemas and core validation semantics. |

## Unsupported Gaps

- WASM package validation and runtime bridge planning are not exposed because
  browser/JS embedding is currently scoped to local validation and workflow
  APIs.
- MCP package bridge planning is not exposed; agents should use CLI or Python
  package bridge helpers when runtime bridge planning is needed.
- The local HTTP service intentionally exposes only health, OpenAPI, and batch
  sequence validation. Tokenization, model-input generation, package checks, and
  runtime bridge planning are unsupported on this surface.
- Python and WASM are integration surfaces, not the primary product narrative;
  when a parity path is intentionally different, the CLI/MCP workflow remains
  the 1.0 researcher/agent path.

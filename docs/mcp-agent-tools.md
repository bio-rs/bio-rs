# MCP Agent Tools

bio-rs exposes local MCP tools as the agent-callable surface of the local
bio-AI tool layer for research agents that need deterministic biological
validation, tokenization, workflow, and package checks. This local tool layer
does not provide autonomous research planning. It does not provide hosted
execution. It does not provide literature review, long-term memory, or remote
lab automation.

## Output Policy

MCP tools return machine-readable JSON text. For long biological FASTA inputs,
sequence tools protect agent context by returning summary/counts/issues by
default using `schema_version: "biors.mcp.compact.v0"`. Request full per-record
payloads only when the caller actually needs them:

The compact-output contract is summary/counts/issues by default.

- `include_records: true` for `tokenize` and `validate`.
- `include_payload: true` for `workflow`.

Package tools already return validation summaries and structured issues rather
than biological payloads. All tools run locally and do not upload data,
do not upload telemetry, and do not upload model inputs.

## Tool Catalog

| Tool | Purpose | Input schema | Output schema | Default payload behavior | Failure codes | Recommended sequence |
| --- | --- | --- | --- | --- | --- | --- |
| `tokenize` | Tokenize FASTA text with a protein, DNA, or RNA tokenizer profile. | `fasta_text`, optional `profile`, optional `include_records`. | Full: tokenized record array. Long input default: `biors.mcp.compact.v0` with tokenization summary and issue counts. | Compact for long FASTA unless `include_records=true`. | FASTA diagnostics such as `fasta.missing_header`; invalid profile is MCP invalid params. | After `validate` when token IDs are needed. |
| `validate` | Validate FASTA text as `auto`, `protein`, `dna`, or `rna`. | `fasta_text`, optional `kind`, optional `include_records`. | Full: kind-aware validation report. Long input default: `biors.mcp.compact.v0` with counts, kind counts, and issue counts. | Compact for long FASTA unless `include_records=true`. | FASTA diagnostics such as `fasta.empty_input` and `fasta.missing_header`; invalid kind is MCP invalid params. | First step for sequence input triage. |
| `workflow` | Run validation -> tokenization -> model-input preparation. | `fasta_text`, optional `kind`, optional `profile`, `max_length`, `pad_token_id`, `padding`, optional `include_payload`. | Full: `sequence-workflow-output.v0`. Long input default: `biors.mcp.compact.v0` with validation/tokenization summaries and readiness issues. | Compact for long FASTA unless `include_payload=true`. | `sequence.not_model_ready` in readiness issues; invalid policy, bad padding, kind/profile mismatch, or FASTA parse errors are MCP invalid params. | Use after `validate` when model-ready local records are required. |
| `package_validate_fields` | Validate package manifest JSON fields without filesystem artifact checks. | `manifest_json`. | `package-validation-report.v0`. | Field-only report with structured issues. | Manifest parse/contract errors as MCP invalid params or `structured_issues`. | Use before filesystem checks when an agent only has manifest JSON. |
| `package_validate` | Validate a package manifest and local artifacts. | `manifest_path`, or `manifest_json` plus `base_dir`. | `package-validation-report.v0`. | Validation report with structured issues; no biological payload expansion. | `asset_read_failed`, `invalid_asset_path`, `checksum_mismatch`, tokenizer/vocab/pipeline config issue codes. | Use after sequence workflow when verifying a local package directory. |
| `doctor` | Report MCP server readiness metadata. | No parameters. | `{ biors_version, platform, mcp_server_ready }`. | Small diagnostic payload. | MCP internal error only. | Use at session start to confirm local server readiness. |

## Agent Sequences

1. Validate sequence input with `validate`.
2. If counts are acceptable, call `workflow` with a matching `kind`, `profile`,
   and bounded `max_length`.
3. Use `package_validate_fields` for manifest-only checks.
4. Use `package_validate` only when the local package directory is available.
5. Request `include_records` or `include_payload` only when downstream work
   genuinely needs full per-record payloads.

## Non-Goals

- not an autonomous research agent.
- Not hosted execution.
- Not cloud model calls, telemetry, literature review, long-term memory, or remote lab automation.

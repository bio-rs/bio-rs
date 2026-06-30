# biors-mcp-server

Model Context Protocol (MCP) server for
[bio-rs](https://github.com/bio-rs/bio-rs) validation, tokenization, workflow,
and package-check tools.

This crate makes bio-rs agent-callable over local stdio through deterministic
JSON contracts. It is for research agents that need validation, model-ready
preparation, package checks, and reproducible JSON without uploading biological
data.

## Usage

Run the server over stdio for local agent integration:

```bash
cargo run -p biors-mcp-server
```

Configure your MCP client (Claude Desktop, Cursor, etc.) to launch:

```json
{
  "mcpServers": {
    "bio-rs": {
      "command": "cargo",
      "args": ["run", "-p", "biors-mcp-server"]
    }
  }
}
```

## Output Policy

MCP tools return machine-readable JSON text. For long biological FASTA inputs,
sequence tools protect agent context by returning summary/counts/issues by
default using `schema_version: "biors.mcp.compact.v0"`. Request full per-record
payloads only when the caller actually needs them:

- `include_records: true` for `tokenize` and `validate`.
- `include_payload: true` for `workflow`.

Package tools already return validation summaries and structured issues rather
than biological payloads. All tools run locally and do not upload data,
telemetry, or model inputs.

The compact-output contract is summary/counts/issues by default.

## Tool Catalog

| Tool | Purpose | Default payload behavior |
| --- | --- | --- |
| `tokenize` | Tokenize FASTA text with a protein, DNA, or RNA tokenizer profile. | Compact for long FASTA unless `include_records=true`. |
| `validate` | Validate FASTA text as `auto`, `protein`, `dna`, or `rna`. | Compact for long FASTA unless `include_records=true`. |
| `workflow` | Run validation -> tokenization -> model-input preparation. | Compact for long FASTA unless `include_payload=true`. |
| `package_validate_fields` | Validate package manifest JSON fields without filesystem artifact checks. | Field-only report with structured issues. |
| `package_validate` | Validate a package manifest and local artifacts. | Validation report with structured issues; no biological payload expansion. |
| `package_bridge` | Plan package runtime bridge contract readiness from a local manifest path or manifest JSON plus base directory. | Bridge report with `contract_ready`, `artifact_checked`, and `execution_ready`; does not execute models. |
| `doctor` | Report MCP server readiness metadata. | Small diagnostic payload. |

## Agent Sequence

1. Validate sequence input with `validate`.
2. If counts are acceptable, call `workflow` with a matching `kind`, `profile`,
   and bounded `max_length`.
3. Use `package_validate_fields` for manifest-only checks.
4. Use `package_validate` only when the local package directory is available.
5. Use `package_bridge` when an agent needs package runtime compatibility
   planning fields without model execution.
6. Request `include_records` or `include_payload` only when downstream work
   genuinely needs full per-record payloads.

## Non-Goals

This server is not an autonomous research agent. It is not hosted execution and
does not provide cloud model calls, telemetry, literature review, long-term
memory, or remote lab automation.

## License

MIT OR Apache-2.0

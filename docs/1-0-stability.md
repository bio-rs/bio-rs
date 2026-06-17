# bio-rs 1.0 Surface Roles And Stability

This is a product 1.0 classification, not a statement that every implemented
surface is product-primary. The local bio-AI tool-layer target is organized by
researcher and research-agent workflows first; repository ownership and crate
boundaries are secondary.

## Product Roles

| Role | Meaning |
| --- | --- |
| primary researcher interface | Local CLI workflows a researcher runs directly. |
| primary agent interface | Local MCP tools and compact JSON contracts a research agent can call. |
| embedding interface | Rust, Python, and WASM APIs for tools that embed bio-rs locally. |
| secondary local integration | Local HTTP/service and diagnostics surfaces that wrap the same contracts. |
| package/artifact assurance | Package validation, verification, bridge, compatibility, diff, and migration surfaces. |
| preview/internal | Useful implementation or preview surface that is not a 1.0 product promise. |

## Workspace Crates

| Surface | Product Role | 1.0 Notes |
| --- | --- | --- |
| `biors` | primary researcher interface | CLI binary that carries the promoted local workflows. |
| `biors-backend-candle` | preview/internal | Optional backend adapter; not a primary 1.0 promise. |
| `biors-core` | embedding interface | Deterministic local validation, tokenization, package, service, and workflow logic. |
| `biors-mcp-server` | primary agent interface | Local stdio tools for research agents. |
| `biors-python` | embedding interface | Local scripting integration over supported core contracts. |
| `biors-wasm` | embedding interface | Local browser/JavaScript validation and workflow embedding, not browser model execution. |

## CLI Commands

| Surface | Product Role | 1.0 Notes |
| --- | --- | --- |
| `biors batch` | primary researcher interface | Batch local validation for file sets. |
| `biors completions` | primary researcher interface | CLI ergonomics for local terminal use. |
| `biors dataset` | preview/internal | Local inspection utility, not a central 1.0 workflow. |
| `biors debug` | preview/internal | Diagnostic command for maintainers. |
| `biors diff` | package/artifact assurance | Compares expected and observed local outputs. |
| `biors doctor` | secondary local integration | Local platform/readiness diagnostics. |
| `biors fasta` | primary researcher interface | FASTA validation entry point. |
| `biors formats` | primary researcher interface | FASTQ and format capability validation entry point. |
| `biors inspect` | primary researcher interface | Local file inspection surface. |
| `biors model-input` | primary researcher interface | Builds model-ready local records from validated biological inputs. |
| `biors molecule` | primary researcher interface | Local molecule validation and inspection. |
| `biors package` | package/artifact assurance | Package inspect, validate, verify, bridge, migrate, compatibility, and diff. |
| `biors pipeline` | primary researcher interface | Local preprocessing plan execution and lock generation. |
| `biors seq` | primary researcher interface | Kind-aware protein/DNA/RNA validation. |
| `biors report` | primary researcher interface | Reproducible local Markdown/JSON reports. |
| `biors serve` | secondary local integration | Narrow local HTTP service wrapper. |
| `biors service` | secondary local integration | Service contract and local integration commands. |
| `biors structure` | primary researcher interface | Local PDB validation and sequence extraction. |
| `biors tokenize` | primary researcher interface | Local tokenization for supported sequence profiles. |
| `biors tokenizer` | preview/internal | Tokenizer inspection and preview conversion utilities. |
| `biors workflow` | primary researcher interface | Promoted validation-to-model-input workflow command. |

## MCP Tools

| Surface | Product Role | 1.0 Notes |
| --- | --- | --- |
| `doctor` | secondary local integration | Reports local MCP server readiness. |
| `package_validate` | primary agent interface | Agent-callable package validation with filesystem checks. |
| `package_validate_fields` | primary agent interface | Agent-callable package field validation without filesystem checks. |
| `tokenize` | primary agent interface | Agent-callable local tokenization. |
| `validate` | primary agent interface | Agent-callable sequence validation. |
| `workflow` | primary agent interface | Agent-callable validation-to-model-input workflow. |

## Service Routes

| Surface | Product Role | 1.0 Notes |
| --- | --- | --- |
| `GET /health` | secondary local integration | Local health and policy status. |
| `GET /openapi.json` | secondary local integration | Local OpenAPI document for the narrow service surface. |
| `POST /v0/batch/sequence/validate` | secondary local integration | Inline FASTA batch validation for local service callers. |

## Schema Stability

Schema lifecycle is separate from crate versioning. These classes describe the
1.0 contract posture of each checked schema file:

| Class | Meaning |
| --- | --- |
| public stable | Promoted 1.0 contract shape; changes require migration notes. |
| public experimental | Publicly documented but still allowed to evolve before 1.0. |
| internal-only | Used for local integration, diagnostics, or generated wrappers rather than a central product promise. |
| candidate for merge | Legacy, preview, or overlapping schema that should be merged or retired before a hard 1.0 freeze. |

| Schema | Stability Class | Product Role |
| --- | --- | --- |
| `batch-validation-output.v0.json` | public experimental | primary researcher interface |
| `bio-entity-export-output.v0.json` | public experimental | embedding interface |
| `browser-tooling-output.v0.json` | internal-only | embedding interface |
| `cli-error.v0.json` | public stable | primary researcher interface |
| `cli-success.v0.json` | public stable | primary researcher interface |
| `dataset-inspect-output.v0.json` | public experimental | preview/internal |
| `doctor-output.v0.json` | public experimental | secondary local integration |
| `fasta-validation-output.v0.json` | public stable | primary researcher interface |
| `fastq-validation-output.v0.json` | public experimental | primary researcher interface |
| `format-capabilities-output.v0.json` | public experimental | primary researcher interface |
| `inspect-output.v0.json` | public experimental | primary researcher interface |
| `model-input-output.v0.json` | public stable | primary researcher interface |
| `molecule-records-output.v0.json` | public experimental | primary researcher interface |
| `molecule-validation-output.v0.json` | public experimental | primary researcher interface |
| `output-diff.v0.json` | public experimental | package/artifact assurance |
| `package-bridge-output.v0.json` | public experimental | package/artifact assurance |
| `package-compatibility-output.v0.json` | public experimental | package/artifact assurance |
| `package-conversion-output.v0.json` | candidate for merge | preview/internal |
| `package-diff-output.v0.json` | public experimental | package/artifact assurance |
| `package-inspect-output.v0.json` | public experimental | package/artifact assurance |
| `package-manifest.v0.json` | candidate for merge | package/artifact assurance |
| `package-manifest.v1.json` | public stable | package/artifact assurance |
| `package-migration-output.v0.json` | public experimental | package/artifact assurance |
| `package-skeleton-output.v0.json` | public experimental | package/artifact assurance |
| `package-validation-report.v0.json` | public stable | package/artifact assurance |
| `package-verify-output.v0.json` | public stable | package/artifact assurance |
| `pipeline-config.v0.json` | public stable | primary researcher interface |
| `pipeline-lock.v0.json` | public experimental | primary researcher interface |
| `pipeline-output.v0.json` | public experimental | primary researcher interface |
| `report-output.v0.json` | public stable | primary researcher interface |
| `sequence-debug-output.v0.json` | internal-only | preview/internal |
| `sequence-workflow-output.v0.json` | public stable | primary researcher interface |
| `service-batch-sequence-validate-output.v0.json` | public experimental | secondary local integration |
| `service-batch-sequence-validate-request.v0.json` | public experimental | secondary local integration |
| `service-empty-request.v0.json` | internal-only | secondary local integration |
| `service-health-output.v0.json` | public experimental | secondary local integration |
| `service-interface-output.v0.json` | public experimental | secondary local integration |
| `service-openapi-output.v0.json` | public experimental | secondary local integration |
| `structure-sequence-output.v0.json` | public experimental | primary researcher interface |
| `structure-validation-output.v0.json` | public experimental | primary researcher interface |
| `tokenize-output.v0.json` | public stable | primary researcher interface |
| `tokenizer-conversion-output.v0.json` | candidate for merge | preview/internal |
| `tokenizer-inspect-output.v0.json` | public experimental | preview/internal |

## Cross-Surface Parity Gaps

Workflow parity details live in `docs/1-0-workflow-parity.md`. Important gaps
are explicit:

- Package validation and runtime bridge planning are not exposed in WASM.
- Package bridge planning is not exposed in MCP.
- Tokenization, model-input generation, package checks, and runtime bridge
  planning are unsupported on this surface for the local HTTP service.
- Some embedding APIs are intentionally different from CLI/MCP ergonomics; those
  differences are documented as integration-surface boundaries rather than
  researcher workflow failures.

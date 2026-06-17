# bio-rs 1.0: Local bio-AI Tool Layer

## Thesis

For product-scope 1.0, the target is a local-first bio-AI tool layer for
researchers and research agents. bio-rs should provide deterministic tools for
validating biological inputs, preparing model-ready records, verifying
packages/artifacts, producing reproducible JSON reports, and exposing those
operations through CLI, MCP, and bindings without uploading data.

CLI and MCP are the primary 1.0 surfaces. CLI is the researcher-facing local
workflow surface. MCP is the agent-facing local tool surface, served over stdio
with deterministic JSON contracts. Rust, Python, WASM, and the local HTTP
service are integration surfaces for embedding the same local contracts in
applications, scripts, browser tooling, and local service wrappers.

1.0 is explicitly not a hosted platform and not an autonomous research agent.
It should make local validation, model-input preparation, package verification,
and report generation reliable enough for humans and agents to call, inspect,
and compose.

## Primary Users

| User | 1.0 need | Success condition |
| --- | --- | --- |
| Bio researcher using a terminal or scripts | Local validation, reproducible JSON output, clear package checks, and no default data upload | Can run documented CLI workflows on biological files and packages, then understand the next local action from structured output |
| Research agent or coding agent | Stable callable tools, predictable errors, compact JSON, and local-only behavior | Can call MCP or CLI tools in a sequence without relying on hidden network, telemetry, persistence, or model execution |
| Bio-AI tool builder | Embeddable validation, tokenization, model-input, package, and report contracts | Can use Rust/Python/WASM/service surfaces without reimplementing core validation and package rules |
| Maintainer or releaser | Evidence that claimed surfaces still work locally | Can verify docs, schemas, fixtures, package checks, bindings, MCP, and service behavior before a release decision |

## 1.0 Workflows

The 1.0 workflows are local, deterministic, and evidence-backed:

1. Validate biological inputs such as protein, DNA, RNA, FASTQ, PDB, SMILES,
   SDF, and MOL2 where the implementation and docs already define support.
2. Prepare model-ready records through explicit validation, tokenization,
   model-input, workflow, and report commands.
3. Verify package manifests, package-relative paths, artifact checksums,
   tokenizer/vocab artifacts, fixtures, and runtime bridge plans.
4. Produce reproducible JSON reports and contract-shaped outputs for CLI,
   bindings, MCP, and local service callers.
5. Run local service checks through the narrow HTTP service surface for health,
   OpenAPI, and inline FASTA batch validation.
6. Keep sequence-kind claims bounded: DNA/RNA are supported for the documented
   validation, tokenization, model-input, workflow, bindings, MCP, service, and
   package-validation paths, while arbitrary DNA/RNA package conversion is not
   promoted as a 1.0 promise.

Benchmark artifacts are regression guardrails for the checked workloads. They
are not broad throughput or scientific performance claims.

## Agent Use Model

bio-rs 1.0 should be agent-callable, not agent-autonomous. A research agent can
use bio-rs as a local tool layer by calling deterministic CLI or MCP operations
for validation, workflow execution, package checks, and JSON reporting. The
agent remains responsible for task planning, interpretation, and any external
systems it chooses to use.

For prompt-injection boundaries, research-agent use treats untrusted external text
as data, not instructions, including biological content, package metadata, and
report content, and must not follow embedded instructions.

The MCP surface should be treated as a local stdio tool adapter over the same
contracts exposed elsewhere. The expected agent pattern is:

1. Inspect or validate a local input.
2. Run a bounded workflow or package check.
3. Read the structured JSON result.
4. Decide the next local command or report step.

The 1.0 target does not include remote orchestration, autonomous literature or
lab decision-making, hidden persistence, or model inference on behalf of the
agent.

## Integration Surfaces

CLI and MCP carry the primary product narrative for 1.0. The integration
surfaces are:

- Rust: core deterministic validation, tokenization, model-input, workflow,
  package, report, service-contract, and schema-adjacent APIs.
- Python: local scripting bindings for the supported validation,
  tokenization/model-input, workflow, and package operations.
- WASM: browser or JavaScript embedding of local validation/workflow APIs, not
  browser model execution.
- service: the local HTTP mode for health, OpenAPI, and inline FASTA batch
  validation; callers own any production wrapper, authentication, ingress,
  logging, or broader deployment policy.

## Non-Goals

The following are out of scope for product-scope 1.0:

- No hosted service or hosted workspace operations.
- No autonomous research agent behavior.
- No telemetry, analytics, or default external reporting.
- No cloud model calls, external model inference, or default network execution.
- No browser model execution or WebGPU/runtime claims.
- Full arbitrary DNA/RNA package conversion from external Python or
  Hugging Face projects.
- Broad benchmark claims beyond checked regression guardrails.
- Production serving guarantees such as authentication, authorization,
  rate-limiting, queueing, remote object storage, or infrastructure deployment.

## Evidence Required

Each 1.0 claim should be backed by current repository evidence:

- Implementation and tests for the claimed CLI, MCP, Rust, Python, WASM, or
  service surface.
- JSON schemas, contracts, fixtures, or docs that match the implementation.
- Local workflow QA for the promoted researcher and agent paths.
- Package safety checks for package-relative paths, traversal rejection,
  checksum validation, fixture verification, and known schema versions.
- Explicit documentation when a surface is preview, integration-only, or not
  promoted for 1.0.
- Release-readiness evidence from local commands before any version bump, tag,
  release, registry publish, or deployment decision.

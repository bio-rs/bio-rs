# Service Interface And Local HTTP Mode

bio-rs exposes deterministic service contracts for teams that want to place
the preprocessing layer behind a local command, lab service, notebook gateway,
agent tool, or workflow runner.

The contract is available from Rust:

```rust
let contract = biors_core::service::current_service_interface_document();
```

and from the CLI:

```bash
biors service contract
biors service hosted-boundary
```

The output is a JSON success envelope whose `data` payload matches
`schemas/service-interface-output.v0.json` for the service contract and
`schemas/hosted-workflow-boundary-output.v0.json` for the hosted boundary
contract.

## Local HTTP Mode

`biors serve` starts a local-first HTTP JSON server:

```bash
biors serve --host 127.0.0.1 --port 8787
```

The default bind address is `127.0.0.1:8787`. The runtime performs no external
network calls, uploads, telemetry, model inference, request persistence, remote
object storage access, or hosted workspace operations. Inputs are processed in
memory and returned to the caller as JSON.

Current endpoints:

| Endpoint | Purpose | Schema |
|---|---|---|
| `GET /health` | Return service version, local execution policy, and enabled endpoints | `service-health-output.v0.json` |
| `GET /openapi.json` | Return the served OpenAPI 3.1 document for the local runtime | `service-openapi-output.v0.json` |
| `POST /v0/batch/sequence/validate` | Validate multiple inline FASTA payloads with `auto`, `protein`, `dna`, or `rna` kind policy | `service-batch-sequence-validate-output.v0.json` |

Example request:

```bash
curl -s http://127.0.0.1:8787/v0/batch/sequence/validate \
  -H 'content-type: application/json' \
  -d '{"kind":"auto","inputs":[{"id":"sample1","fasta_text":">seq1\nACDE\n"}]}'
```

## Boundary

`biors-core` owns the operation list, schema names, deterministic validation
contracts, OpenAPI metadata, and runtime/package separation policy. It still
does not bind sockets, authenticate users, rate-limit requests, queue jobs,
touch remote object storage, or deploy infrastructure.

The CLI crate owns the built-in local HTTP listener. External service hosts can
still adapt the same deterministic contracts to their own transport and
deployment stack.

This keeps the research contract portable across:

- local command wrappers
- internal lab services
- agent tool adapters
- workflow runners
- containerized local REST templates

## Hosted Boundary

`biors service hosted-boundary` emits a machine-readable policy document for
teams evaluating hosted bio-rs workflows. The contract keeps the published
open-source packages local-first and no-network-by-default. User accounts,
project workspaces, remote object storage, billing, audit logs, product web UI,
and landing pages are assigned to a separate hosted web or service layer.

Any hosted layer that wraps bio-rs output must record explicit consent before
remote processing, avoid silent biological data uploads, preserve input/output
hashes, pin the bio-rs package version and schema identifiers, and provide
retention, deletion, and export controls for persisted project data.

## Operations

The v0 service surface covers workflows that bio-AI researchers need before
inference:

| Operation | Purpose | Boundary |
|---|---|---|
| `service.health` | Report local service status and endpoint inventory | CLI local server |
| `service.openapi` | Serve the local OpenAPI 3.1 document | CLI local server |
| `sequence.batch_validate` | Validate multiple inline FASTA payloads without filesystem access | deterministic core |
| `sequence.validate` | Validate FASTA and emit structured residue diagnostics | deterministic core |
| `sequence.inspect` | Summarize FASTA records and input hashes | deterministic core |
| `sequence.tokenize` | Tokenize protein, DNA, or RNA FASTA with stable profiles | deterministic core |
| `model_input.build` | Build model-ready `input_ids` and `attention_mask` arrays for explicit profiles | deterministic core |
| `package.inspect` | Inspect package metadata and artifact declarations | package contract |
| `package.validate` | Validate package layout, checksums, fixtures, and metadata | package contract |
| `package.bridge.plan` | Produce runtime bridge readiness without executing a backend | runtime planning only |
| `package.compatibility.compare` | Compare package schema/runtime compatibility | package contract |

All listed operations are deterministic and idempotent. File access is limited
to caller-provided inputs, read-only package directories, or no file access for
local runtime metadata endpoints.

## Request And Response Schemas

Each service route references a checked-in request schema and response schema
under `schemas/`. Hosts can wrap these payloads in HTTP, queue, notebook, or
agent transports without changing deterministic core behavior.

| Operation | Request example | Response schema |
|---|---|---|
| `service.health` | `{}` | `service-health-output.v0.json` |
| `service.openapi` | `{}` | `service-openapi-output.v0.json` |
| `sequence.batch_validate` | `{ "kind": "auto", "inputs": [{ "id": "sample1", "fasta_text": ">seq1\nACDE\n" }] }` | `service-batch-sequence-validate-output.v0.json` |
| `sequence.validate` | `{ "fasta_text": ">seq1\nACDE\n", "kind": "auto" }` | `fasta-validation-output.v0.json` |
| `sequence.inspect` | `{ "fasta_text": ">seq1\nACDE\n" }` | `inspect-output.v0.json` |
| `sequence.tokenize` | `{ "fasta_text": ">seq1\nACDE\n", "profile": "protein-20" }` or `{ "fasta_text": ">dna\nACGT\n", "profile": "dna-iupac" }` | `tokenize-output.v0.json` |
| `model_input.build` | `{ "fasta_text": ">seq1\nACDE\n", "profile": "protein-20", "max_length": 512, "pad_token_id": 0, "padding": "fixed_length" }` | `model-input-output.v0.json` |
| `package.inspect` | `{ "manifest": { "schema_version": "biors.package.v0", "...": "..." } }` | `package-inspect-output.v0.json` |
| `package.validate` | `{ "manifest": { "schema_version": "biors.package.v0", "...": "..." } }` | `package-validation-report.v0.json` |
| `package.bridge.plan` | `{ "manifest": { "schema_version": "biors.package.v0", "...": "..." } }` | `package-bridge-output.v0.json` |
| `package.compatibility.compare` | `{ "left_manifest": { "schema_version": "biors.package.v0", "...": "..." }, "right_manifest": { "schema_version": "biors.package.v1", "...": "..." } }` | `package-compatibility-output.v0.json` |

## OpenAPI

`GET /openapi.json` returns a generated OpenAPI 3.1 document for the endpoints
currently served by `biors serve`. The document references the checked-in JSON
schemas by stable `https://bio-rs.dev/schemas/...` identifiers.

## `biors-service` Crate Review

The 0.57.1 implementation keeps the HTTP runtime inside the `biors` CLI crate
instead of introducing `crates/biors-service`. That is intentional for this
release: the current server is small, local-only, dependency-light, and directly
tied to CLI process lifecycle. Splitting a crate now would add release surface
without giving researchers a stronger runtime contract.

The extraction point is concrete: create a dedicated service crate when the
runtime needs reusable middleware, auth hooks, async transport adapters,
background execution, or a public library API for embedding a server process.

## Versioning

The current contract version is `biors.service_interface.v0`. Route identifiers,
schema names, and boundary labels are intended to be stable within the same
minor release line. Because bio-rs is pre-1.0, incompatible contract changes may
ship in a future minor version and will be reflected in this document and the
JSON schema.

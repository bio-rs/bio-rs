# Service Interface Contract

bio-rs exposes a transport-agnostic service interface contract for teams that
want to place the deterministic preprocessing layer behind their own service.

The contract is available from Rust:

```rust
let contract = biors_core::service::current_service_interface_document();
```

and from the CLI:

```bash
biors service contract
```

The output is a JSON success envelope whose `data` payload matches
`schemas/service-interface-output.v0.json`.

## Boundary

The service contract is deliberately not a server runtime. `biors-core` owns the
operation list, route identifiers, JSON schema names, and runtime/package
separation policy. A service host owns request authentication, sockets, rate
limits, background jobs, remote object storage, observability pipelines, and
deployment.

This keeps the research contract portable across:

- local command wrappers
- internal lab services
- agent tool adapters
- workflow runners

without coupling `biors-core` to a specific web framework or hosting provider.

## Operations

The v0 service surface covers the workflows that bio-AI researchers currently
need before inference:

| Operation | Purpose | Boundary |
|---|---|---|
| `sequence.validate` | Validate FASTA and emit structured residue diagnostics | deterministic core |
| `sequence.inspect` | Summarize FASTA records and input hashes | deterministic core |
| `sequence.tokenize` | Tokenize protein FASTA with stable profiles | deterministic core |
| `model_input.build` | Build model-ready `input_ids` and `attention_mask` arrays | deterministic core |
| `package.inspect` | Inspect package metadata and artifact declarations | package contract |
| `package.validate` | Validate package layout, checksums, fixtures, and metadata | package contract |
| `package.bridge.plan` | Produce runtime bridge readiness without executing a backend | runtime planning only |
| `package.compatibility.compare` | Compare package schema/runtime compatibility | package contract |

All listed operations are deterministic and idempotent. File access is limited
to caller-provided inputs or read-only package directories.

## Request And Response Schemas

Each service route references a checked-in request schema and a checked-in
response schema under `schemas/`. Hosts can wrap these payloads in HTTP,
queue, notebook, or agent transports without changing the deterministic core
contract.

| Operation | Request example | Response schema |
|---|---|---|
| `sequence.validate` | `{ "fasta_text": ">seq1\nACDE\n", "kind": "auto" }` | `fasta-validation-output.v0.json` |
| `sequence.inspect` | `{ "fasta_text": ">seq1\nACDE\n" }` | `inspect-output.v0.json` |
| `sequence.tokenize` | `{ "fasta_text": ">seq1\nACDE\n", "profile": "protein-20" }` | `tokenize-output.v0.json` |
| `model_input.build` | `{ "fasta_text": ">seq1\nACDE\n", "max_length": 512, "pad_token_id": 0, "padding": "fixed_length" }` | `model-input-output.v0.json` |
| `package.inspect` | `{ "manifest": { "schema_version": "biors.package.v0", "...": "..." } }` | `package-inspect-output.v0.json` |
| `package.validate` | `{ "manifest": { "schema_version": "biors.package.v0", "...": "..." } }` | `package-validation-report.v0.json` |
| `package.bridge.plan` | `{ "manifest": { "schema_version": "biors.package.v0", "...": "..." } }` | `package-bridge-output.v0.json` |
| `package.compatibility.compare` | `{ "left_manifest": { "schema_version": "biors.package.v0", "...": "..." }, "right_manifest": { "schema_version": "biors.package.v1", "...": "..." } }` | `package-compatibility-output.v0.json` |

## OpenAPI Direction

The contract includes an `openapi` block with `status:
offline_contract`. That means OpenAPI generation is expected to be performed by
the embedding service host from the stable route and schema metadata. bio-rs does
not start an HTTP server, bind a network port, or prescribe authentication.

## Versioning

The current contract version is `biors.service_interface.v0`. Route identifiers,
schema names, and boundary labels are intended to be stable within the same
minor release line. Because bio-rs is pre-1.0, incompatible contract changes may
ship in a future minor version and will be reflected in this document and the
JSON schema.

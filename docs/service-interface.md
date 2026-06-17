# Service Interface And Local HTTP Mode

bio-rs exposes a small deterministic service contract for the built-in local
HTTP server. It covers health metadata, the served OpenAPI document, and inline
FASTA batch validation.

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

## Local HTTP Mode

`biors serve` starts a local-first HTTP JSON server:

```bash
biors serve --host 127.0.0.1 --port 8787
```

The default bind address is `127.0.0.1:8787`. The runtime performs no external
network calls, uploads, telemetry, model inference, request persistence, remote
object storage access, or hosted workspace operations. Inputs are processed in
memory and returned to the caller as JSON; the service does not upload
biological data.

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

The batch endpoint rejects empty input lists, empty IDs, duplicate IDs, empty
FASTA text, malformed JSON, oversized request bodies, and FASTA parsing errors
with structured JSON error codes.

## Local REST And Container Template

The built-in local REST surface is intentionally minimal:

| Method | Path | Purpose |
|---|---|---|
| `GET` | `/health` | Version, endpoint inventory, and local execution policy |
| `GET` | `/openapi.json` | OpenAPI 3.1 document for served endpoints |
| `POST` | `/v0/batch/sequence/validate` | Inline FASTA batch validation |

The checked-in Docker/OCI template is in `deploy/service/Dockerfile`:

```bash
docker build -f deploy/service/Dockerfile \
  --build-arg BIORS_VERSION=0.57.3 \
  -t biors-service:0.57.3 .

docker run --rm -p 8787:8787 biors-service:0.57.3
```

The container binds `0.0.0.0:8787` inside the container so Docker can publish
the port. The host mapping should stay local unless an operator deliberately
places authentication, TLS, request limits, and logging policy in front of it.

`biors serve` gives researchers a reproducible local REST surface for inline
FASTA batch validation. Operators own authentication, authorization, TLS
termination, request and body-size policy beyond `--max-body-bytes`, audit
logging, ingress controls, scaling, service supervision, and shutdown handling.
Keep biological input data in local or institution-controlled environments
unless a separate data-governance review explicitly approves wider exposure.

## Boundary

`biors-core` owns the served operation list, schema names, deterministic batch
validation contract, OpenAPI metadata, and local runtime boundary metadata. It
still does not bind sockets, authenticate users, rate-limit requests, queue
jobs, touch remote object storage, or deploy infrastructure.

The CLI crate owns the built-in local HTTP listener. External wrappers can
adapt the same served payloads to their own transport, but that hosted or
production layer is caller-owned.

This keeps the research contract portable across:

- local command wrappers
- internal lab services
- agent tool adapters
- workflow runners
- containerized local REST templates

## Operations

The v0 service surface lists only endpoints served by `biors serve`:

| Operation | Purpose | Boundary |
|---|---|---|
| `service.health` | Report local service status and endpoint inventory | CLI local server |
| `service.openapi` | Serve the local OpenAPI 3.1 document | CLI local server |
| `sequence.batch_validate` | Validate multiple inline FASTA payloads without filesystem access | deterministic core |

All listed operations are deterministic and idempotent. File access is limited
to inline caller-provided FASTA inputs, or no file access for local runtime
metadata endpoints.

## Request And Response Schemas

Each served route references a checked-in request schema and response schema
under `schemas/`.

| Operation | Request example | Response schema |
|---|---|---|
| `service.health` | `{}` | `service-health-output.v0.json` |
| `service.openapi` | `{}` | `service-openapi-output.v0.json` |
| `sequence.batch_validate` | `{ "kind": "auto", "inputs": [{ "id": "sample1", "fasta_text": ">seq1\nACDE\n" }] }` | `service-batch-sequence-validate-output.v0.json` |

## OpenAPI

`GET /openapi.json` returns a generated OpenAPI 3.1 document for the endpoints
currently served by `biors serve`. The document references the checked-in JSON
schemas by stable `https://bio-rs.dev/schemas/...` identifiers.

## `biors-service` Crate Review

The 0.57.3 implementation keeps the HTTP runtime inside the `biors` CLI crate
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

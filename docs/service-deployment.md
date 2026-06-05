# Service Template Deployment Guide

`biors serve` is a local-first REST API for deterministic validation and
preprocessing. It is designed for lab-local services, CI sidecars, notebooks,
and internal workflow runners. It is not a hosted model service and does not
perform external calls, model inference, telemetry, request persistence, or
remote object storage access.

## Local Server

```bash
biors serve --host 127.0.0.1 --port 8787
```

Useful checks:

```bash
curl -s http://127.0.0.1:8787/health
curl -s http://127.0.0.1:8787/openapi.json
curl -s http://127.0.0.1:8787/v0/batch/sequence/validate \
  -H 'content-type: application/json' \
  -d '{"kind":"auto","inputs":[{"id":"sample1","fasta_text":">seq1\nACDE\n"}]}'
```

## REST API Template

The minimal API surface is:

| Method | Path | Purpose |
|---|---|---|
| `GET` | `/health` | Version, endpoint inventory, and local execution policy |
| `GET` | `/openapi.json` | OpenAPI 3.1 document for served endpoints |
| `POST` | `/v0/batch/sequence/validate` | Inline FASTA batch validation |

The batch endpoint accepts:

```json
{
  "kind": "auto",
  "inputs": [
    {
      "id": "sample1",
      "fasta_text": ">seq1\nACDE\n"
    }
  ]
}
```

Use stable sample IDs. The server rejects empty input lists, empty IDs,
duplicate IDs, empty FASTA text, malformed JSON, oversized request bodies, and
FASTA parsing errors with structured JSON error codes.

## Docker/OCI Template

The checked-in template is in `deploy/service/Dockerfile`.

Build:

```bash
docker build -f deploy/service/Dockerfile \
  --build-arg BIORS_VERSION=0.57.0 \
  -t biors-service:0.57.0 .
```

Run on a local workstation:

```bash
docker run --rm -p 8787:8787 biors-service:0.57.0
```

The container binds `0.0.0.0:8787` inside the container so Docker can publish
the port. The host mapping should stay local unless an operator deliberately
places authentication, TLS, request limits, and logging policy in front of it.

## Operational Boundary

`biors serve` gives researchers a reproducible validation/tokenization-adjacent
REST surface, not a full production platform. Operators own:

- authentication and authorization
- TLS termination
- request and body-size policy beyond `--max-body-bytes`
- audit logging and retention policy
- ingress controls and network exposure
- scaling, service supervision, and shutdown handling

Keep biological input data in local or institution-controlled environments
unless a separate data-governance review explicitly approves wider exposure.

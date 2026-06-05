# Reproducible Reports

`biors report generate` converts bio-rs JSON output into a deterministic,
shareable report. It is intended for lab notes, package QA handoff, issue
triage, and reviewer-friendly summaries where the machine-readable source
payload must remain traceable.

## Generate A Report

```bash
biors workflow --max-length 8 examples/protein.fasta > workflow.json
biors report generate workflow.json \
  --output workflow-report.md \
  --shareable-json workflow-report.json
```

The command always writes a JSON success envelope to stdout. The `data` payload
uses `schemas/report-output.v0.json` and schema version `biors.report.v0`.
`--output` writes the deterministic Markdown report. `--shareable-json` writes
the bare shareable report JSON without the CLI envelope.

`-` is accepted for input so commands can be chained:

```bash
biors seq validate --kind auto examples/multi.fasta \
  | biors report generate - --output validation-report.md
```

File output paths cannot be `-` because stdout is reserved for the CLI JSON
envelope.

## Input Contracts

The report generator accepts any valid JSON. It recognizes and summarizes these
bio-rs payload families:

| Input shape | Report kind |
|---|---|
| CLI success envelope with `data.workflow`, `data.provenance`, and `data.tokenization` | `sequence_workflow_output` |
| CLI error envelope | `cli_error` |
| `schema_version: "biors.conversion.v0"` | `bio_entity_export` |
| JSON object with `valid` and `records` fields | `validation_report` |
| Other JSON | `generic_json` |

Recognized payloads get domain-specific sections for validation counts,
model-readiness, conversion issues, entity types, or CLI error codes. Generic
JSON still receives deterministic provenance and a top-level field summary.

## Provenance

Reports are reproducible for the same input bytes. The shareable report records:

- bio-rs core version and report generator schema
- detected input container and input kind
- source CLI version and `input_hash` when present
- raw input SHA-256
- canonical JSON SHA-256
- rendered Markdown SHA-256

Raw and canonical hashes intentionally both exist. The raw hash tracks the
exact artifact bytes. The canonical JSON hash lets reviewers recognize
semantically identical JSON that only changed whitespace or object field order.

The report does not call external services, upload biological data, fetch
remote metadata, persist source payloads, or infer scientific conclusions beyond
the source JSON.

## Shareable Format

`biors.report.v0` contains:

- `title`, `summary`, and `status`
- `provenance`
- deterministic report `sections`
- `human_report` Markdown

Downstream systems should validate the JSON against
`schemas/report-output.v0.json`, then render `human_report` or build their own
view from `sections`. Required fields and enum removals require a new report
schema version.

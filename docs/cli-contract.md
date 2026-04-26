# CLI and JSON Contract

This document freezes the candidate `0.9.0` CLI behavior for review before
`1.0.0`.

## Commands

- `biors tokenize <path|->`
- `biors inspect <path|->`
- `biors fasta validate <path|->`
- `biors package inspect <manifest>`
- `biors package validate <manifest|->`
- `biors package bridge <manifest>`
- `biors package verify <manifest> <observations>`

## JSON Success Envelope

All successful command output is written to stdout as pretty JSON:

```json
{
  "ok": true,
  "biors_version": "0.9.0",
  "input_hash": "fnv1a64:...",
  "data": {}
}
```

`input_hash` is present for FASTA-backed commands. Package commands omit it
unless they directly hash a user input contract in a later release.

## JSON Error Mode

Passing `--json` writes errors to stdout as:

```json
{
  "ok": false,
  "error": {
    "code": "fasta.missing_header",
    "message": "human readable message",
    "location": {
      "line": 1,
      "record_index": null
    }
  }
}
```

Without `--json`, errors are written to stderr as human-readable text.

## Exit Codes

- `0`: success
- `1`: I/O or internal serialization failure
- `2`: user input, FASTA, JSON, package validation, bridge, or verification failure

## Canonical JSON Serialization

The canonical CLI output policy is:

- UTF-8 JSON
- stdout only on success
- pretty printed with two-space indentation
- field order follows the public Rust response structs
- stderr is empty for successful commands
- `--json` errors use stdout and keep stderr empty

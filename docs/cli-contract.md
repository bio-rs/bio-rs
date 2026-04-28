# CLI and JSON Contract

This document records the frozen `0.9.6` CLI and JSON contract surface before
`1.0.0`.

## Commands

- `biors tokenize <path|->`
- `biors inspect <path|->`
- `biors model-input --max-length <usize> [--pad-token-id <u8>] [--padding fixed_length|no_padding] <path|->`
- `biors fasta validate <path|->`
- `biors package inspect <manifest>`
- `biors package validate <manifest|->`
- `biors package bridge <manifest>`
- `biors package verify <manifest> <observations>`

`model-input` tokenizes FASTA records and emits deterministic model-ready `input_ids` plus `attention_mask` records.
It rejects sequences that still contain residue warnings or errors, so model-ready output cannot silently drop unresolved residues.
`--max-length` must be greater than zero.
`tokenize` preserves positional alignment by emitting explicit unknown-token IDs for ambiguous or invalid residues instead of shortening the token vector.

Manifest-relative paths are resolved against the manifest file's parent directory. If the manifest is read from stdin, relative paths are resolved against the current working directory.
Absolute paths and `..` parent traversal are rejected so packages remain portable and self-contained.

Observation paths in `package verify` are resolved against the observations file's parent directory. If the observations file is read from stdin, relative paths are resolved against the current working directory.
Absolute observation paths and `..` parent traversal are rejected for the same reason.

The package manifest contract is closed over enumerated values for `schema_version`, `model.format`, `runtime.backend`, `runtime.target`, and tensor `dtype` fields. Unsupported values fail JSON deserialization instead of being accepted as loose strings.

## JSON Success Envelope

All successful command output is written to stdout as pretty JSON:

```json
{
  "ok": true,
  "biors_version": "0.9.6",
  "input_hash": "fnv1a64:...",
  "data": {}
}
```

`input_hash` is present for FASTA-backed commands. Package commands omit it
unless they directly hash a user input contract in a later release.

The `input_hash` field remains `fnv1a64:` for FASTA-backed compatibility. Package manifest checksums and fixture hashes use `sha256:`.

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

## Package Verification Inputs

`package verify` reads an observations JSON array shaped like:

```json
[
  {
    "name": "tiny-protein",
    "path": "observed/tiny.output.json"
  }
]
```

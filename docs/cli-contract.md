# CLI and JSON Contract

This document records the current pre-1.0 CLI and JSON contract surface.

## Commands

- `biors --version`
- `biors completions <bash|elvish|fish|powershell|zsh>`
- `biors doctor`
- `biors batch validate [--kind auto|protein|dna|rna] <path|directory|glob>...`
- `biors tokenize <path|->`
- `biors tokenize [--profile protein-20|protein-20-special] [--config <json>] <path|->`
- `biors tokenizer inspect [--profile protein-20|protein-20-special] [--config <json>]`
- `biors inspect <path|->`
- `biors model-input --max-length <usize> [--pad-token-id <u8>] [--padding fixed_length|no_padding] <path|->`
- `biors workflow --max-length <usize> [--pad-token-id <u8>] [--padding fixed_length|no_padding] <path|->`
- `biors fasta validate [--kind protein|dna|rna|auto] <path|->`
- `biors seq validate [--kind auto|protein|dna|rna] <path|->`
- `biors package inspect <manifest>`
- `biors package validate <manifest|->`
- `biors package bridge <manifest>`
- `biors package verify <manifest> <observations>`

`model-input` tokenizes FASTA records and emits deterministic model-ready `input_ids` plus `attention_mask` records.
`workflow` runs protein FASTA validation, deterministic `protein-20`
tokenization, model-input generation, readiness reporting, and reproducibility
provenance in a single JSON payload. It keeps validation and tokenization
context when residues are not model-ready and sets `model_ready=false` with
stable `sequence.not_model_ready` readiness issue codes.
`biors --version` prints the installed CLI package version so workflow logs and
benchmark records can be tied back to the exact released binary.
`biors completions <shell>` writes shell completion scripts to stdout.
`biors doctor` emits local readiness diagnostics for platform identity,
available Rust/Cargo toolchains, optional WASM target support, and committed
demo/package fixtures.
`batch validate` accepts multiple file paths, directories, and quoted glob
patterns. Directory inputs include common FASTA file extensions and ignore
unrelated files. It emits memory-bounded per-file validation summaries and a
batch summary without retaining per-record validation payloads.
It rejects sequences that still contain residue warnings or errors, so model-ready output cannot silently drop unresolved residues.
`--max-length` must be greater than zero.
`tokenize` preserves positional alignment by emitting explicit unknown-token IDs for ambiguous or invalid residues instead of shortening the token vector.
Without `--config`, tokenization defaults to the stable `protein-20` profile.
Tokenizer config JSON currently supports `profile` and `add_special_tokens`.
The built-in `protein-20-special` profile keeps residue IDs stable and exposes
`UNK=20`, `PAD=21`, `CLS=22`, `SEP=23`, and `MASK=24`.
`tokenizer inspect` emits the resolved config, vocabulary, unknown-token policy,
and special-token policy as JSON.
FASTA validation defaults to the protein policy for pre-0.14 compatibility.
Passing `--kind dna`, `--kind rna`, or `--kind protein` applies one policy to
all records; `--kind auto` assigns `protein`, `dna`, or `rna` per record and
defaults ambiguous nucleotide-only ties such as `ACGN` to DNA.
`seq validate` uses the same output contract but defaults to `--kind auto` for
mixed biological sequence batches.
FASTA-backed CLI commands read through buffered reader APIs and compute the legacy `fnv1a64:` input hash during the same pass.
`inspect` uses a summary-only reader path and does not materialize token vectors
when it only needs record, residue, warning, and error counts.

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
  "biors_version": "0.x.y",
  "input_hash": "fnv1a64:...",
  "data": {}
}
```

`input_hash` is present for FASTA-backed commands. Package commands omit it
unless they directly hash a user input contract in a later release.

The `input_hash` field remains `fnv1a64:` for FASTA-backed compatibility. Package manifest checksums and fixture hashes use `sha256:`.

Package validation reports include both the legacy string `issues` list and a typed `structured_issues` list with stable issue codes.

FASTA validation reports include `kind_counts` and per-record `kind` /
`alphabet` fields. Sequence warnings and errors expose stable issue codes such
as `ambiguous_symbol` and `invalid_symbol` plus human-readable messages.

Workflow payloads use `schemas/sequence-workflow-output.v0.json`. The
provenance section records the `biors-core` version, input hash, normalization
policy, validation alphabet, tokenizer metadata, model-input policy, resolved
CLI invocation arguments, vocabulary SHA-256, and output-content SHA-256. The
output-content digest covers the workflow payload excluding the hash values
themselves.

Batch validation payloads use `schemas/batch-validation-output.v0.json` and
include `inputs`, aggregate `summary`, and a deterministic `files` list with
per-file `input_hash`, validation counts, and `kind_counts`.

Tokenizer inspection payloads use `schemas/tokenizer-inspect-output.v0.json`.
Tokenizer config files reject unknown top-level fields so preprocessing
configuration stays explicit.

`structured_issues` entries use this shape:

```json
{
  "code": "invalid_asset_path",
  "field": "fixtures[0].input",
  "message": "asset path '../outside.fasta' must stay inside the package root"
}
```

Package verification reports include stable `issue_code` values when a fixture
does not pass. Content mismatches also include a `content_diff` object with the
expected path, observed path, canonical byte lengths, and first differing byte
offset when one exists.

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

Without `--json`, errors are written to stderr as `error[code]: message`.

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
- success envelopes are serialized directly to the stdout writer; the JSON shape is unchanged

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

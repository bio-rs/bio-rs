# CLI and JSON Contract

This document records the current pre-1.0 CLI and JSON contract surface.

## Commands

- `biors --version`
- `biors cache inspect [--root <path>]`
- `biors cache clean [--root <path>] (--dry-run|--yes)`
- `biors completions <bash|elvish|fish|powershell|zsh>`
- `biors dataset inspect [--source <name>] [--version <version>] [--split <split>] [--metadata key=value]... <path|directory|glob>...`
- `biors debug --max-length <usize> <path|->`
- `biors diff <expected> <observed>`
- `biors doctor`
- `biors batch validate [--kind auto|protein|dna|rna] <path|directory|glob>...`
- `biors tokenize <path|->`
- `biors tokenize [--profile protein-20|protein-20-special|dna-iupac|dna-iupac-special|rna-iupac|rna-iupac-special] [--config <json>] <path|->`
- `biors tokenizer convert-hf <tokenizer_config.json> [--output <json>]`
- `biors tokenizer inspect [--profile protein-20|protein-20-special|dna-iupac|dna-iupac-special|rna-iupac|rna-iupac-special] [--config <json>]`
- `biors inspect <path|->`
- `biors model-input [--profile protein-20|protein-20-special|dna-iupac|dna-iupac-special|rna-iupac|rna-iupac-special] --max-length <usize> [--pad-token-id <u8>] [--padding fixed-length|no-padding] <path|->`
- `biors workflow [--profile protein-20|protein-20-special|dna-iupac|dna-iupac-special|rna-iupac|rna-iupac-special] --max-length <usize> [--pad-token-id <u8>] [--padding fixed-length|no-padding] <path|->`
- `biors fasta validate [--kind protein|dna|rna|auto] <path|->`
- `biors seq validate [--kind auto|protein|dna|rna] <path|->`
- `biors package inspect <manifest>`
- `biors package validate <manifest|->`
- `biors package init <output-dir> --name <name> --model <model.onnx> [--tokenizer-config <json>] --fixture-input <fasta> --fixture-output <json> --license <expr> --citation <text> [--doi <doi>] --model-card-summary <text> --intended-use <text> --limitation <text> [--force]`
- `biors package migrate <manifest|-> [--to biors.package.v0|biors.package.v1]`
- `biors package convert <manifest|-> [--to biors.package.v1] [--output <manifest.json>] --license <expr> --citation <text> [--doi <doi>] --model-card <path> --model-card-summary <text> --intended-use <text> --limitation <text> [--license-file <path>] [--citation-file <path>] [--models-dir <dir>] [--tokenizers-dir <dir>] [--vocabs-dir <dir>] [--pipelines-dir <dir>] [--fixtures-dir <dir>] [--observed-dir <dir>] [--docs-dir <dir>]`
- `biors package convert-project <python-project-dir> --output <package-dir> --name <name> [--model <model.onnx>] [--include-generated] [--tokenizer-config <json>] --fixture-input <fasta> --fixture-output <json> --license <expr> --citation <text> [--doi <doi>] --model-card-summary <text> --intended-use <text> --limitation <text> [--force]`
- `biors package compatibility <left-manifest> <right-manifest>`
- `biors package diff <left-manifest> <right-manifest>`
- `biors package bridge <manifest>`
- `biors package verify <manifest> <observations>`
- `biors pipeline --max-length <usize> [--pad-token-id <u8>] [--padding fixed-length|no-padding] <path|->`
- `biors pipeline --config <toml|json> [--dry-run] [--explain-plan]`
- `biors pipeline --config <toml|json> [--package <manifest>] --write-lock <pipeline.lock>`
- `biors service contract`

`model-input` tokenizes FASTA records with the selected tokenizer profile and
emits deterministic model-ready `input_ids` plus `attention_mask` records.
`model-input-output.v0.json` is the structural JSON contract; integration
boundaries that accept externally supplied model-input JSON must also run the
core `validate_model_input_payload` semantic validator for cross-field
invariants that JSON Schema cannot express portably.
`workflow` runs FASTA validation, deterministic profile tokenization,
model-input generation, readiness reporting, and reproducibility provenance in
a single JSON payload. It keeps validation and tokenization context when
residues are not model-ready and sets `model_ready=false` with stable
`sequence.not_model_ready` readiness issue codes.
`pipeline` wraps the same no-config preprocessing path in explicit
validate -> tokenize -> export step statuses for CLI chaining and pipeline
orchestration. With `--config`, it reads `biors.pipeline.v0` TOML/JSON and
runs parse -> normalize -> validate -> tokenize -> export. `--dry-run` validates
the config and emits planned stages without reading FASTA input, with
`ready: false` because no model-ready output was produced; `--explain-plan`
includes the static plan with an executing run. `--write-lock` records a
`biors.pipeline.lock.v0` file for an executed config pipeline. When `--package`
is supplied, the lock pins the package manifest path, model checksum, runtime
backend, runtime target, and backend version alongside the pipeline config hash,
vocabulary hash, input hash, and output-content hash. In that mode `--config`
must resolve to a preprocessing or postprocessing config artifact declared by
the package manifest.
`debug` emits a step-by-step per-record view from normalized sequence to token
IDs to model-input records, with compact `W`/`E` residue markers for warnings
and errors.
`diff` compares expected and observed outputs as canonical JSON when possible
and raw bytes otherwise. It always emits a report with SHA-256 hashes,
`matches`, and first-difference metadata for mismatches.
`biors --version` prints the installed CLI package version so workflow logs and
benchmark records can be tied back to the exact released binary.
`biors completions <shell>` writes shell completion scripts to stdout.
`biors doctor` emits local readiness diagnostics for platform identity and
capability groups covering core CLI, WASM, Python bindings, package fixtures,
release tooling, and benchmark tooling. Optional binding/release tools are
reported as warnings with install hints rather than causing the command to fail.
`biors service contract` emits the transport-agnostic service interface
contract. It lists deterministic operations, JSON schema names, file access
policy, runtime/package boundaries, and offline OpenAPI direction without
starting a service runtime.
`batch validate` accepts multiple file paths, directories, and quoted glob
patterns. Directory inputs recurse into nested folders, include common FASTA
file extensions, and ignore unrelated files. Empty glob expansion fails with
`batch.no_inputs`. The command emits memory-bounded per-file validation
summaries and a batch summary without retaining per-record validation payloads.
It rejects sequences that still contain residue warnings or errors, so model-ready output cannot silently drop unresolved residues.
`dataset inspect` uses the same FASTA file, recursive directory, and quoted glob
resolver as `batch validate`, then emits a dataset descriptor, optional
metadata, resolved file paths, byte counts, file SHA-256 values, record counts,
a portable dataset content hash, a local mapping hash, and a dataset-to-sample
mapping from FASTA record IDs. File inspection streams FASTA bytes and records
only sample IDs and sequence lengths rather than retaining full sequence
records; the emitted JSON can still be large when a dataset has many records
because `samples[]` is an explicit per-record mapping. Empty datasets fail with
`dataset.no_inputs`.
`cache inspect` reports the local artifact store root, layout policy, and file
inventory. The default root is `.biors/artifacts`, overridden by
`BIORS_ARTIFACT_STORE` or `--root`. `cache clean` accepts only `.biors/artifacts`
or an existing artifact-store layout, rejects broad or generic roots, and
requires `--dry-run` or `--yes`.
`--max-length` must be greater than zero.
`tokenize` preserves positional alignment by emitting explicit unknown-token IDs for ambiguous or invalid residues instead of shortening the token vector.
Without `--config`, tokenization defaults to the stable `protein-20` profile.
DNA and RNA tokenization are available through explicit `dna-iupac`,
`dna-iupac-special`, `rna-iupac`, or `rna-iupac-special` profiles. The
nucleotide profiles assign canonical bases to stable IDs, emit the profile
unknown token for ambiguous IUPAC symbols as warnings, and emit the same unknown
token for invalid symbols as errors.
Tokenizer config JSON currently supports `profile` and `add_special_tokens`.
`tokenizer convert-hf` accepts a Hugging Face `tokenizer_config.json`, maps it
to the closest supported protein tokenizer config, and emits preview
tokenizer/preprocessing fragments plus conversion assumptions, warnings, and a
`conversion_status` marker. The result is not package-ready until fixture parity
against the source tokenizer is validated. `output_path` remains the host write
destination; `preview_tokenizer_asset.path` is package-relative.
The built-in `protein-20-special` profile keeps residue IDs stable and exposes
`UNK=20`, `PAD=21`, `CLS=22`, `SEP=23`, and `MASK=24`.
`tokenizer inspect` emits the resolved config, vocabulary, unknown-token policy,
and special-token policy as JSON.
FASTA validation defaults to the protein policy for pre-0.14 compatibility.
Passing `--kind dna`, `--kind rna`, or `--kind protein` applies one policy to
all records; `--kind auto` assigns `protein`, `dna`, or `rna` per record and
defaults ambiguous nucleotide-only ties such as `ACGN` to DNA. Auto-detected
records include `auto_detection.selected_kind`,
`auto_detection.candidate_kinds`, and `auto_detection.ambiguous` so callers can
spot short or alphabet-overlapping sequences and rerun with an explicit kind.
`seq validate` uses the same output contract but defaults to `--kind auto` for
mixed biological sequence batches.
FASTA-backed CLI commands read through buffered reader APIs and compute the legacy `fnv1a64:` input hash during the same pass.
`inspect` uses a summary-only reader path and does not materialize token vectors
when it only needs record, residue, warning, and error counts.

Manifest-relative paths are resolved against the manifest file's parent directory. If the manifest is read from stdin, relative paths are resolved against the current working directory.
Absolute paths and `..` parent traversal are rejected so packages remain portable and self-contained.

Observation paths in `package verify` are resolved against the observations file's parent directory. If the observations file is read from stdin, relative paths are resolved against the current working directory.
Absolute observation paths and `..` parent traversal are rejected for the same reason.
`package verify` runs package artifact validation before fixture comparison. If
the manifest is invalid, the command fails with a package validation error and
includes the structured validation report in `error.details.validation`.
For CLI commands, package artifact validation also parses manifest-referenced
pipeline config artifacts with the `biors pipeline --config` validator.
Tokenizer artifacts are parsed as bio-rs tokenizer configs and checked against
manifest tokenizer profile metadata before a package is accepted.
Vocab artifacts are parsed as bio-rs `Vocabulary` JSON and checked against
manifest vocab metadata before a package is accepted.

The package manifest contract is closed over enumerated values for
`schema_version`, `model.format`, `runtime.backend`, `runtime.target`, and
tensor `dtype` fields. Unsupported values fail JSON deserialization instead of
being accepted as loose strings. `biors.package.v1` requires declared package
layout and research metadata for license, citation, and model-card inspection.

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

Package validation reports include both the legacy string `issues` list and a
typed `structured_issues` list with stable issue codes. Manifest v1 validation
also checks that artifacts live under the declared package layout directories.
Package migration reports use `schemas/package-migration-output.v0.json` and
emit the explicit steps needed to move between supported manifest schema
versions. Package compatibility reports use
`schemas/package-compatibility-output.v0.json` and state whether the left
manifest can be read or migrated as the right manifest schema. Package manifest
diff reports use `schemas/package-diff-output.v0.json`; the nested `diff`
payload follows the canonical JSON/raw comparison contract from
`schemas/output-diff.v0.json`.
Package conversion reports use `schemas/package-conversion-output.v0.json` and
include the converted manifest. Conversion to `biors.package.v1` requires
caller-supplied license, citation, model-card summary, intended-use, and
limitation fields so the CLI does not invent research metadata.
Advanced `package convert` layout-directory overrides (`--models-dir`,
`--tokenizers-dir`, `--vocabs-dir`, `--pipelines-dir`, `--fixtures-dir`,
`--observed-dir`, and `--docs-dir`) are supported for existing packages with
non-default layout names; they must still produce manifest-relative package
paths that pass artifact validation.
Package skeleton reports use `schemas/package-skeleton-output.v0.json`.
`package init` and `package convert-project` create a local package layout,
write docs and pipeline config, copy supplied fixture artifacts, and record
SHA-256 checksums. Python project conversion scans for one ONNX model and an
optional `tokenizer_config.json`, skipping generated/cache directories unless
`--include-generated` is passed. If multiple ONNX candidates remain, it fails
with `package.project_model_ambiguous` and returns the sorted candidate list in
JSON error details. If multiple tokenizer config candidates remain, it fails
with `package.project_tokenizer_config_ambiguous` and returns the sorted
candidate list in JSON error details. Optional model artifact metadata is left
unset for the package author to fill in.

FASTA validation reports include `kind_counts` and per-record `kind` /
`alphabet` fields. Records produced by `--kind auto` also include
`auto_detection` metadata with the selected kind, equally plausible candidate
kinds, and an ambiguity flag. Sequence warnings and errors expose stable issue
codes such as `ambiguous_symbol` and `invalid_symbol` plus human-readable
messages.

Workflow payloads use `schemas/sequence-workflow-output.v0.json`. The
provenance section records the `biors-core` version, input hash, normalization
policy, validation alphabet, tokenizer metadata, model-input policy, resolved
CLI invocation arguments, vocabulary SHA-256, and output-content SHA-256. The
output-content digest covers the workflow payload excluding the hash values
themselves.

Pipeline payloads use `schemas/pipeline-output.v0.json`; pipeline lockfiles use
`schemas/pipeline-lock.v0.json`. Debug payloads use
`schemas/sequence-debug-output.v0.json`. Output diff reports use
`schemas/output-diff.v0.json`.

Batch validation payloads use `schemas/batch-validation-output.v0.json` and
include `inputs`, aggregate `summary`, and a deterministic `files` list with
per-file `input_hash`, validation counts, and `kind_counts`.
Dataset inspection payloads use `schemas/dataset-inspect-output.v0.json` and
include `provided_inputs`, descriptor, metadata, resolved file count, total
byte count, sample count, portable dataset content hash, local mapping hash,
deterministic `resolved_files`, and sample mapping lists.
Cache payloads use `schemas/cache-output.v0.json`.
Service interface payloads use `schemas/service-interface-output.v0.json`.

Tokenizer inspection payloads use `schemas/tokenizer-inspect-output.v0.json`.
Tokenizer config files reject unknown top-level fields so preprocessing
configuration stays explicit.
Tokenizer conversion payloads use
`schemas/tokenizer-conversion-output.v0.json`.

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
`--json` also applies to CLI argument parse failures, including invalid enum
values rejected by clap; those use `code: "cli.invalid_arguments"` with
`location: null`.

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

# Python API Reference

`biors-python` exposes the core bio-rs FASTA, validation, tokenization, model
input, package-manifest, and runtime-planning surfaces through PyO3.

This is a local bio-AI integration surface for researchers and research agents
that need Python notebooks, scripts, or pipelines to call the same deterministic
validation, model-ready preparation, package checks, and reproducible JSON
contracts as the CLI.

> **Status:** The `biors-python` crate is implemented in this repository and is
> published to PyPI by the tag release workflow.

## Installation

```bash
pip install biors
```

Requirements:

- Python 3.9 or later
- No Rust toolchain on the install side when using a published wheel

For local development:

```bash
cd crates/biors-python
maturin develop
pytest
```

## Interop Paths

Python users can either consume schema-rich JSON from the `biors` CLI or call
the PyO3 bindings directly.

Use `biors workflow`, `biors pipeline`, or `biors debug` when notebooks,
PyTorch, Hugging Face, or other caller-owned code should control downstream
tensor and model runtime choices. The JSON boundary stays dependency-light and
deterministic. Model-specific adapters belong in caller projects because ESM,
ProtBERT, and similar model families each have their own tokenizer and tensor
contracts.

## Data Classes

The current Python boundary intentionally returns small immutable PyO3 classes.
For full schema-rich JSON reports, use the Rust CLI or core crate.

The wheel ships a `py.typed` marker and a maintained `biors/__init__.pyi` stub
for the public functions and compact PyO3 classes listed below.

| Type | Attributes |
| --- | --- |
| `ResidueIssue` | `residue: str`, `position: int` |
| `ProteinSequence` | `id: str`, `sequence: str` |
| `ValidatedSequence` | `id: str`, `sequence: str`, `alphabet: str`, `valid: bool`, `warnings: list[ResidueIssue]`, `errors: list[ResidueIssue]` |
| `SequenceValidationReport` | `records: int`, `valid_records: int`, `warning_count: int`, `error_count: int`, `sequences: list[ValidatedSequence]` |
| `TokenizedProtein` | `id: str`, `alphabet: str`, `valid: bool`, `tokens: list[int]`, `length: int`, `warnings: list[ResidueIssue]`, `errors: list[ResidueIssue]` |
| `ModelInput` | `records: list[ModelInputRecord]` |
| `ModelInputRecord` | `id: str`, `input_ids: list[int]`, `attention_mask: list[int]`, `truncated: bool` |
| `SequenceWorkflowOutput` | `model_ready: bool`, `input_hash: str`, `records: list[ModelInputRecord]`, `report_json: str` |

Errors are raised as `BioRsError`, a `ValueError` subclass with stable
`code`, `message`, and `location` attributes. FASTA parse errors preserve
structured locations such as `{"line": 1, "record_index": null}`; errors
without a source location expose `location = None`.

## FASTA Parsing

```python
import biors

fasta_text = """>sp|P01308|INS_HUMAN
MALWMRLLPLLALLALWGPDPAAA
>sp|P68871|HBB_HUMAN
MVHLTPEEKSAVTALWGKVNVDEVGGEALGR
"""

records = biors.parse_fasta_records(fasta_text)
for record in records:
    print(record.id, len(record.sequence))
```

### `parse_fasta_records(fasta_text: str) -> list[ProteinSequence]`

Parses FASTA text and returns record identifiers plus normalized sequence
strings. Malformed input raises `ValueError`.

## Validation

```python
report = biors.validate_fasta_input(fasta_text)
rna_report = biors.validate_fasta_input_with_kind(">rna\nACGUN\n", "rna")
print(report.valid_records, report.records)
print(report.warning_count, report.error_count)
for record in report.sequences:
    for issue in record.warnings:
        print(record.id, "ambiguous", issue.residue, issue.position)
    for issue in record.errors:
        print(record.id, "invalid", issue.residue, issue.position)
```

### `validate_fasta_input(fasta_text: str) -> SequenceValidationReport`

Returns a validation summary plus per-record diagnostics for FASTA input.
`warning_count` counts ambiguous but recognized residues such as `X`; `error_count`
counts residues outside the supported protein alphabet and ambiguity policy.
Diagnostic positions are one-based offsets in the normalized sequence.

### `validate_fasta_input_with_kind(fasta_text: str, kind: str) -> SequenceValidationReport`

Runs the same FASTA validation with an explicit sequence kind. `kind` must be
`auto`, `protein`, `dna`, or `rna`. DNA and RNA reports reuse the same
`SequenceValidationReport` and per-record `ResidueIssue` objects as the
protein-default helper.

## Tokenization

```python
tokenized = biors.tokenize_fasta_records(fasta_text)
for record in tokenized:
    print(record.id, record.alphabet, record.valid, record.length, record.tokens[:8])
    for issue in record.warnings + record.errors:
        print(issue.residue, issue.position)

single = biors.tokenize_protein("ACDEFGHIK")
print(single.tokens)

dna = biors.tokenize_fasta_records(">dna\nACGT\n", profile="dna-iupac")
rna = biors.tokenize_protein("ACGU", profile="rna-iupac")
```

### `tokenize_fasta_records(fasta_text: str, profile="protein-20") -> list[TokenizedProtein]`

Parses and tokenizes all FASTA records with the selected tokenizer profile. The
default profile is `protein-20`; DNA/RNA workflows can pass `dna-iupac`,
`dna-iupac-special`, `rna-iupac`, or `rna-iupac-special`. Each returned record
preserves tokenization diagnostics. Ambiguous residues are reported in
`warnings`, invalid residues are reported in `errors`, and `valid` is `False`
when either list is non-empty.

### `tokenize_protein(sequence: str, id="user", profile="protein-20") -> TokenizedProtein`

Tokenizes one in-memory sequence with the selected tokenizer profile. The input
is normalized like FASTA-backed tokenization: whitespace is removed and residues
are uppercased before tokenization. The default returned record id is `"user"`.

## Model Input

```python
model_input = biors.build_model_inputs_checked(
    tokenized,
    max_length=512,
    pad_token_id=0,
    padding="fixed_length",
)

first = model_input.records[0]
print(first.id, len(first.input_ids), len(first.attention_mask), first.truncated)
```

### `build_model_inputs_checked(tokenized, max_length, pad_token_id=0, padding="no_padding") -> ModelInput`

Builds model-ready token arrays from tokenized sequence records. Records with
unresolved tokenization warnings or errors are rejected with `ValueError`
instead of being silently converted into model input.

Parameters:

- `tokenized`: `list[TokenizedProtein]`
- `max_length`: positive maximum token count per record
- `pad_token_id`: token id used when fixed-length padding is enabled
- `padding`: `"no_padding"` or `"fixed_length"`

`"no_padding"` truncates to `max_length` and preserves each record's resulting
length. `"fixed_length"` pads every record to `max_length` and sets padding
positions to `0` in `attention_mask`.

## End-To-End Workflow

```python
output = biors.prepare_workflow_from_fasta(
    fasta_text,
    max_length=512,
    pad_token_id=0,
    padding="fixed_length",
)

dna_output = biors.prepare_workflow_from_fasta(
    ">dna\nACGT\n",
    max_length=512,
    profile="dna-iupac",
)

print(output.model_ready)
print(output.report_json)
print(output.records[0].input_ids[:8])
```

### `prepare_workflow(input_hash, records, max_length, pad_token_id=0, padding="no_padding", profile="protein-20") -> SequenceWorkflowOutput`

Runs validation, tokenization, and model-input workflow for records already
parsed by `parse_fasta_records`. The default profile is `protein-20`; pass
`dna-iupac` or `rna-iupac` for nucleotide model-input workflows. `input_hash`
must match `fnv1a64:<16 lowercase hex>`; prefer `prepare_workflow_from_fasta`
when Python has the original FASTA text.

### `prepare_workflow_from_fasta(fasta_text, max_length, pad_token_id=0, padding="no_padding", profile="protein-20") -> SequenceWorkflowOutput`

Parses FASTA text and computes the stable workflow input hash internally before
running the selected profile workflow. Prefer this API for notebook workflows
unless you already have a trusted input hash from the exact FASTA bytes.

The Python output keeps compact convenience fields and the full workflow report:

- `model_ready`: `True` when all records can be converted into model input
- `input_hash`: stable FASTA input hash carried in workflow provenance
- `records`: model-input records when ready, or an empty list when unresolved
  validation/tokenization issues block model input
- `report_json`: the complete core workflow payload as JSON, including
  validation, tokenization, readiness issues, model-input policy, provenance,
  and reproducibility hashes

## Package And Runtime Planning

These functions return JSON strings because package validation and runtime
bridge reports are schema-rich compatibility documents.

```python
import json

report = json.loads(biors.validate_package_manifest(manifest_json))
artifact_report = json.loads(
    biors.validate_package_manifest_artifacts(manifest_json, "/path/to/package")
)
file_report = json.loads(biors.validate_package_manifest_file("/path/to/package/manifest.json"))
summary = json.loads(biors.inspect_package_manifest(manifest_json))
bridge = json.loads(biors.plan_runtime_bridge(manifest_json))

print(summary["name"])
print(report["valid"])
print(artifact_report["valid"])
print(file_report["valid"])
print(bridge["contract_ready"])
print(bridge["artifact_checked"])
print(bridge["execution_ready"])
```

### `inspect_package_manifest(manifest_json: str) -> str`

Parses a package manifest JSON document and returns the compact inspect summary
as a JSON string.

### `validate_package_manifest(manifest_json: str) -> str`

Parses a package manifest JSON document and returns the package validation report
as a JSON string. This is field-only validation for the manifest JSON document;
it does not read package artifacts, verify checksums, or validate
manifest-relative paths on disk.

### `validate_package_manifest_artifacts(manifest_json: str, base_dir: str) -> str`

Parses a package manifest JSON document and validates artifact paths,
checksums, declared layout placement, and tokenizer/vocab artifact content
relative to `base_dir`. Returns the package validation report as a JSON string.

### `validate_package_manifest_file(manifest_path: str) -> str`

Reads a manifest from `manifest_path` and validates package artifacts relative
to the manifest's parent directory. Returns the package validation report as a
JSON string.

### `plan_runtime_bridge(manifest_json: str) -> str`

Parses a package manifest JSON document and returns the runtime bridge
compatibility report as a JSON string.

## Notebook Pattern

For notebook work, keep the boundary explicit:

```python
rows = [
    {
        "id": record.id,
        "valid": record.valid,
        "length": record.length,
        "tokens": record.tokens,
        "warnings": [(issue.residue, issue.position) for issue in record.warnings],
        "errors": [(issue.residue, issue.position) for issue in record.errors],
    }
    for record in biors.tokenize_fasta_records(fasta_text)
]
```

The package does not depend on notebook or tensor libraries. Convert
`input_ids` and `attention_mask` to caller-owned tensor or array types in
notebook code when needed.

## Related Documents

- [Rust API](rust-api.md)

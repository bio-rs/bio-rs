# Python API Reference

`biors-python` exposes the core bio-rs FASTA, validation, tokenization, model
input, package-manifest, and runtime-planning surfaces through PyO3.

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
cd packages/rust/biors-python
maturin develop
pytest
```

## Data Classes

The current Python boundary intentionally returns small immutable PyO3 classes.
For full schema-rich JSON reports, use the Rust CLI or core crate.

| Type | Attributes |
| --- | --- |
| `ProteinSequence` | `id: str`, `sequence: str` |
| `SequenceValidationReport` | `records: int`, `valid_records: int`, `warning_count: int`, `error_count: int` |
| `TokenizedProtein` | `id: str`, `tokens: list[int]`, `length: int` |
| `ModelInput` | `records: list[ModelInputRecord]` |
| `ModelInputRecord` | `id: str`, `input_ids: list[int]`, `attention_mask: list[int]`, `truncated: bool` |
| `SequenceWorkflowOutput` | `model_ready: bool`, `records: list[ModelInputRecord]` |

Errors are raised as `ValueError` for malformed FASTA, invalid padding policy,
invalid package JSON, or model-input records that are not model-ready.

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
print(report.valid_records, report.records)
print(report.warning_count, report.error_count)
```

### `validate_fasta_input(fasta_text: str) -> SequenceValidationReport`

Returns a compact validation summary for FASTA input. The current Python class
does not expose per-residue diagnostics; use the Rust CLI JSON output when a
notebook needs residue-level warnings and errors.

## Tokenization

```python
tokenized = biors.tokenize_fasta_records(fasta_text)
for record in tokenized:
    print(record.id, record.length, record.tokens[:8])

single = biors.tokenize_protein("ACDEFGHIK")
print(single.tokens)
```

### `tokenize_fasta_records(fasta_text: str) -> list[TokenizedProtein]`

Parses and tokenizes all FASTA records with the default `protein-20` tokenizer.

### `tokenize_protein(sequence: str) -> TokenizedProtein`

Tokenizes one protein sequence. The returned record id is `"user"`.

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

Builds model-ready token arrays from tokenized protein records.

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
records = biors.parse_fasta_records(fasta_text)
output = biors.prepare_workflow(
    input_hash="sha256:example",
    records=records,
    max_length=512,
    pad_token_id=0,
    padding="fixed_length",
)

print(output.model_ready)
print(output.records[0].input_ids[:8])
```

### `prepare_workflow(input_hash, records, max_length, pad_token_id=0, padding="no_padding") -> SequenceWorkflowOutput`

Runs the standard protein validation, tokenization, and model-input workflow for
records already parsed by `parse_fasta_records`.

The compact Python output exposes:

- `model_ready`: `True` when all records can be converted into model input
- `records`: model-input records when ready, or an empty list when unresolved
  validation/tokenization issues block model input

## Package And Runtime Planning

These functions return JSON strings because package validation and runtime
bridge reports are schema-rich compatibility documents.

```python
import json

report = json.loads(biors.validate_package_manifest(manifest_json))
bridge = json.loads(biors.plan_runtime_bridge(manifest_json))

print(report["valid"])
print(bridge["compatible"])
```

### `validate_package_manifest(manifest_json: str) -> str`

Parses a package manifest JSON document and returns the package validation report
as a JSON string.

### `plan_runtime_bridge(manifest_json: str) -> str`

Parses a package manifest JSON document and returns the runtime bridge
compatibility report as a JSON string.

## Notebook Pattern

For Jupyter or pandas-heavy work, keep the boundary explicit:

```python
rows = [
    {
        "id": record.id,
        "length": record.length,
        "tokens": record.tokens,
    }
    for record in biors.tokenize_fasta_records(fasta_text)
]
```

The package does not currently depend on NumPy. Convert `input_ids` and
`attention_mask` to NumPy arrays in notebook code when needed.

## Related Documents

- [Python Interop](python-interop.md)
- [Rust API](rust-api.md)
- [Phase 7 Status](phase7-status.md)

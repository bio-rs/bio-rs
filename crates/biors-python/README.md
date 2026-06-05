# biors-python

Python bindings for [bio-rs](https://github.com/bio-rs/bio-rs) via PyO3.

## Installation

```bash
pip install biors
```

Requires Python 3.9+.

## Quickstart

```python
import json

import biors

fasta_text = (
    ">seq1\nACDEFGHIKLMNPQRSTVWY\n"
    ">seq2\nMKWVTFISLLFLFSSAYSRGVFRRDAHKSEVAHRFKDLGEENFKALVLIAFAQYLQQCP\n"
)

# Parse FASTA text
records = biors.parse_fasta_records(fasta_text)

# Validate
report = biors.validate_fasta_input(fasta_text)
rna_report = biors.validate_fasta_input_with_kind(">rna\nACGUN\n", "rna")
print(f"Valid records: {report.valid_records}/{report.records}")
for record in report.sequences:
    for issue in record.warnings:
        print(f"{record.id}: ambiguous residue {issue.residue} at {issue.position}")
    for issue in record.errors:
        print(f"{record.id}: invalid residue {issue.residue} at {issue.position}")

# Tokenize
tokenized = biors.tokenize_fasta_records(fasta_text)
for t in tokenized:
    print(t.id, t.alphabet, t.valid, t.tokens)
    for issue in t.warnings + t.errors:
        print(issue.residue, issue.position)

# Tokenize one in-memory sequence with a caller-provided ID
single = biors.tokenize_protein("ACDEFG", id="sample-001")
print(single.id, single.tokens)

# DNA/RNA use explicit tokenizer profiles.
dna = biors.tokenize_fasta_records(">dna\nACGT\n", profile="dna-iupac")
rna_workflow = biors.prepare_workflow_from_fasta(
    ">rna\nACGU\n",
    max_length=8,
    profile="rna-iupac",
)

# Build model input
model_input = biors.build_model_inputs_checked(
    tokenized,
    max_length=512,
    pad_token_id=0,
    padding="fixed_length",
)
for r in model_input.records:
    print(r.id, r.input_ids, r.attention_mask)

# End-to-end workflow
output = biors.prepare_workflow_from_fasta(
    fasta_text,
    max_length=512,
    pad_token_id=0,
    padding="fixed_length",
)
print(f"Model ready: {output.model_ready}")
print(output.report_json)

# Validate a package on disk, including artifact paths and checksums.
package_report = json.loads(
    biors.validate_package_manifest_file("./protein-package/manifest.json")
)
print(package_report["valid"])
```

`validate_package_manifest(manifest_json)` is field-only validation for an
in-memory manifest. Use `validate_package_manifest_artifacts(manifest_json,
base_dir)` or `validate_package_manifest_file(manifest_path)` when package
files, checksums, and layout placement must be verified.

Errors raised by core parsing, model-input, and package helpers are
`BioRsError` values with stable `code`, `message`, and optional `location`
attributes, while still subclassing `ValueError`.

## Development

Build locally with maturin:

```bash
cd crates/biors-python
maturin develop
pytest
```

## License

MIT OR Apache-2.0

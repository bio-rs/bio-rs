# biors-python

Python bindings for [bio-rs](https://github.com/bio-rs/bio-rs) via PyO3.

## Installation

```bash
pip install biors
```

Requires Python 3.9+.

## Quickstart

```python
import biors

fasta_text = (
    ">seq1\nACDEFGHIKLMNPQRSTVWY\n"
    ">seq2\nMKWVTFISLLFLFSSAYSRGVFRRDAHKSEVAHRFKDLGEENFKALVLIAFAQYLQQCP\n"
)

# Parse FASTA text
records = biors.parse_fasta_records(fasta_text)

# Validate
report = biors.validate_fasta_input(fasta_text)
print(f"Valid records: {report.valid_records}/{report.records}")

# Tokenize
tokenized = biors.tokenize_fasta_records(fasta_text)
for t in tokenized:
    print(t.id, t.alphabet, t.valid, t.tokens)
    for issue in t.warnings + t.errors:
        print(issue.residue, issue.position)

# Tokenize one in-memory sequence with a caller-provided ID
single = biors.tokenize_protein("ACDEFG", id="sample-001")
print(single.id, single.tokens)

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
```

## Development

Build locally with maturin:

```bash
cd packages/rust/biors-python
maturin develop
pytest
```

## License

MIT OR Apache-2.0

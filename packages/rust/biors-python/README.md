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

# Parse FASTA text
records = biors.parse_fasta_records(
    ">seq1\nACDEFGHIKLMNPQRSTVWY\n>seq2\nMKWVTFISLLFLFSSAYSRGVFRRDAHKSEVAHRFKDLGEENFKALVLIAFAQYLQQCP"
)

# Validate
report = biors.validate_fasta_input(fasta_text)
print(f"Valid records: {report.valid_records}/{report.records}")

# Tokenize
tokenized = biors.tokenize_fasta_records(fasta_text)
for t in tokenized:
    print(t.id, t.tokens)

# Build model input
model_input = biors.build_model_inputs_checked(tokenized, max_length=512)
for r in model_input.records:
    print(r.id, r.input_ids, r.attention_mask)

# End-to-end workflow
output = biors.prepare_workflow(
    input_hash="sha256:abc123",
    records=records,
    max_length=512,
)
print(f"Model ready: {output.model_ready}")
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

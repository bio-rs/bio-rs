# Python Interop

bio-rs supports Python interoperability through two paths:

1. **JSON boundary** (stable today) --- use `biors` CLI to produce JSON, then
   adapt in Python.
2. **PyO3 bindings** (planned for v0.44.0) --- direct `pip install biors` with
   native Rust performance.

## Stable JSON Boundary

Use `biors workflow`, `biors pipeline`, or `biors debug` to produce
machine-readable JSON, then adapt that JSON in Python. This keeps Rust
preprocessing deterministic while allowing notebooks, pandas, NumPy, PyTorch,
and Hugging Face workflows to choose their own runtime dependencies.

## Examples

- `examples/python/reference_preprocess.py` reproduces the
  `protein-20-special` preprocessing fixture without dependencies.
- `examples/python/esm_from_biors_json.py` converts model-ready bio-rs JSON into
  `input_ids` and `attention_mask` lists suitable for ESM-style tensor code.
- `examples/python/protbert_from_biors_json.py` converts `biors debug` output
  into whitespace-separated amino acid strings used by common ProtBERT
  preprocessing examples.
- `examples/python/pandas_numpy_friendly.py` converts model-ready bio-rs JSON
  into row dictionaries and column arrays that can be passed to pandas or NumPy.

## pandas And NumPy Convention

Model-ready records expose:

- `id`: stable record identifier
- `input_ids`: fixed-length or unpadded integer list
- `attention_mask`: integer list aligned with `input_ids`
- `truncated`: boolean flag for max-length truncation

The convention is intentionally plain JSON so downstream code can use:

```python
rows = table_rows(payload)
# pandas.DataFrame(rows)
# numpy.asarray([row["input_ids"] for row in rows], dtype="uint8")
```

## PyO3 Bindings (Planned)

The Python API surface is documented in [docs/python-api.md](python-api.md).
It covers:

- `pip install biors`
- FASTA parsing, validation, tokenization, model input, workflow
- Package manifest inspection and runtime bridge planning
- NumPy-compatible output
- Type stubs for IDE support
- Jupyter notebook examples

The actual `biors` PyPI package is planned for v0.44.0.
The v0.43.0 release documents the intended API surface as a design contract.

## Reference Model Notes

ESM and ProtBERT each have model-specific tokenizer expectations. The examples
show safe JSON adaptation patterns, not a claim that bio-rs token IDs are a
drop-in replacement for every model vocabulary. Confirm the target model
contract before inference.

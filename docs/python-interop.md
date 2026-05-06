# Python Interop

bio-rs keeps Python interoperability at the JSON boundary for now. There is no
PyO3 binding in this phase.

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

## Reference Model Notes

ESM and ProtBERT each have model-specific tokenizer expectations. The examples
show safe JSON adaptation patterns, not a claim that bio-rs token IDs are a
drop-in replacement for every model vocabulary. Confirm the target model
contract before inference.

## No PyO3

No PyO3 binding is included. Keeping the boundary at JSON avoids Python ABI
packaging complexity while the core Rust contracts are still maturing.

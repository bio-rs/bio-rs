# biors-python PyO3 Binding Design Document

## Overview

`biors-python` is a PyO3-based Python binding crate for `biors-core`. It exposes
the Rust preprocessing engine to Python without duplicating logic. The crate is
separate from `biors-core` so the default Rust build stays dependency-light.

## Design Principles

1. **Zero duplication**: All domain logic stays in Rust; Python is a thin wrapper.
2. **NumPy-native**: Outputs are `numpy.ndarray` or `pandas.DataFrame`-ready.
3. **Pythonic API**: Class-based, context managers, and `__repr__` where natural.
4. **Error transparency**: Rust errors map to standard Python exceptions.
5. **Optional**: The crate does not affect `biors-core` or `biors` default builds.

## Crate Structure

```
packages/rust/biors-python/
├── Cargo.toml
├── pyproject.toml          # Maturin build configuration
├── src/
│   ├── lib.rs              # PyO3 module init
│   ├── fasta.rs            # FASTA parsing and validation
│   ├── tokenize.rs         # Tokenization
│   ├── model_input.rs      # Model input generation
│   ├── workflow.rs         # End-to-end workflow
│   ├── package.rs          # Package manifest inspection
│   └── errors.rs           # Error-to-exception mapping
├── tests/
│   └── test_python_api.py  # pytest integration tests
├── examples/
│   ├── basic_workflow.ipynb
│   └── pytorch_integration.ipynb
└── .github/workflows/
    └── python-release.yml  # CI for PyPI publishing
```

## Cargo.toml

```toml
[package]
name = "biors-python"
version = "0.44.0"
edition = "2021"
license = "MIT OR Apache-2.0"
repository = "https://github.com/bio-rs/bio-rs"

[lib]
name = "biors"
crate-type = ["cdylib"]

[dependencies]
pyo3 = { version = "0.24", features = ["extension-module"] }
pyo3-numpy = "0.24"
numpy = "0.24"
biors-core = { path = "../biors-core", version = "0.44.0" }
serde_json = "1.0"

[features]
default = []

[package.metadata.maturin]
name = "biors"
```

## Python API Surface

### Module: `biors.fasta`

```python
import biors

# Parse FASTA from file
records = biors.fasta.parse("examples/protein.fasta")
for record in records:
    print(record.id, record.sequence)

# Validate
report = biors.fasta.validate("examples/protein.fasta", kind="auto")
print(report.valid)          # bool
print(report.issues)         # list of dict

# Validate with per-record kind detection
report = biors.fasta.validate_reader(file_handle, kind="auto")
```

### Module: `biors.tokenize`

```python
import biors
import numpy as np

# Tokenize protein sequences
tokenizer = biors.tokenize.ProteinTokenizer(profile="protein-20-special")
result = tokenizer.tokenize(records)

# NumPy arrays
input_ids = result.input_ids       # np.ndarray, shape (n_records, max_len)
attention_mask = result.attention_mask  # np.ndarray
ids = result.ids                    # list of str

# Convert existing vocab JSON
config = biors.tokenize.load_config("path/to/config.json")
```

### Module: `biors.model_input`

```python
import biors

builder = biors.model_input.Builder(max_length=512, padding="max_length")
model_input = builder.build(tokenized_records)

# Returns ModelInput with:
#   .records -> list of ModelInputRecord
#   .input_ids -> np.ndarray
#   .attention_mask -> np.ndarray
#   .truncated -> np.ndarray (bool)
```

### Module: `biors.workflow`

```python
import biors

# One-shot end-to-end
output = biors.workflow.run(
    fasta_path="examples/protein.fasta",
    tokenizer_profile="protein-20-special",
    max_length=512,
)

# Returns WorkflowOutput with JSON-serializable dict
print(output.to_dict())
```

### Module: `biors.package`

```python
import biors

manifest = biors.package.load_manifest("examples/protein-package/manifest.json")
summary = biors.package.inspect(manifest)
print(summary.name, summary.runtime_backend)

# Runtime bridge report
report = biors.package.plan_runtime_bridge(manifest)
print(report.ready)
print(report.backend_config.backend_id)
```

## Error Mapping

| Rust Error | Python Exception |
|---|---|
| `io::Error` | `OSError` |
| `serde_json::Error` | `ValueError` |
| `BackendExecutionError` | `RuntimeError` |
| Validation errors | `ValueError` |
| Tokenization errors | `ValueError` |

## NumPy Output Strategy

Use `pyo3-numpy` to return `numpy.ndarray` from Rust:

```rust
use numpy::PyArray2;
use pyo3::prelude::*;

#[pyfunction]
fn input_ids_as_numpy<'py>(py: Python<'py>, model_input: &ModelInput) -> &'py PyArray2<u32> {
    let ids: Vec<Vec<u32>> = model_input.records.iter()
        .map(|r| r.input_ids.clone())
        .collect();
    PyArray2::from_vec2(py, &ids).unwrap()
}
```

For variable-length sequences, return a list of 1D arrays rather than padding to
a 2D array, unless `padding="max_length"` is requested.

## Build and Packaging Strategy

Use **maturin** for cross-platform builds:

```bash
# Local build
maturin develop

# Build wheels for all platforms
maturin build --release

# Publish to PyPI
maturin publish
```

### CI Matrix (GitHub Actions)

```yaml
strategy:
  matrix:
    os: [ubuntu-latest, macos-latest, windows-latest]
    python-version: ["3.9", "3.10", "3.11", "3.12"]
```

Jobs:
1. `test`: Build and run pytest on all matrix combinations.
2. `sdist`: Build source distribution.
3. `wheels`: Build platform-specific wheels (manylinux, macOS universal, Windows).
4. `publish`: Upload to PyPI on tagged releases.

## Jupyter Notebook Examples

### `basic_workflow.ipynb`

Demonstrates:
- FASTA parsing from Python
- Tokenization
- Model input generation
- Output inspection with pandas

### `pytorch_integration.ipynb`

Demonstrates:
- bio-rs preprocessing → PyTorch tensor
- Hugging Face tokenizer config conversion
- Batching with PyTorch DataLoader

## Implementation Order

1. `Cargo.toml` + `pyproject.toml` + CI skeleton
2. `src/errors.rs` — exception mapping
3. `src/fasta.rs` — parse + validate
4. `src/tokenize.rs` — tokenizer wrapper
5. `src/model_input.rs` — model input builder
6. `src/workflow.rs` — end-to-end workflow
7. `src/package.rs` — manifest inspection
8. `tests/test_python_api.py` — pytest suite
9. `examples/*.ipynb` — Jupyter notebooks
10. `README.md` — Python-specific docs

## Dependencies

- `pyo3` >= 0.24
- `pyo3-numpy` >= 0.24
- `biors-core` (workspace path)
- Python 3.9+
- NumPy (runtime dependency for users)

## Notes

- The crate does not bundle pretrained models or tokenizers.
- CUDA/Metal Candle features are out of scope for the initial release.
- Keep the Python API surface minimal; advanced users can fall back to JSON
  boundary via `biors` CLI.

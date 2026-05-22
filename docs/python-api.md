# Python API Reference

bio-rs exposes its core preprocessing engine to Python through PyO3 bindings.
This document describes the intended API surface for the `biors` Python package.

> **Status:** The `biors-python` crate is planned for v0.44.0 implementation.
> This document is a design contract for v0.43.0. It defines the API boundary
> that the PyO3 bindings will implement. Until v0.44.0 ships, use the
> [JSON boundary](python-interop.md) for Python interoperability.

## Installation

```bash
pip install biors
```

Requirements:

- Python 3.9 or later
- NumPy (optional, for array conversion)

The package publishes a single abi3 wheel compatible with Python 3.9+.
No Rust toolchain is needed on the install side.

## API Overview

The `biors` module exposes pure computation functions and plain data classes.
All types are immutable (`frozen` pyclasses) and thread-safe. Errors map to
standard Python exceptions:

- `ValueError` for validation and tokenization errors
- `OSError` for filesystem I/O errors
- `RuntimeError` for backend execution errors

### Module Summary

| Module | Key Functions |
|--------|---------------|
| `biors` (top-level) | `parse_fasta_records`, `validate_fasta_input`, `tokenize_fasta_records`, `tokenize_protein`, `build_model_inputs_checked`, `prepare_workflow` |
| `biors` (package) | `PackageManifest`, `validate_package_manifest`, `read_package_file`, `plan_runtime_bridge` |

## Top-Level Functions

### `biors.parse_fasta_records(fasta_text: str) -> list[ProteinSequence]`

Parse a FASTA string into a list of `ProteinSequence` records.

```python
import biors

fasta = """>sp|P01308|INS_HUMAN Insulin OS=Homo sapiens OX=9606 GN=INS PE=1 SV=1
MALWMRLLPLLALLALWGPDPAAAFVNQHLCGSHLVEALYLVCGERGFFYTPKT
>sp|P68871|HBB_HUMAN Hemoglobin subunit beta OS=Homo sapiens OX=9606 GN=HBB PE=1 SV=2
MVHLTPEEKSAVTALWGKVNVDEVGGEALGRLLVVYPWTQRFFESFGDLSTPDAVMGNPKVKAHGKKVLGAFSDGLAHLDNLKGTFATLSELHCDKLHVDPENFRLLGNVLVCVLAHHFGKEFTPPVQAAYQKVVAGVANALAHKYH
"""

records = biors.parse_fasta_records(fasta)
for rec in records:
    print(rec.id, len(rec.sequence))
```

**Returns:**
A list of `ProteinSequence` objects. Each object has:

- `id` (`str`): the record header identifier
- `sequence` (`str`): the normalized sequence string
- `kind` (`str`): detected sequence kind (`"protein"`, `"dna"`, or `"rna"`)

**Raises:**
`ValueError` if the FASTA text is malformed.

---

### `biors.validate_fasta_input(fasta_text: str) -> SequenceValidationReport`

Validate FASTA text and return a structured report with per-record diagnostics.

```python
report = biors.validate_fasta_input(fasta)
print(f"Records: {report.records}, Valid: {report.valid_records}")
print(f"Warnings: {report.warning_count}, Errors: {report.error_count}")

for seq in report.sequences:
    print(f"  {seq.id}: valid={seq.valid}, kind={seq.kind}")
    for issue in seq.errors:
        print(f"    Error at position {issue.position}: {issue.symbol} -> {issue.message}")
```

**Returns:**
A `SequenceValidationReport` with these attributes:

- `records` (`int`): total number of records parsed
- `valid_records` (`int`): number of records that passed validation
- `warning_count` (`int`): total warnings across all records
- `error_count` (`int`): total errors across all records
- `kind_counts` (`dict[str, int]`): counts by kind (`protein`, `dna`, `rna`)
- `sequences` (`list[ValidatedSequence]`): per-record detail

Each `ValidatedSequence` has:

- `id` (`str`), `sequence` (`str`), `kind` (`str`), `alphabet` (`str`), `valid` (`bool`)
- `warnings` (`list[SequenceIssue]`)
- `errors` (`list[SequenceIssue]`)

Each `SequenceIssue` has:

- `symbol` (`str`): the problematic residue character
- `position` (`int`): 1-based position in the sequence
- `kind` (`str`): sequence kind
- `code` (`str`): issue code (`"ambiguous_symbol"` or `"invalid_symbol"`)
- `message` (`str`): human-readable description

---

### `biors.tokenize_fasta_records(fasta_text: str) -> list[TokenizedProtein]`

Parse and tokenize all records in a FASTA string using the default
`protein-20` tokenizer.

```python
tokenized = biors.tokenize_fasta_records(fasta)
for t in tokenized:
    print(t.id, t.tokens[:10])
```

**Returns:**
A list of `TokenizedProtein` objects with:

- `id` (`str`): record identifier
- `length` (`int`): number of residues
- `alphabet` (`str`): tokenizer profile name
- `valid` (`bool`): whether the record passed validation
- `tokens` (`list[int]`): token ID list (0-19 for protein-20)
- `warnings` (`list[ResidueIssue]`)
- `errors` (`list[ResidueIssue]`)

Each `ResidueIssue` has:

- `residue` (`str`): the residue character
- `position` (`int`): 1-based position

---

### `biors.tokenize_protein(sequence: str) -> TokenizedProtein`

Tokenize a single protein sequence string.

```python
t = biors.tokenize_protein("MALWMRLLPLLALLALWGPDPAAA")
print(t.tokens)
# [12, 0, 10, 18, 12, 16, 10, 10, 13, 10, 10, 10, 0, 10, 10, 18, 6, 15, 3, 15, 0, 0, 0]
```

**Returns:**
A single `TokenizedProtein`.

---

### `biors.build_model_inputs_checked(tokenized: list[TokenizedProtein], policy: ModelInputPolicy) -> ModelInput`

Build model-ready inputs from tokenized records with safety checks.

```python
policy = biors.ModelInputPolicy(
    max_length=512,
    pad_token_id=20,
    padding="fixed_length",
)

model_input = biors.build_model_inputs_checked(tokenized, policy)
print(f"Records: {len(model_input.records)}")
for rec in model_input.records:
    print(rec.id, rec.truncated)
```

**Parameters:**

- `tokenized`: list of `TokenizedProtein` objects
- `policy`: a `ModelInputPolicy` instance

**Returns:**
A `ModelInput` with:

- `policy` (`ModelInputPolicy`): the policy used
- `records` (`list[ModelInputRecord]`): per-record model input

Each `ModelInputRecord` has:

- `id` (`str`)
- `input_ids` (`list[int]`): token IDs, padded or truncated to `max_length`
- `attention_mask` (`list[int]`): `1` for real tokens, `0` for padding
- `truncated` (`bool`): whether the sequence was truncated

**Raises:**
`ValueError` if the tokenized list contains unresolved residues that would
produce an unsafe model input.

---

### `biors.prepare_workflow(input_hash: str, records: list[ProteinSequence], policy: ModelInputPolicy) -> SequenceWorkflowOutput`

Run the full end-to-end workflow: validation, tokenization, model input
building, and provenance generation.

```python
records = biors.parse_fasta_records(fasta)
policy = biors.ModelInputPolicy(max_length=512, pad_token_id=20, padding="fixed_length")
output = biors.prepare_workflow("fnv1a64:abc123...", records, policy)

print(output.model_ready)
print(output.provenance.biors_core_version)
print(output.provenance.input_hash)
```

**Returns:**
A `SequenceWorkflowOutput` with:

- `workflow` (`str`): workflow identifier (`"protein_model_input.v0"`)
- `model_ready` (`bool`): whether all records are ready for model inference
- `provenance` (`SequenceWorkflowProvenance`): reproducibility metadata
- `validation` (`ValidationSummary`): validation results
- `tokenization` (`TokenizationSummary`): tokenization results
- `model_input` (`ModelInput | None`): the built model input, or `None` if not ready
- `readiness_issues` (`list[ReadinessIssue]`): blocking issues if `model_ready` is `False`

The `SequenceWorkflowProvenance` has:

- `biors_core_version` (`str`)
- `invocation` (`dict`): command and arguments
- `input_hash` (`str`): FNV-1a64 hash of the input
- `normalization` (`str`): normalization strategy used
- `validation_alphabet` (`str`)
- `tokenizer` (`dict`): tokenizer name, vocab size, unknown token ID, policy
- `model_input_policy` (`dict`): the policy settings
- `hashes` (`dict`): vocabulary and output data SHA-256 hashes

---

## Package Manifest API

### `biors.PackageManifest`

A frozen PyO3 class representing a bio-rs package manifest (v1 schema).

```python
manifest = biors.PackageManifest(
    schema_version="biors.package.v1",
    name="my-protein-model",
    package_layout={...},
    metadata={...},
    model={...},
    preprocessing=[...],
    postprocessing=[...],
    runtime={...},
    fixtures=[...],
)
```

**Attributes:**

- `schema_version` (`str`): always `"biors.package.v1"`
- `name` (`str`): package name
- `package_layout` (`dict`): directory layout with keys `manifest`, `models`, `tokenizers`, `vocabs`, `pipelines`, `fixtures`, `observed`, `docs`
- `metadata` (`dict`): `license`, `citation`, `model_card`
- `model` (`dict`): model artifact with `format`, `path`, optional `checksum` and `metadata`
- `tokenizer` (`dict | None`): tokenizer asset
- `vocab` (`dict | None`): vocabulary asset
- `preprocessing` (`list[dict]`): pipeline steps
- `postprocessing` (`list[dict]`): pipeline steps
- `runtime` (`dict`): `backend`, `target`, optional `version`
- `expected_input` (`dict | None`): shape descriptor
- `expected_output` (`dict | None`): shape descriptor
- `fixtures` (`list[dict]`): test fixtures

---

### `biors.validate_package_manifest(manifest_json: str) -> PackageValidationReport`

Validate a package manifest JSON string.

```python
with open("manifest.json") as f:
    manifest_json = f.read()

report = biors.validate_package_manifest(manifest_json)
print(report.valid)
for issue in report.structured_issues:
    print(issue.code, issue.field, issue.message)
```

**Returns:**
A `PackageValidationReport` with:

- `valid` (`bool`)
- `issues` (`list[str]`): human-readable issue strings
- `structured_issues` (`list[StructuredIssue]`): typed issues with `code`, `field`, `message`

Issue codes include: `required_field`, `missing_fixture`, `invalid_shape`,
`invalid_checksum_format`, `checksum_mismatch`, `invalid_asset_path`,
`asset_read_failed`, `layout_mismatch`.

---

### `biors.read_package_file(base_dir: str, path: str) -> bytes`

Read a file from a package directory. This is the Python-safe equivalent of
`biors_core::package::paths::read_package_file`.

```python
data = biors.read_package_file("examples/protein-package", "models/model.onnx")
print(len(data))
```

**Returns:**
`bytes` containing the file contents.

**Raises:**
`OSError` if the file does not exist or cannot be read.

---

### `biors.plan_runtime_bridge(manifest_json: str) -> RuntimeBridgeReport`

Plan a runtime bridge from a package manifest. This returns a report that
describes whether the local environment can run the model described in the
manifest.

```python
report = biors.plan_runtime_bridge(manifest_json)
print(report.ready)
print(report.backend)
print(report.target)
print(report.model_format)
print(report.blocking_issues)

for check in report.compatibility_checks:
    print(check.code, check.passed, check.message)
```

**Returns:**
A `RuntimeBridgeReport` with:

- `ready` (`bool`): whether the environment can execute the model
- `backend` (`str`): backend identifier
- `target` (`str`): runtime target
- `model_format` (`str`): model artifact format
- `model_metadata` (`dict | None`): name, version, architecture, task, source, description
- `backend_config` (`dict`): `backend_id`, `provider`, optional `version` and `model_artifact`
- `execution_provider` (`str`): execution provider name
- `compatibility_checks` (`list[dict]`): each with `code`, `passed`, `message`
- `blocking_issues` (`list[str]`): issues preventing execution
- `backend_capabilities` (`dict | None`): `deterministic`, `supports_batch`, `supports_streaming`, `supported_inputs`, `supported_outputs`
- `benchmark_evidence` (`list[dict] | None`): benchmark records
- `regression_baseline` (`dict | None`): baseline metadata

## NumPy Integration

Token IDs from `biors` are returned as Python `list[int]`. Converting to NumPy
arrays is straightforward and lets you feed directly into PyTorch, TensorFlow,
or JAX workflows.

### Converting token IDs to NumPy arrays

```python
import biors
import numpy as np

fasta = """>sp|P01308|INS_HUMAN
MALWMRLLPLLALLALWGPDPAAA
"""

tokenized = biors.tokenize_fasta_records(fasta)

# List of 1D arrays (variable length)
arrays = [np.array(t.tokens, dtype=np.uint8) for t in tokenized]
print(arrays[0])
# array([12,  0, 10, 18, 12, 16, 10, 10, 13, 10, 10, 10,  0, 10, 10, 18,
#         6, 15,  3, 15,  0,  0,  0], dtype=uint8)

# Stacked 2D array (if all sequences are the same length or padded)
model_input = biors.build_model_inputs_checked(
    tokenized,
    biors.ModelInputPolicy(max_length=512, pad_token_id=20, padding="fixed_length"),
)
input_ids = np.array([r.input_ids for r in model_input.records], dtype=np.uint16)
attention_mask = np.array([r.attention_mask for r in model_input.records], dtype=np.uint8)
print(input_ids.shape)       # (n_records, 512)
print(attention_mask.shape)  # (n_records, 512)
```

### PyTorch tensor conversion

```python
import torch

input_ids_tensor = torch.from_numpy(input_ids)
attention_mask_tensor = torch.from_numpy(attention_mask)
print(input_ids_tensor.dtype)  # torch.int64 or torch.int32 depending on numpy dtype
```

### Performance note

The `biors` Rust core processes FASTA at roughly 200-350M residues per second.
Conversion to NumPy is a shallow copy for simple dtypes. For large batches,
the NumPy conversion overhead is usually negligible compared to model inference.

## Type Stubs

The `biors` package includes a `biors.pyi` stub file for IDE support. This
enables autocompletion, type checking with mypy, and inline documentation in
VS Code, PyCharm, and Jupyter.

### Stub file location

```
biors/
  __init__.py
  biors.pyi
```

### Key stub signatures

```python
from typing import List, Dict, Optional

class ProteinSequence:
    id: str
    sequence: str
    kind: str

class SequenceIssue:
    symbol: str
    position: int
    kind: str
    code: str
    message: str

class ValidatedSequence:
    id: str
    sequence: str
    kind: str
    alphabet: str
    valid: bool
    warnings: List[SequenceIssue]
    errors: List[SequenceIssue]

class SequenceValidationReport:
    records: int
    valid_records: int
    warning_count: int
    error_count: int
    kind_counts: Dict[str, int]
    sequences: List[ValidatedSequence]

class ResidueIssue:
    residue: str
    position: int

class TokenizedProtein:
    id: str
    length: int
    alphabet: str
    valid: bool
    tokens: List[int]
    warnings: List[ResidueIssue]
    errors: List[ResidueIssue]

class ModelInputPolicy:
    max_length: int
    pad_token_id: int
    padding: str

class ModelInputRecord:
    id: str
    input_ids: List[int]
    attention_mask: List[int]
    truncated: bool

class ModelInput:
    policy: ModelInputPolicy
    records: List[ModelInputRecord]

class SequenceWorkflowProvenance:
    biors_core_version: str
    invocation: Dict
    input_hash: str
    normalization: str
    validation_alphabet: str
    tokenizer: Dict
    model_input_policy: Dict
    hashes: Dict

class ReadinessIssue:
    code: str
    id: str
    warning_count: int
    error_count: int
    message: str

class SequenceWorkflowOutput:
    workflow: str
    model_ready: bool
    provenance: SequenceWorkflowProvenance
    validation: Dict
    tokenization: Dict
    model_input: Optional[ModelInput]
    readiness_issues: List[ReadinessIssue]

class PackageValidationReport:
    valid: bool
    issues: List[str]
    structured_issues: List[Dict]

class RuntimeBridgeReport:
    ready: bool
    backend: str
    target: str
    model_format: str
    model_metadata: Optional[Dict]
    backend_config: Dict
    execution_provider: str
    compatibility_checks: List[Dict]
    blocking_issues: List[str]
    backend_capabilities: Optional[Dict]
    benchmark_evidence: Optional[List[Dict]]
    regression_baseline: Optional[Dict]

class PackageManifest:
    schema_version: str
    name: str
    package_layout: Dict
    metadata: Dict
    model: Dict
    tokenizer: Optional[Dict]
    vocab: Optional[Dict]
    preprocessing: List[Dict]
    postprocessing: List[Dict]
    runtime: Dict
    expected_input: Optional[Dict]
    expected_output: Optional[Dict]
    fixtures: List[Dict]

def parse_fasta_records(fasta_text: str) -> List[ProteinSequence]: ...
def validate_fasta_input(fasta_text: str) -> SequenceValidationReport: ...
def tokenize_fasta_records(fasta_text: str) -> List[TokenizedProtein]: ...
def tokenize_protein(sequence: str) -> TokenizedProtein: ...
def build_model_inputs_checked(
    tokenized: List[TokenizedProtein], policy: ModelInputPolicy
) -> ModelInput: ...
def prepare_workflow(
    input_hash: str, records: List[ProteinSequence], policy: ModelInputPolicy
) -> SequenceWorkflowOutput: ...
def validate_package_manifest(manifest_json: str) -> PackageValidationReport: ...
def read_package_file(base_dir: str, path: str) -> bytes: ...
def plan_runtime_bridge(manifest_json: str) -> RuntimeBridgeReport: ...
```

## Complete Working Example

This script reads a FASTA file, validates it, tokenizes the records, builds
model inputs, and prints a summary.

```python
#!/usr/bin/env python3
"""End-to-end bio-rs Python API demo."""

import json
import biors

FASTA = """>sp|P01308|INS_HUMAN Insulin
MALWMRLLPLLALLALWGPDPAAAFVNQHLCGSHLVEALYLVCGERGFFYTPKT
>sp|P68871|HBB_HUMAN Hemoglobin subunit beta
MVHLTPEEKSAVTALWGKVNVDEVGGEALGRLLVVYPWTQRFFESFGDLSTPDAVMGNPKVKAHGKKVLGAFSDGLAHLDNLKGTFATLSELHCDKLHVDPENFRLLGNVLVCVLAHHFGKEFTPPVQAAYQKVVAGVANALAHKYH
"""

def main():
    # Parse
    records = biors.parse_fasta_records(FASTA)
    print(f"Parsed {len(records)} records")

    # Validate
    report = biors.validate_fasta_input(FASTA)
    print(f"Valid: {report.valid_records}/{report.records}")
    if report.error_count > 0:
        print("Errors found; stopping.")
        return

    # Tokenize
    tokenized = biors.tokenize_fasta_records(FASTA)
    for t in tokenized:
        print(f"  {t.id}: {t.length} residues -> {len(t.tokens)} tokens")

    # Build model input
    policy = biors.ModelInputPolicy(
        max_length=512,
        pad_token_id=20,
        padding="fixed_length",
    )
    model_input = biors.build_model_inputs_checked(tokenized, policy)
    print(f"Model input records: {len(model_input.records)}")

    # Full workflow with provenance
    output = biors.prepare_workflow("fnv1a64:demo123", records, policy)
    print(f"Model ready: {output.model_ready}")
    print(f"Core version: {output.provenance.biors_core_version}")
    print(f"Input hash: {output.provenance.input_hash}")

    # Package manifest validation (optional)
    manifest = {
        "schema_version": "biors.package.v1",
        "name": "demo-package",
        "package_layout": {
            "manifest": "manifest.json",
            "models": "models",
            "tokenizers": "tokenizers",
            "vocabs": "vocabs",
            "pipelines": "pipelines",
            "fixtures": "fixtures",
            "observed": "observed",
            "docs": "docs",
        },
        "metadata": {
            "license": {"expression": "MIT"},
            "citation": {"preferred_citation": "Doe et al. 2024"},
            "model_card": {
                "path": "docs/model-card.md",
                "summary": "Demo model",
                "intended_use": ["research"],
                "limitations": ["not for clinical use"],
            },
        },
        "model": {"format": "onnx", "path": "models/model.onnx"},
        "preprocessing": [],
        "postprocessing": [],
        "runtime": {"backend": "onnx-webgpu", "target": "browser-wasm-webgpu"},
        "fixtures": [
            {
                "name": "basic",
                "input": "fixtures/input.fasta",
                "expected_output": "fixtures/expected.json",
            }
        ],
    }
    manifest_json = json.dumps(manifest)
    pkg_report = biors.validate_package_manifest(manifest_json)
    print(f"Manifest valid: {pkg_report.valid}")

    bridge = biors.plan_runtime_bridge(manifest_json)
    print(f"Runtime bridge ready: {bridge.ready}")
    if not bridge.ready:
        for issue in bridge.blocking_issues:
            print(f"  Blocker: {issue}")

if __name__ == "__main__":
    main()
```

## Jupyter Notebook Snippets

These cells work in a standard Jupyter or JupyterLab notebook.

### Cell 1: Installation and import

```python
# In a fresh environment:
# !pip install biors numpy pandas

import biors
import numpy as np
import pandas as pd
```

### Cell 2: Parse and inspect FASTA

```python
fasta = """>sp|P01308|INS_HUMAN Insulin
MALWMRLLPLLALLALWGPDPAAAFVNQHLCGSHLVEALYLVCGERGFFYTPKT
>sp|P68871|HBB_HUMAN Hemoglobin subunit beta
MVHLTPEEKSAVTALWGKVNVDEVGGEALGRLLVVYPWTQRFFESFGDLSTPDAVMGNPKVKAHGKKVLGAFSDGLAHLDNLKGTFATLSELHCDKLHVDPENFRLLGNVLVCVLAHHFGKEFTPPVQAAYQKVVAGVANALAHKYH
"""

records = biors.parse_fasta_records(fasta)
df = pd.DataFrame([{"id": r.id, "length": len(r.sequence), "kind": r.kind} for r in records])
df
```

### Cell 3: Validate and visualize issues

```python
report = biors.validate_fasta_input(fasta)

issue_rows = []
for seq in report.sequences:
    for issue in seq.warnings:
        issue_rows.append({
            "id": seq.id,
            "position": issue.position,
            "symbol": issue.symbol,
            "code": issue.code,
            "severity": "warning",
        })
    for issue in seq.errors:
        issue_rows.append({
            "id": seq.id,
            "position": issue.position,
            "symbol": issue.symbol,
            "code": issue.code,
            "severity": "error",
        })

if issue_rows:
    issues_df = pd.DataFrame(issue_rows)
    display(issues_df)
else:
    print("No issues found.")
```

### Cell 4: Tokenize and convert to NumPy

```python
tokenized = biors.tokenize_fasta_records(fasta)

# Build a DataFrame of token IDs (padded display)
max_len = max(len(t.tokens) for t in tokenized)
token_df = pd.DataFrame(
    [{"id": t.id, "tokens": t.tokens, "length": t.length} for t in tokenized]
)
display(token_df)

# Convert to NumPy for downstream ML
input_ids = np.array([t.tokens for t in tokenized], dtype=object)  # ragged
print("Ragged array shape:", input_ids.shape)
```

### Cell 5: Build model inputs and stack for PyTorch

```python
policy = biors.ModelInputPolicy(max_length=512, pad_token_id=20, padding="fixed_length")
model_input = biors.build_model_inputs_checked(tokenized, policy)

# Stack to 2D arrays
input_ids_2d = np.array([r.input_ids for r in model_input.records], dtype=np.uint16)
attention_mask_2d = np.array([r.attention_mask for r in model_input.records], dtype=np.uint8)
truncated = np.array([r.truncated for r in model_input.records], dtype=bool)

print("input_ids shape:", input_ids_2d.shape)
print("attention_mask shape:", attention_mask_2d.shape)
print("truncated:", truncated)

# Ready for PyTorch
# import torch
# torch_input_ids = torch.from_numpy(input_ids_2d)
```

### Cell 6: Full workflow with provenance

```python
import hashlib

input_hash = "fnv1a64:" + hashlib.sha256(fasta.encode()).hexdigest()[:16]
output = biors.prepare_workflow(input_hash, records, policy)

print("Model ready:", output.model_ready)
print("Core version:", output.provenance.biors_core_version)
print("Tokenizer:", output.provenance.tokenizer["name"])
print("Vocab SHA-256:", output.provenance.hashes["vocabulary_sha256"])

if not output.model_ready:
    for issue in output.readiness_issues:
        print("Readiness issue:", issue.message)
```

### Cell 7: Package manifest inspection

```python
import json

manifest = {
    "schema_version": "biors.package.v1",
    "name": "my-model",
    "package_layout": {
        "manifest": "manifest.json",
        "models": "models",
        "tokenizers": "tokenizers",
        "vocabs": "vocabs",
        "pipelines": "pipelines",
        "fixtures": "fixtures",
        "observed": "observed",
        "docs": "docs",
    },
    "metadata": {
        "license": {"expression": "MIT"},
        "citation": {"preferred_citation": "Demo 2024"},
        "model_card": {
            "path": "docs/model-card.md",
            "summary": "Demo",
            "intended_use": ["research"],
            "limitations": ["not for clinical use"],
        },
    },
    "model": {"format": "onnx", "path": "models/model.onnx"},
    "preprocessing": [],
    "postprocessing": [],
    "runtime": {"backend": "candle", "target": "local-cpu"},
    "fixtures": [
        {"name": "test", "input": "fixtures/in.fasta", "expected_output": "fixtures/out.json"}
    ],
}

report = biors.validate_package_manifest(json.dumps(manifest))
print("Valid:", report.valid)

bridge = biors.plan_runtime_bridge(json.dumps(manifest))
print("Bridge ready:", bridge.ready)
print("Backend:", bridge.backend)
print("Target:", bridge.target)
```

## Design Reference

This API surface is specified in
`docs/superpowers/specs/2026-05-22-phase7-external-interface-0.43-design.md`.
The design document defines the Python-safe subset of `biors-core` and maps
Rust types to Pythonic equivalents.

Key design decisions:

- **String-based FASTA input:** Python passes FASTA text as `str`, not file
  paths or generic readers. This mirrors the WASM-safe subset and avoids
  lifetime complexity at the PyO3 boundary.
- **Immutable data classes:** All exposed types use `#[pyclass(frozen)]` so
  they are safe to share across Python threads.
- **List-of-int tokens:** `Vec<u8>` token IDs convert to Python `list[int]`.
  NumPy conversion is left to the caller so the core binding stays lightweight.
- **Filesystem access enabled:** Unlike the WASM binding, the Python binding
  exposes `read_package_file` because Python has native filesystem access.
- **Error transparency:** Rust `Result` types convert to Python exceptions
  through PyO3's automatic mapping.

## See Also

- [Python Interop](python-interop.md) for the JSON boundary approach (current v0.43.0)
- [WASM API](wasm-api.md) for the browser-safe binding subset
- [Package Format](package-format.md) for manifest schema details
- [CLI Contract](cli-contract.md) for the command-line interface

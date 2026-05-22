# bio-rs v0.43.0 External Interface API Review Design Document

## 1. Executive Summary

biors-core (v0.41.0) is already structurally well-positioned for multi-target bindings. The crate has **zero platform-specific dependencies** (only `serde`, `serde_json`, `sha2`), and the public API surface is composed almost entirely of plain data structs and pure computation functions. The two I/O-heavy modules---`runtime` (external process spawning) and `package::paths` (filesystem reads)---are cleanly isolated and can be gated behind `#[cfg(not(target_arch = "wasm32"))]`.

This document defines:
- A **Rust public API compatibility audit checklist** to ensure binding-safe contracts.
- A **WASM binding boundary** that exposes a browser-safe subset via `wasm-bindgen`.
- A **Python binding boundary** that exposes a `PyO3`-safe subset.
- A **TypeScript definition generation strategy** and **npm package structure**.
- An **external interface documentation plan** with three new docs files.

---

## 2. Rust Public API Compatibility Audit Checklist

### 2.1 Crate Metadata

| Check | Status | Note |
|-------|--------|------|
| `cargo check --target wasm32-unknown-unknown -p biors-core` passes | Current CI | Already enforced in `scripts/check.sh` |
| No `std::fs` in core public API | Partial | `package::paths::read_package_file` uses `std::fs::read` |
| No `std::process` in core public API | Partial | `runtime::ExternalProcessBackend` uses `std::process::Command` |
| No `std::net` anywhere | Pass | None found |
| No `unsafe` in public API | Pass | No `unsafe` blocks in public functions |
| No raw pointers in public types | Pass | All public types use `Vec<u8>`, `String`, `usize`, etc. |
| All public types implement `Serialize + Deserialize` | Pass | Verified across all modules |
| All public error types implement `std::error::Error` | Pass | `BioRsError`, `FastaReadError`, `ModelInputBuildError`, etc. |
| No `tokio` or async runtime dependency | Pass | None in `Cargo.toml` |
| No `wasm-bindgen` or `pyo3` in core deps | Pass | Keeps core dependency-light |

### 2.2 Module-by-Module Safety Audit

#### `fasta` --- WASM-safe (string APIs), Conditional (reader APIs)

**Public types:** `ParsedFastaInput`, `ValidatedFastaInput` --- WASM-safe.

**Public functions:**
- `parse_fasta_records(input: &str)` --- WASM-safe, pure string input.
- `parse_fasta_records_reader<R: BufRead>(reader: R)` --- Needs `BufRead`; WASM can use it with `Cursor<&[u8]>` but not with `std::fs::File`.
- `validate_fasta_input(input: &str)` --- WASM-safe.
- `validate_fasta_reader<R: BufRead>(reader: R)` --- Same as above.

**Binding recommendation:** Expose string-based APIs to WASM and Python. Reader-based APIs are fine for Python (native `BufRead` works) but for WASM should be hidden or wrapped to accept `&[u8]` / `Uint8Array`.

#### `sequence` --- Fully WASM-safe

**Public types:** `ProteinSequence`, `SequenceKind`, `SequenceValidationReport`, `ValidatedSequence`, `ResidueIssue`, `SequenceValidationIssue`, `AlphabetPolicy`, etc.

**Public functions:** `validate_protein_sequence`, `validate_sequence_record`, `normalize_sequence`, `detect_sequence_kind`, etc.

**Binding recommendation:** Expose everything. All types are plain data, all functions are pure computation.

#### `tokenizer` --- WASM-safe (string APIs), Conditional (reader APIs)

**Public types:** `ProteinTokenizer`, `ProteinTokenizerConfig`, `ProteinTokenizerProfile`, `TokenizedProtein`, `TokenizedFastaInput`, etc.

**Public functions:**
- `tokenize_fasta_records(input: &str)` --- WASM-safe.
- `tokenize_fasta_records_reader<R: BufRead>(reader: R)` --- Reader-based.
- `summarize_fasta_records_reader<R: BufRead>(reader)` --- Reader-based.
- `tokenize_protein`, `tokenize_protein_with_config` --- WASM-safe.
- `load_protein_20_vocab`, `protein_20_vocab_tokens` --- WASM-safe.

**Binding recommendation:** Expose string-based and config/vocab APIs. Reader-based APIs should be wrapped or omitted in WASM.

#### `model_input` --- Fully WASM-safe

**Public types:** `ModelInput`, `ModelInputRecord`, `ModelInputPolicy`, `PaddingPolicy`, `ModelInputBuildError`.

**Public functions:** `build_model_inputs_checked`, `build_model_inputs_unchecked`, `validate_model_input_policy`.

**Binding recommendation:** Expose everything. Pure computation, no I/O.

#### `workflow` --- Fully WASM-safe

**Public types:** `SequenceWorkflowOutput`, `SequenceWorkflowProvenance`, `TokenizationWorkflowOutput`, `SequenceWorkflowReadinessIssue`.

**Public functions:** `prepare_protein_model_input_workflow`, `prepare_protein_model_input_workflow_with_invocation`.

**Binding recommendation:** Expose everything. Takes already-in-memory `ProteinSequence` records and `ModelInputPolicy`.

#### `package` --- Mixed (types safe, I/O functions unsafe)

**Public types (all WASM-safe):** `PackageManifest`, `PackageDirectoryLayout`, `PackageMetadata`, `LicenseMetadata`, `CitationMetadata`, `ModelCardMetadata`, `DocumentArtifact`, `ModelArtifact`, `ModelArtifactMetadata`, `PipelineStep`, `RuntimeTarget`, `DataShape`, `TokenAsset`, `PackageFixture`, `PackageValidationReport`, `PackageValidationIssue`, `PackageValidationIssueCode`, `PackageManifestSummary`, `RuntimeBridgeReport`, `BackendCapabilitiesSummary`, etc.

**Public functions:**
- `validate_package_manifest` --- WASM-safe (validates in-memory manifest).
- `inspect_package_manifest` --- WASM-safe (inspects in-memory manifest).
- `validate_package_manifest_artifacts` --- Calls `read_package_file` internally; needs filesystem.
- `plan_runtime_bridge` --- WASM-safe (pure planning logic).
- `compare_package_manifest_schemas`, `convert_package_manifest`, `diff_package_manifests` --- WASM-safe.
- `read_package_file`, `resolve_package_asset_path` --- NOT WASM-safe (`std::fs::read`, `std::path::Path`).

**Binding recommendation:** Expose all manifest types and pure validation/planning functions. Hide `read_package_file` and `resolve_package_asset_path` from WASM. For Python, expose them (Python has filesystem access).

#### `runtime` --- NOT WASM-safe

**Public types:** `Backend`, `BackendCapabilities`, `BackendConfig`, `BackendExecutionError`, `ExecutionContext`, `ExecutionResult`, `ExecutionMetadata`, `RuntimeCompatibilityIssue`, etc.

**Public functions/types:**
- `ExternalProcessBackend` --- NOT WASM-safe. Uses `std::process::Command`, `std::thread::spawn`.
- `ExternalProcessConfig` --- Structurally safe (plain data), but only useful with `ExternalProcessBackend`.

**Binding recommendation:** Hide entire `runtime` module from WASM. For Python, expose `Backend` trait and `ExternalProcessBackend` (Python can spawn processes).

#### `verification` --- Fully WASM-safe

**Public types:** `StableInputHasher`, `OutputDiffReport`, `ContentMismatchDiff`, `FixtureVerificationResult`, `VerificationStatus`, `VerificationIssueCode`.

**Public functions:** `stable_input_hash`, `diff_output_bytes`, `verify_package_outputs`, `verify_package_outputs_with_observation_base`.

**Binding recommendation:** Expose everything. Pure computation.

#### `versioning` --- Fully WASM-safe

**Public types:** `SchemaContractPolicy`, `SupportedSchemaVersion`, `SchemaLifecycleStatus`, `DeprecationPolicy`, etc.

**Public functions:** `package_manifest_policy`, `pipeline_config_policy`, `manifest_schema_compatibility`, `manifest_schema_migration_plan`.

**Binding recommendation:** Expose everything. Pure policy data.

---

## 3. WASM Binding Boundary Design

### 3.1 Design Principles

1. **Browser-safe subset:** No `std::fs`, no `std::process`, no `std::net`, no `std::thread`.
2. **String and bytes at the boundary:** WASM consumers pass FASTA as `String` or `Uint8Array`, not file paths.
3. **DTO-style boundary:** Export small, typed structs and functions. Avoid `JsValue` in real code.
4. **serde for complex data:** Use `serde-wasm-bindgen` for serializing/deserializing complex structs across the boundary.
5. **Typed arrays for bulk data:** `Vec<u8>` maps to `Uint8Array` in JS.

### 3.2 WASM-Safe API Subset

The following biors-core types and functions are **WASM-safe** and should be exported:

**Module: `fasta`**
```rust
#[wasm_bindgen]
pub fn parse_fasta_records(input: &str) -> Result<JsValue, JsValue> {
    let records = biors_core::fasta::parse_fasta_records(input)?;
    Ok(serde_wasm_bindgen::to_value(&records)?)
}

#[wasm_bindgen]
pub fn validate_fasta_input(input: &str) -> Result<JsValue, JsValue> {
    let report = biors_core::fasta::validate_fasta_input(input)?;
    Ok(serde_wasm_bindgen::to_value(&report)?)
}
```

**Module: `sequence`**
- Export all public types: `ProteinSequence`, `SequenceValidationReport`, `ResidueIssue`, `SequenceKind`, etc.
- Export `validate_protein_sequence`.

**Module: `tokenizer`**
- Export `tokenize_fasta_records(input: &str)`.
- Export `tokenize_protein` and `tokenize_protein_with_config`.
- Export `ProteinTokenizerConfig`, `ProteinTokenizerProfile`, `SpecialToken`.
- Export `load_protein_20_vocab`, `protein_20_vocab_tokens`.

**Module: `model_input`**
- Export `ModelInput`, `ModelInputRecord`, `ModelInputPolicy`, `PaddingPolicy`.
- Export `build_model_inputs_checked`, `build_model_inputs_unchecked`.

**Module: `workflow`**
- Export `SequenceWorkflowOutput`, `SequenceWorkflowProvenance`, `TokenizationWorkflowOutput`.
- Export `prepare_protein_model_input_workflow`.

**Module: `package` (manifest-only)**
- Export all manifest types.
- Export `validate_package_manifest`, `inspect_package_manifest`.
- Export `compare_package_manifest_schemas`, `convert_package_manifest`, `diff_package_manifests`.
- Export `plan_runtime_bridge`.
- **Hide:** `read_package_file`, `resolve_package_asset_path`, `validate_package_manifest_artifacts`.

**Module: `verification`**
- Export `StableInputHasher`, `diff_output_bytes`, `verify_package_outputs`.

**Module: `versioning`**
- Export all policy types and functions.

### 3.3 What Must Be Hidden or Wrapped in WASM

| Item | Reason | WASM Strategy |
|------|--------|---------------|
| `fasta::parse_fasta_records_reader` | Requires `BufRead` trait | Hide. Use `parse_fasta_records` with `String` instead. |
| `fasta::validate_fasta_reader` | Requires `BufRead` trait | Hide. Use `validate_fasta_input` with `String` instead. |
| `tokenizer::tokenize_fasta_records_reader` | Requires `BufRead` trait | Hide. Use `tokenize_fasta_records` with `String` instead. |
| `package::read_package_file` | Uses `std::fs::read` | Hide. JS side loads file content as `String`/`Uint8Array`, then passes to WASM. |
| `package::validate_package_manifest_artifacts` | Calls `read_package_file` | Hide. JS side reads artifacts and passes bytes to `verify_package_outputs`. |
| `runtime::ExternalProcessBackend` | Uses `std::process::Command`, `std::thread` | **Hide entire `runtime` module.** |

### 3.4 TypeScript Definition Generation Strategy

**Approach:** Use `wasm-bindgen` native `.d.ts` generation + hand-maintained types for `serde-wasm-bindgen` payloads.

1. **Build step:**
   ```bash
   wasm-pack build --target bundler --out-dir pkg packages/rust/biors-wasm
   ```

2. **For serde-exported types:** Maintain a `types/biors-core.d.ts` file that mirrors the serde-exported structs. Use the JSON schemas in `schemas/` as the source of truth for TypeScript interfaces.

3. **npm package structure:**
   ```
   @bio-rs/core-wasm/
   ├── package.json
   ├── README.md
   ├── index.d.ts
   ├── index.js
   ├── biors_core_wasm.d.ts
   ├── biors_core_wasm.js
   ├── biors_core_wasm_bg.wasm
   └── types/
       └── serde-types.d.ts
   ```

---

## 4. Python Binding Boundary Design

### 4.1 Design Principles

1. **PyO3-safe subset:** All exposed types must be `Send + Sync`, have no lifetime parameters, no generic parameters.
2. **GIL-aware:** Python's GIL requires thread-safe types. biors-core types are already `Send + Sync` (plain data).
3. **Bytes and strings at the boundary:** Accept Python `str` for FASTA text, `bytes` for raw sequences.
4. **NumPy-friendly:** Return token IDs as Python `list[int]` or optionally `numpy.ndarray`.
5. **No raw pointers:** biors-core has none in the public API.

### 4.2 PyO3 Best Practices

From the PyO3 user guide (v0.28):

- `#[pyclass]` requirements: No lifetime parameters, no generic parameters, must be `Send` and `Sync`.
- Interior mutability: Use `#[pyclass(frozen)]` for immutable data.
- Smart pointers: Use `Bound<'py, T>` for function arguments; `Py<T>` for storing Python objects in structs.
- Error handling: Return `PyResult<T>` from `#[pyfunction]`; PyO3 converts `Result` to Python exceptions.
- Packaging: Use `maturin` for building and publishing wheels. Use `abi3-py39` for a single wheel compatible with Python 3.9+.

### 4.3 Python-Safe API Subset

All biors-core public types are already **Python-safe** because they are plain data structs with `String`, `Vec<u8>`, `usize`, `bool`, and nested structs. No `Rc`, no `RefCell`, no raw pointers, no `Cell`. All are automatically `Send + Sync`.

**The only concern is `BufRead`-based APIs.** In PyO3, it is better to expose string-based APIs or byte-slice APIs rather than generic `BufRead` APIs, because Python does not have a native `BufRead` equivalent.

**Recommended Python API surface:**

```rust
#[pyfunction]
fn parse_fasta_records_py(input: &str) -> PyResult<Vec<ProteinSequence>> {
    Ok(biors_core::fasta::parse_fasta_records(input)?)
}

#[pyfunction]
fn validate_fasta_input_py(input: &str) -> PyResult<SequenceValidationReport> {
    Ok(biors_core::fasta::validate_fasta_input(input)?)
}

#[pyfunction]
fn tokenize_fasta_records_py(input: &str) -> PyResult<Vec<TokenizedProtein>> {
    Ok(biors_core::tokenizer::tokenize_fasta_records(input)?)
}

#[pyfunction]
fn build_model_inputs_checked_py(
    tokenized: Vec<TokenizedProtein>,
    policy: ModelInputPolicy,
) -> PyResult<ModelInput> {
    Ok(biors_core::model_input::build_model_inputs_checked(&tokenized, policy)?)
}

#[pyfunction]
fn prepare_workflow_py(
    input_hash: String,
    records: Vec<ProteinSequence>,
    policy: ModelInputPolicy,
) -> PyResult<SequenceWorkflowOutput> {
    Ok(biors_core::workflow::prepare_protein_model_input_workflow(input_hash, &records, policy)?)
}
```

**Module: `package`**
- Export all manifest types as `#[pyclass]`.
- Export `validate_package_manifest(manifest_json: &str)`, `inspect_package_manifest(manifest_json: &str)`.
- Export `read_package_file(base_dir: &str, relative_path: &str)` -> `Vec<u8>` (Python has filesystem access).
- Export `plan_runtime_bridge`.

**Module: `runtime`**
- Export `ExternalProcessBackend` and `ExternalProcessConfig` as `#[pyclass]`.
- Export the `Backend` trait methods.
- Python can spawn processes, so this is safe.

**Module: `verification`**
- Export `StableInputHasher` with methods.
- Export `diff_output_bytes`, `verify_package_outputs`.

### 4.4 What Must Be Wrapped or Adapted for Python

| Item | Python Concern | Strategy |
|------|---------------|----------|
| `ProteinSequence.sequence: Vec<u8>` | Python expects `str` or `bytes` | Expose as `#[getter] fn sequence(&self) -> &PyBytes` or `-> String` |
| `TokenizedProtein.tokens: Vec<u8>` | Python expects `list[int]` | Expose as `#[getter] fn tokens(&self) -> Vec<u8>` (PyO3 converts `Vec<u8>` to `list[int]` by default) |
| `BufRead` APIs | Python has no `BufRead` | Wrap with `&str` or `&[u8]` accepting functions |
| `ModelInputBuildError` | Python exception | Return `PyResult` --- PyO3 auto-converts |
| `FastaReadError::Io` | Python `IOError` | Map to `pyo3::exceptions::PyIOError` |

### 4.5 Python Package Structure

```
biors/
├── Cargo.toml
├── pyproject.toml
├── src/
│   └── lib.rs          # PyO3 bindings
├── python/
│   └── biors/
│       ├── __init__.py   # Re-exports and Pythonic wrappers
│       └── types.pyi     # Type stubs for IDE support
└── README.md
```

**`pyproject.toml`:**
```toml
[build-system]
requires = ["maturin>=1.0,<2.0"]
build-backend = "maturin"

[project]
name = "biors"
version = "0.43.0"
description = "Python bindings for bio-rs core"
requires-python = ">=3.9"

[tool.maturin]
bindings = "pyo3"
features = ["pyo3/extension-module"]
```

---

## 5. External Interface Documentation Plan

### 5.1 New Documentation Files

#### `docs/wasm-api.md`

- Overview of the browser-safe WASM subset
- Installation: `npm install @bio-rs/core-wasm`
- Initialization and API reference by module
- TypeScript interfaces
- Package manifest workflow (JS loads files -> WASM validates)
- Limitations: no filesystem, no external process spawning
- Performance notes: `Uint8Array` vs `string` for large FASTA
- Example code

#### `docs/python-api.md`

- Overview of the PyO3 binding subset
- Installation: `pip install biors`
- API reference by module
- NumPy integration notes
- Type stubs reference
- Example code

#### `docs/rust-api.md`

- Complete public API reference for `biors-core` as a Rust library
- Module-by-module breakdown with doc links
- Stability guarantees (pre-1.0 semver)
- Feature flags
- Migration guide from JSON-boundary usage to direct library usage
- Compatibility matrix with WASM and Python bindings

### 5.2 Updates to Existing Docs

- `docs/wasm-readiness.md`: Update from "compile-level proof" to "runtime WASM package available". Add npm install instructions.
- `docs/python-interop.md`: Update from "JSON boundary only" to "PyO3 bindings available". Add `pip install` instructions. Keep JSON boundary section as alternative.
- `README.md`: Add WASM and Python badges. Update "Not yet" section to move Python bindings from roadmap to current.

---

## 6. Compatibility Matrix

| Feature | Rust Native | WASM (`@bio-rs/core-wasm`) | Python (`biors`) |
|---------|-------------|---------------------------|------------------|
| **FASTA parse** | `parse_fasta_records`, `parse_fasta_records_reader` | `parse_fasta_records` (string) | `parse_fasta_records` (string) |
| **FASTA validate** | `validate_fasta_input`, `validate_fasta_reader` | `validate_fasta_input` (string) | `validate_fasta_input` (string) |
| **Sequence validation** | All types and functions | All types and functions | All types and functions |
| **Tokenize** | `tokenize_fasta_records`, `tokenize_protein`, reader variants | `tokenize_fasta_records`, `tokenize_protein` | `tokenize_fasta_records`, `tokenize_protein` |
| **Tokenizer config** | `ProteinTokenizerConfig`, profiles, vocab | All config types | All config types |
| **Model input** | `build_model_inputs_checked/unchecked` | Both builders | Both builders |
| **Workflow** | `prepare_protein_model_input_workflow` | Full workflow | Full workflow |
| **Package manifest types** | All manifest structs | All manifest structs | All manifest structs |
| **Package manifest validation** | `validate_package_manifest` | `validate_package_manifest` (JSON string) | `validate_package_manifest` (JSON string) |
| **Package manifest inspect** | `inspect_package_manifest` | `inspect_package_manifest` | `inspect_package_manifest` |
| **Package artifact read** | `read_package_file` | Not available (no `std::fs`) | `read_package_file` |
| **Package artifact verify** | `validate_package_manifest_artifacts` | Not available | Available |
| **Runtime backend (abstract)** | `Backend` trait, `BackendCapabilities` | Types only (no execution) | Types + execution |
| **Runtime external process** | `ExternalProcessBackend::execute` | Not available | `ExternalProcessBackend::execute` |
| **Verification / hashing** | `StableInputHasher`, `diff_output_bytes` | All functions | All functions |
| **Versioning policy** | All policy types and functions | All policy types and functions | All policy types and functions |
| **Error types** | `BioRsError`, `FastaReadError`, `ModelInputBuildError` | `BioRsError`, `ModelInputBuildError` | All error types |

### 6.1 Legend

- **Rust Native:** Direct `biors-core` crate usage. Full API, no restrictions.
- **WASM:** Browser-safe subset. No filesystem, no process spawning. Input must be passed as in-memory strings or bytes.
- **Python:** PyO3 bindings. Full API except where Python's type system makes generic traits awkward (replaced with concrete functions).

---

## 7. Implementation Roadmap (Post-Design)

### Phase 1: WASM bindings (v0.45.0 target)
1. Create `packages/rust/biors-wasm/` crate.
2. Add `wasm-bindgen`, `serde-wasm-bindgen`, `js-sys` dependencies.
3. Implement thin wrappers for the WASM-safe subset.
4. Set up `wasm-pack` build in CI.
5. Publish `@bio-rs/core-wasm` to npm.

### Phase 2: Python bindings (v0.44.0 target)
1. Create `packages/rust/biors-python/` crate.
2. Implement `#[pyfunction]` and `#[pyclass]` wrappers.
3. Set up `maturin` build in CI.
4. Publish `biors` to PyPI.

### Phase 3: Documentation
1. Write `docs/wasm-api.md`, `docs/python-api.md`, `docs/rust-api.md`.
2. Update `docs/wasm-readiness.md` and `docs/python-interop.md`.
3. Update `README.md` with binding install instructions.

---

*End of design document.*

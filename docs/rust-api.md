# biors-core Rust API Reference

Version: 0.51.0

This document is the comprehensive public API reference for `biors-core`, the Rust engine behind bio-rs. It covers every public module, type, trait, and function exposed by the crate.

## Table of Contents

- [Overview](#overview)
- [Adding biors-core as a Dependency](#adding-biors-core-as-a-dependency)
- [Stability Guarantees](#stability-guarantees)
- [Feature Flags](#feature-flags)
- [Module Reference](#module-reference)
  - [`error`](#module-error)
  - [`conversion`](#module-conversion)
  - [`fasta`](#module-fasta)
  - [`formats`](#module-formats)
  - [`hash`](#module-hash)
  - [`model_input`](#module-model_input)
  - [`molecule`](#module-molecule)
  - [`package`](#module-package)
  - [`runtime`](#module-runtime)
  - [`sequence`](#module-sequence)
  - [`service`](#module-service)
  - [`structure`](#module-structure)
  - [`templates`](#module-templates)
  - [`tokenizer`](#module-tokenizer)
  - [`verification`](#module-verification)
  - [`versioning`](#module-versioning)
  - [`workflow`](#module-workflow)
- [Usage Examples](#usage-examples)
- [Migration Guide: JSON Boundary to Direct Library Usage](#migration-guide-json-boundary-to-direct-library-usage)
- [WASM and Python Binding Compatibility](#wasm-and-python-binding-compatibility)

## Overview

`biors-core` is the Rust library that powers bio-rs. It handles biological sequence parsing, FASTQ/PDB/SMILES/SDF/MOL2 format parsing, unified record conversion, local task template contracts, protein/DNA/RNA validation, profile-aware tokenization, model input construction, package manifest management, service contracts, runtime planning, and fixture verification. The crate is designed to be dependency-light and deterministic. It uses `serde` for serialization and `sha2` for checksums. It is a `std` crate today; WASM compatibility is maintained through the `wasm32-unknown-unknown` check described below rather than a `no_std` contract.

The library is organized into focused modules. Each module owns one responsibility: FASTA parsing lives in `fasta`, tokenization lives in `tokenizer`, and package management lives in `package`. This makes the API easy to navigate and test.

## Adding biors-core as a Dependency

Add this to your `Cargo.toml`:

```toml
[dependencies]
biors-core = "0.51.0"
```

The crate depends on:

- `serde` and `serde_json` for data contracts
- `sha2` for SHA-256 checksum computation

No other external dependencies are required.

## Stability Guarantees

`biors-core` is pre-1.0. We follow SemVer with one important caveat: minor version bumps may include breaking API changes. Patch releases are reserved for bug fixes and documentation improvements only.

If you pin to a specific minor version, you should expect stability within that line. Before upgrading across minor versions, review the changelog for renamed types, removed functions, or changed defaults.

All public types implement `Debug`, `Clone`, `PartialEq`, and `Eq` where appropriate. Most also derive `Serialize` and `Deserialize` so they can cross FFI or network boundaries without custom glue.

## Feature Flags

`biors-core` currently has no Cargo feature flags. All public APIs are always available, and there is no feature-gated `no_std` mode.

## Module Reference

### Module: `conversion`

The `conversion` module maps parsed biological records into the shared
`BioEntity` JSON export contract. It is deterministic and local-only; it does
not call models, upload data, or claim inference/search behavior.

Public types:

- **`BioEntityJsonExport`** — JSON-ready export with `schema_version`,
  aggregate record counts, warning/error counts, and converted entities.
- **`BioEntity`** — One converted biological entity with stable `id`,
  `entity_type`, `source`, `record`, and `validation`.
- **`BioEntityType`** — `Sequence`, `Structure`, or `Molecule`.
- **`ConversionRecord`** — Tagged record enum wrapping
  `ConvertedSequenceRecord`, `ConvertedStructureRecord`, or
  `ConvertedMoleculeRecord`.
- **`ConversionSource`** — Source `BioFormat` and optional
  `FormatMetadata`.
- **`ConversionValidation`** — Per-entity validity, model-readiness,
  warnings, and errors.
- **`ConversionIssue`**, **`ConversionIssueCode`**, and
  **`ConversionIssueSeverity`** — Stable conversion issue contract.
- **`CONVERSION_SCHEMA_VERSION`** — `biors.conversion.v0`.

Public functions:

- `pub fn convert_fasta_records(records: &[ProteinSequence], kind_selection: SequenceKindSelection) -> BioEntityJsonExport`
  Convert parsed FASTA records into sequence entities.
- `pub fn fasta_record_to_bio_entity(record: &ProteinSequence, kind_selection: SequenceKindSelection) -> BioEntity`
  Convert one FASTA record.
- `pub fn convert_fastq_records(records: &[FastqRecord]) -> BioEntityJsonExport`
  Convert parsed FASTQ records into DNA sequence entities with quality strings.
- `pub fn fastq_record_to_bio_entity(record: &FastqRecord) -> BioEntity`
  Convert one FASTQ record.
- `pub fn structure_record_to_bio_entity(record: &StructureRecord) -> BioEntity`
  Convert a parsed structure record and attach extracted chain sequences.
- `pub fn molecule_record_to_bio_entity(record: &MoleculeRecord) -> BioEntity`
  Convert one parsed molecule record with `FormatRecord` projection and
  deterministic molecule features.
- `pub fn convert_molecule_records(records: &[MoleculeRecord]) -> BioEntityJsonExport`
  Convert parsed molecule records.
- `pub fn export_bio_entities(entities: Vec<BioEntity>) -> BioEntityJsonExport`
  Wrap already converted entities into the aggregate JSON export shape.

### Module: `error`

The `error` module defines the diagnostic trait and the core error types used across the crate. Every validation or parse problem surfaces through these types.

#### Types

- **`Diagnostic` trait** — Common interface for errors and validation issues. Methods:
  - `fn code(&self) -> &'static str` — stable machine-readable diagnostic code
  - `fn message(&self) -> String` — human-readable message
  - `fn location(&self) -> Option<ErrorLocation>` — optional structured location

- **`ErrorLocation`** — Machine-readable location metadata for parse or validation errors.
  - `pub line: Option<usize>` — one-based source line
  - `pub record_index: Option<usize>` — zero-based FASTA record index
  - `pub const fn line(line: usize) -> Self`
  - `pub const fn record(line: usize, record_index: usize) -> Self`

- **`BioRsError`** — Public parse errors from FASTA string APIs.
  - `EmptyInput`
  - `MissingIdentifier { line, record_index }`
  - `MissingHeader { line }`
  - `MissingSequence { id, line, record_index }`
  - `pub const fn code(&self) -> &'static str`
  - `pub const fn location(&self) -> Option<ErrorLocation>`

- **`FastaReadError`** — Error type for streaming FASTA reader APIs.
  - `Parse(BioRsError)`
  - `Io(std::io::Error)`
  - `pub const fn code(&self) -> &'static str`

Both `BioRsError` and `FastaReadError` implement `std::error::Error`, `Display`, and `Diagnostic`. `From` conversions exist for both `BioRsError -> FastaReadError` and `std::io::Error -> FastaReadError`.

### Module: `fasta`

The `fasta` module provides FASTA parsing and validation APIs. It works with both in-memory strings and buffered readers.

#### Types

- **`ParsedFastaInput`** — Result of reader-based FASTA parsing.
  - `pub input_hash: String`
  - `pub records: Vec<ProteinSequence>`

- **`ValidatedFastaInput`** — Result of reader-based FASTA validation.
  - `pub input_hash: String`
  - `pub report: SequenceValidationReport`

#### Functions

- `pub fn parse_fasta_records(input: &str) -> Result<Vec<ProteinSequence>, BioRsError>`
  Parse FASTA text into normalized sequence records. The return type keeps the
  legacy `ProteinSequence` name for compatibility; new examples can use the
  `BiologicalSequence` or `FastaSequence` aliases when the code is not
  protein-specific.

- `pub fn parse_fasta_records_reader<R: BufRead>(reader: R) -> Result<ParsedFastaInput, FastaReadError>`
  Parse FASTA records from a buffered reader without preloading the full input.

- `pub fn validate_fasta_input(input: &str) -> Result<SequenceValidationReport, BioRsError>`
  Validate FASTA text and return aggregate sequence validation details.

- `pub fn validate_fasta_reader<R: BufRead>(reader: R) -> Result<SequenceValidationReport, FastaReadError>`
  Validate FASTA from a buffered reader and discard the raw input hash.

- `pub fn validate_fasta_reader_with_hash<R: BufRead>(reader: R) -> Result<ValidatedFastaInput, FastaReadError>`
  Validate FASTA from a buffered reader and include a stable raw input hash.

### Module: `formats`

The `formats` module provides shared biological file-format contracts and
format-specific parser support. FASTQ and PDB are executable parser families;
other common formats are represented in the capability matrix as reviewed
candidates until their parser contracts are implemented.

#### Types

- **`BioFormat`** — recognized format family.
  - `Fasta`, `Fastq`, `Gff3`, `Gtf`, `Bed`, `Vcf`, `Genbank`, `UniprotFlat`, `Pdb`, `Mmcif`, `Csv`, `Tsv`
  - `pub const fn as_str(self) -> &'static str`
  - `pub const fn display_name(self) -> &'static str`

- **`FormatMetadata`** — shared source location metadata.
  - `pub record_index: usize`
  - `pub line_start: usize`
  - `pub line_end: usize`
  - `pub const fn new(record_index, line_start, line_end) -> Self`

- **`FormatField`** — shared name/value field.
  - `pub name: String`
  - `pub value: String`
  - `pub fn new(name, value) -> Self`

- **`FormatRecord`** — shared parsed-record projection.
  - `pub format: BioFormat`
  - `pub id: String`
  - `pub metadata: FormatMetadata`
  - `pub fields: Vec<FormatField>`
  - `pub fn new(format, id, metadata, fields) -> Self`

- **`FormatSupportStatus`** — `Supported`, `ReviewedCandidate`, or `Future`.

- **`FormatCapability`** — support-matrix row.
  - `pub format: BioFormat`
  - `pub status: FormatSupportStatus`
  - `pub record_contract: String`
  - `pub validation_requirements: Vec<String>`
  - `pub notes: Vec<String>`

- **`FastqRecord`** — parsed FASTQ read.
  - `pub id: String`
  - `pub description: Option<String>`
  - `pub sequence: String`
  - `pub quality: String`
  - `pub metadata: FormatMetadata`
  - `pub fn to_format_record(&self) -> FormatRecord`

- **`FastqValidationReport`** — aggregate FASTQ validation report.
  - `pub format: BioFormat`
  - `pub sequence_kind: SequenceKind`
  - `pub records: usize`
  - `pub valid_records: usize`
  - `pub warning_count: usize`
  - `pub error_count: usize`
  - `pub record_reports: Vec<FastqRecordValidation>`

- **`FastqRecordValidation`** — per-read validation details.
  - `pub id: String`
  - `pub description: Option<String>`
  - `pub sequence_length: usize`
  - `pub quality_length: usize`
  - `pub metadata: FormatMetadata`
  - `pub valid: bool`
  - `pub warnings: Vec<FastqValidationIssue>`
  - `pub errors: Vec<FastqValidationIssue>`

- **`FastqValidationIssueCode`** — `AmbiguousSymbol`, `InvalidSymbol`, or `InvalidQualityCharacter`.

- **`FastqValidationIssue`** — per-symbol FASTQ validation issue.
  - `pub symbol: char`
  - `pub position: usize`
  - `pub code: FastqValidationIssueCode`
  - `pub message: String`

- **`FastqParseError`** — FASTQ parse failures.
  - `EmptyInput`
  - `MissingHeader { line, record_index }`
  - `MissingIdentifier { line, record_index }`
  - `MissingSeparator { id, line, record_index }`
  - `MissingSequence { id, line, record_index }`
  - `SeparatorIdentifierMismatch { id, separator_id, line, record_index }`
  - `MissingQuality { id, expected, observed, record_index }`
  - `QualityLengthMismatch { id, expected, observed, line, record_index }`
  - `pub const fn code(&self) -> &'static str`
  - `pub const fn location(&self) -> Option<ErrorLocation>`

- **`FormatReadError`** — streaming format reader error.
  - `FastqParse(FastqParseError)`
  - `Io(std::io::Error)`
  - `pub const fn code(&self) -> &'static str`

- **`ParsedFastqInput`** — `pub input_hash: String`, `pub records: Vec<FastqRecord>`
- **`ValidatedFastqInput`** — `pub input_hash: String`, `pub report: FastqValidationReport`

#### Functions

- `pub fn format_capabilities() -> Vec<FormatCapability>`
  Return the current format support matrix.

- `pub fn parse_fastq_records(input: &str) -> Result<Vec<FastqRecord>, FastqParseError>`
  Parse in-memory FASTQ text.

- `pub fn parse_fastq_records_reader<R: BufRead>(reader: R) -> Result<ParsedFastqInput, FormatReadError>`
  Parse FASTQ from a buffered reader and return parsed records plus a stable
  raw input hash.

- `pub fn validate_fastq_reader<R: BufRead>(reader: R) -> Result<FastqValidationReport, FormatReadError>`
  Validate FASTQ from a buffered reader and discard the raw input hash.

- `pub fn validate_fastq_reader_with_hash<R: BufRead>(reader: R) -> Result<ValidatedFastqInput, FormatReadError>`
  Validate FASTQ from a buffered reader and return the validation report plus
  a stable raw input hash.

### Module: `hash`

The `hash` module provides explicit SHA-256 digest utilities for raw artifact
checksums and canonical JSON comparisons.

#### Functions

- `pub fn sha256_bytes_digest(bytes: &[u8]) -> String`
  Compute a raw byte-for-byte SHA-256 digest in `sha256:<hex>` form. Use this
  for package artifacts, files, and published checksums.

- `pub fn sha256_canonical_json_digest(bytes: &[u8]) -> String`
  Compute a canonical JSON SHA-256 digest in `sha256:<hex>` form. JSON inputs
  are normalized before hashing so semantically equivalent JSON produces the
  same digest; non-JSON input falls back to raw bytes.

- `pub fn sha256_digest(bytes: &[u8]) -> String`
  Compatibility alias for canonical JSON hashing. Prefer the explicit
  `sha256_bytes_digest` or `sha256_canonical_json_digest` functions in new code.

- `pub fn is_sha256_checksum(checksum: &str) -> bool`
  Return true when a checksum uses the supported `sha256:<64 lowercase hex>` format.

### Module: `model_input`

The `model_input` module converts tokenized sequence records into model-ready arrays with truncation, padding, and attention masks.

#### Types

- **`ModelInputPolicy`** — Policy for converting tokenized records into model-ready arrays.
  - `pub max_length: usize`
  - `pub pad_token_id: u8`
  - `pub padding: PaddingPolicy`

- **`PaddingPolicy`** — Padding strategy.
  - `FixedLength` — pad every record to `max_length`
  - `NoPadding` — preserve each record's truncated length without padding

- **`ModelInput`** — Batch of model-ready input records.
  - `pub policy: ModelInputPolicy`
  - `pub records: Vec<ModelInputRecord>`

- **`ModelInputRecord`** — Model-ready representation of one tokenized record.
  - `pub id: String`
  - `pub input_ids: Vec<u8>`
  - `pub attention_mask: Vec<u8>` — `1` for real tokens, `0` for padding
  - `pub truncated: bool`

- **`ModelInputBuildError`** — Errors from checked builders.
  - `InvalidPolicy { message }`
  - `InvalidInputHash { input_hash }`
  - `InvalidTokenizedSequence { id, warning_count, error_count }`

- **`ModelInputPayloadError`** — Semantic validation errors for externally supplied model-input payloads.
  - `LengthMismatch { id, input_ids, attention_mask }`
  - `FixedLengthMismatch { id, expected, actual }`
  - `NoPaddingLengthExceeded { id, max_length, actual }`
  - `NonBinaryAttentionMask { id, index, value }`
  - `EmptyUnmaskedTokens { id }`

#### Functions

- `pub fn build_model_inputs_unchecked(tokenized: &[TokenizedProtein], policy: ModelInputPolicy) -> ModelInput`
  Build model input without rejecting unresolved tokenization warnings or errors.

- `pub fn build_model_inputs_checked(tokenized: &[TokenizedProtein], policy: ModelInputPolicy) -> Result<ModelInput, ModelInputBuildError>`
  Build model input after rejecting invalid policies and unresolved residue issues.

- `pub fn validate_model_input_policy(policy: &ModelInputPolicy) -> Result<(), ModelInputBuildError>`
  Validate a model-input policy without building records.

- `pub fn validate_model_input_payload(input: &ModelInput) -> Result<(), ModelInputPayloadError>`
  Validate semantic invariants that JSON Schema cannot fully express, including
  matching `input_ids`/`attention_mask` lengths, fixed-length record sizes,
  no-padding maximum length, binary masks, and at least one unmasked token.

### Module: `package`

The `package` module is the largest surface in `biors-core`. It handles manifest inspection, validation, artifact verification, schema migration, and runtime bridge planning.

#### Types

**Manifest types:**

- **`PackageManifest`** — Top-level package manifest.
  - `pub schema_version: SchemaVersion`
  - `pub name: String`
  - `pub package_layout: Option<PackageDirectoryLayout>`
  - `pub metadata: Option<PackageMetadata>`
  - `pub model: ModelArtifact`
  - `pub tokenizer: Option<TokenAsset>`
  - `pub vocab: Option<TokenAsset>`
  - `pub preprocessing: Vec<PipelineStep>`
  - `pub postprocessing: Vec<PipelineStep>`
  - `pub runtime: RuntimeTarget`
  - `pub expected_input: Option<DataShape>`
  - `pub expected_output: Option<DataShape>`
  - `pub fixtures: Vec<PackageFixture>`

- **`PackageDirectoryLayout`** — Declared directory structure.
  - `pub manifest: String`
  - `pub models: String`
  - `pub tokenizers: Option<String>`
  - `pub vocabs: Option<String>`
  - `pub pipelines: Option<String>`
  - `pub fixtures: String`
  - `pub observed: Option<String>`
  - `pub docs: String`

- **`PackageMetadata`** — Research metadata for v1 manifests.
  - `pub license: LicenseMetadata`
  - `pub citation: CitationMetadata`
  - `pub model_card: ModelCardMetadata`

- **`LicenseMetadata`** — `pub expression: String`, `pub file: Option<DocumentArtifact>`
- **`CitationMetadata`** — `pub preferred_citation: String`, `pub doi: Option<String>`, `pub file: Option<DocumentArtifact>`
- **`ModelCardMetadata`** — `pub path: String`, `pub checksum: Option<String>`, `pub summary: String`, `pub intended_use: Vec<String>`, `pub limitations: Vec<String>`
- **`DocumentArtifact`** — `pub path: String`, `pub checksum: Option<String>`
- **`ModelArtifact`** — `pub format: ModelFormat`, `pub path: String`, `pub checksum: Option<String>`, `pub metadata: Option<ModelArtifactMetadata>`
- **`ModelArtifactMetadata`** — `pub name: String`, optional `version`, `architecture`, `task`, `source`, `description`
- **`TokenAsset`** — `pub name: String`, `pub path: String`, `pub checksum: Option<String>`, `pub contract_version: Option<String>`
- **`PipelineStep`** — `pub name: String`, `pub implementation: String`, `pub contract: String`, optional `contract_version`, optional `config`
- **`PipelineConfigArtifact`** — `pub path: String`, `pub schema_version: PipelineConfigVersion`, optional `checksum`
- **`RuntimeTarget`** — `pub backend: RuntimeBackend`, `pub target: RuntimeTargetPlatform`, optional `version`
- **`PackageFixture`** — `pub name: String`, `pub input: String`, `pub expected_output: String`, optional `input_hash`, optional `expected_output_hash`
- **`DataShape`** — `pub shape: Vec<String>`, `pub dtype: DataType`

**Report and summary types:**

- **`PackageManifestSummary`** — Compact manifest summary for inspect-style output.
- **`PackageDirectoryLayoutSummary`** — Declared layout paths.
- **`PackageMetadataSummary`** — Compact research metadata.
- **`ModelArtifactMetadataSummary`** — Compact model artifact metadata.
- **`PackageLayoutSummary`** — Package-relative layout paths.
- **`PackageValidationReport`** — Manifest validation result.
  - `pub valid: bool`
  - `pub issues: Vec<String>`
  - `pub structured_issues: Vec<PackageValidationIssue>`
- **`PackageValidationIssue`** — `pub code: PackageValidationIssueCode`, `pub field: String`, `pub message: String`
- **`PackageValidationIssueCode`** — Enum with `RequiredField`, `MissingFixture`, `InvalidShape`, `InvalidChecksumFormat`, `ChecksumMismatch`, `InvalidAssetPath`, `AssetReadFailed`, `LayoutMismatch`
- **`RuntimeBridgeReport`** — Runtime bridge readiness report. `ready` is a
  backward-compatible alias for manifest/runtime `contract_ready`; use
  `artifact_checked`, `execution_ready`, and `readiness_notes` before treating a
  package as executable.
- **`BackendCompatibilityCheck`** — One deterministic compatibility decision.
- **`BackendCapabilitiesSummary`** — Capability summary.
- **`BenchmarkEvidence`** — One benchmark evidence record.
- **`BenchmarkMetric`** — `pub metric_name: String`, `pub value: f64`, `pub unit: String`
- **`RegressionBaseline`** — Anchored regression baseline.

**Tooling types:**

- **`PackageSchemaMigrationReport`** — Migration guidance.
- **`PackageSchemaCompatibilityReport`** — Schema compatibility between two manifests.
- **`PackageManifestDiffReport`** — Canonical manifest diff with compatibility annotations.
- **`PackageManifestConversionInput`** — Input required for v0-to-v1 conversion.
- **`PackageManifestConversionOutput`** — Converted manifest plus provenance.
- **`PackageManifestConversionReport`** — Conversion report.
- **`PackageManifestConversionError`** — `MissingConversionInput` or `Unsupported`

**Schema types:**

- **`SchemaVersion`** — `BiorsPackageV0`, `BiorsPackageV1`
- **`ModelFormat`** — `Onnx`, `Safetensors`
- **`RuntimeBackend`** — `OnnxWebgpu`, `Candle`, `ExternalProcess`
  - `ExternalProcess` is internal/experimental for package manifests; `validate_package_manifest` rejects it until its public package contract is promoted.
- **`RuntimeTargetPlatform`** — `BrowserWasmWebgpu`, `LocalCpu`
- **`DataType`** — `Uint8`, `Float32`
- **`PipelineConfigVersion`** — `BiorsPipelineV0`

**Path types:**

- **`PackageArtifactError`** — `EmptyPath`, `AbsolutePath`, `PathEscape`, `AssetReadFailed`

#### Functions

- `pub fn validate_package_manifest(manifest: &PackageManifest) -> PackageValidationReport`
  Validate manifest fields that do not require filesystem access.

- `pub fn validate_package_manifest_artifacts(manifest: &PackageManifest, base_dir: &Path) -> PackageValidationReport`
  Validate manifests including artifact presence and checksum verification.

- `pub fn validate_package_manifest_artifacts_with_pipeline_config_validator(manifest: &PackageManifest, base_dir: &Path, pipeline_config_validator: Option<&ReferencedConfigValidator<'_>>) -> PackageValidationReport`
  Validate manifests including package artifacts and optional referenced
  pipeline config content without adding TOML parser dependencies to
  `biors-core`.

- `pub fn validate_package_manifest_artifacts_with_manifest_path_and_pipeline_config_validator(manifest: &PackageManifest, base_dir: &Path, manifest_path: Option<&Path>, pipeline_config_validator: Option<&ReferencedConfigValidator<'_>>) -> PackageValidationReport`
  Validate manifests including package artifacts, optional referenced pipeline
  config content, and optional `package_layout.manifest` matching against the
  actual manifest path.

- `pub fn inspect_package_manifest(manifest: &PackageManifest) -> PackageManifestSummary`
  Build a compact summary for inspect-style output.

- `pub fn plan_runtime_bridge(manifest: &PackageManifest) -> RuntimeBridgeReport`
  Build a runtime bridge readiness report from a manifest.

- `pub fn compare_package_manifest_schemas(left_path: &str, right_path: &str, left: &PackageManifest, right: &PackageManifest) -> PackageSchemaCompatibilityReport`
  Compare schema compatibility between two manifests.

- `pub fn convert_package_manifest(manifest: &PackageManifest, to: SchemaVersion, input: Option<PackageManifestConversionInput>) -> Result<PackageManifestConversionOutput, PackageManifestConversionError>`
  Convert a manifest to a different schema version.

- `pub fn diff_package_manifests(left_path: &str, right_path: &str, left: &PackageManifest, right: &PackageManifest, left_bytes: &[u8], right_bytes: &[u8]) -> PackageManifestDiffReport`
  Build a canonical diff between two manifests.

- `pub fn plan_package_schema_migration(manifest: &PackageManifest, to: SchemaVersion) -> Option<PackageSchemaMigrationReport>`
  Plan a schema migration.

- `pub fn read_package_file(base_dir: &Path, relative_path: &str) -> Result<Vec<u8>, PackageArtifactError>`
  Read a package-relative asset.

- `pub fn resolve_package_asset_path(base_dir: &Path, relative_path: &str) -> Result<PathBuf, PackageArtifactError>`
  Resolve a package-relative path.

### Module: `runtime`

The `runtime` module defines backend abstraction contracts and provides a guarded external-process backend adapter.

#### Types

- **`Backend` trait** — Minimal execution backend interface.
  - `fn config(&self) -> &BackendConfig`
  - `fn capabilities(&self) -> &BackendCapabilities`
  - `fn execute(&self, context: ExecutionContext) -> Result<ExecutionResult, BackendExecutionError>`
  - `fn execute_checked(&self, context: ExecutionContext) -> Result<ExecutionResult, BackendExecutionError>` — validates context before execution

- **`BackendConfig`** — Static backend identity.
  - `pub backend_id: String`
  - `pub provider: String`
  - `pub version: Option<String>`
  - `pub model_artifact: Option<String>`

- **`BackendCapabilities`** — Capability contract.
  - `pub deterministic: bool`
  - `pub supports_batch: bool`
  - `pub supports_streaming: bool`
  - `pub supported_inputs: Vec<String>`
  - `pub supported_outputs: Vec<String>`
  - `pub max_input_bytes: Option<usize>`
  - `pub fn supports_input(&self, format: &str) -> bool`
  - `pub fn supports_output(&self, format: &str) -> bool`
  - `pub fn compatibility_report(&self, context: &ExecutionContext) -> BackendCompatibilityReport`
  - `pub fn ensure_context(&self, backend_id: &str, context: &ExecutionContext) -> Result<(), BackendExecutionError>`

- **`BackendCompatibilityReport`** — `pub compatible: bool`, `pub issues: Vec<RuntimeCompatibilityIssue>`
- **`RuntimeCompatibilityIssue`** — `pub code: RuntimeCompatibilityIssueCode`, `pub message: String`
- **`RuntimeCompatibilityIssueCode`** — `UnsupportedInput`, `UnsupportedOutput`, `PayloadTooLarge`
- **`ExecutionContext`** — `pub trace_id: Option<String>`, `pub input_format: String`, `pub requested_output_format: Option<String>`, `pub payload: Vec<u8>`, `pub metadata: Vec<ExecutionMetadata>`
- **`ExecutionMetadata`** — `pub key: String`, `pub value: String`
- **`ExecutionResult`** — `pub trace_id: Option<String>`, `pub output_format: String`, `pub payload: Vec<u8>`, `pub metadata: Vec<ExecutionMetadata>`
- **`BackendExecutionError`** — `pub backend_id: String`, `pub code: String`, `pub message: String`
  - `pub fn unsupported_input(backend_id: &str, input_format: &str) -> Self`
  - `pub fn unsupported_output(backend_id: &str, output_format: &str) -> Self`
  - `pub fn output_format_mismatch(backend_id: &str, requested_output_format: &str, actual_output_format: &str) -> Self`
  - `pub fn payload_too_large(backend_id: &str, payload_bytes: usize, max_input_bytes: usize) -> Self`
  - `pub fn execution_failed(backend_id: &str, message: impl Into<String>) -> Self`
  - `pub fn process_spawn_failed(backend_id: &str, message: impl Into<String>) -> Self`
  - `pub fn process_io_failed(backend_id: &str, message: impl Into<String>) -> Self`
  - `pub fn process_timeout(backend_id: &str, timeout_millis: u64) -> Self`
  - `pub fn process_exit_failed(backend_id: &str, status: &str, stderr_bytes: usize) -> Self`
  - `pub fn process_stdout_too_large(backend_id: &str, limit_bytes: usize, total_bytes: usize) -> Self`
  - `pub fn process_stderr_too_large(backend_id: &str, limit_bytes: usize, total_bytes: usize) -> Self`
  - `pub fn process_invalid_output(backend_id: &str, output_bytes: usize, message: impl Into<String>) -> Self`

- **`ExternalProcessConfig`** — Invocation policy for external process backends.
  - `pub program: PathBuf`
  - `pub args: Vec<String>`
  - `pub current_dir: Option<PathBuf>`
  - `pub environment: Vec<ExecutionMetadata>`
  - `pub inherit_environment: bool`
  - `pub timeout_millis: u64`
  - `pub max_stdout_bytes: usize`
  - `pub max_stderr_bytes: usize`
  - `pub fn new(program: impl Into<PathBuf>) -> Self`

- **`ExternalProcessBackend`** — Backend that delegates to a local child process.
  - `pub config: BackendConfig`
  - `pub capabilities: BackendCapabilities`
  - `pub process: ExternalProcessConfig`
  - `pub fn new(config: BackendConfig, capabilities: BackendCapabilities, process: ExternalProcessConfig) -> Self`

**Constants:**

- `pub const DEFAULT_PROCESS_TIMEOUT_MILLIS: u64 = 30_000`
- `pub const DEFAULT_STDOUT_LIMIT_BYTES: usize = 16 * 1024 * 1024`
- `pub const DEFAULT_STDERR_LIMIT_BYTES: usize = 1024 * 1024`

### Module: `service`

The `service` module defines a transport-agnostic service interface contract for
embedding bio-rs in a caller-owned service host. It does not include an HTTP
server, network listener, authentication, rate limiting, or deployment runtime.

#### Constants

- **`SERVICE_INTERFACE_SCHEMA_VERSION`** — current service contract schema
  version, `biors.service_interface.v0`.

#### Types

- **`ServiceInterfaceDocument`** — top-level service contract.
  - `pub schema_version: String`
  - `pub service_name: String`
  - `pub service_version: String`
  - `pub server_runtime: String`
  - `pub transport_model: String`
  - `pub runtime_separation: RuntimeServiceSeparation`
  - `pub openapi: OpenApiDirection`
  - `pub routes: Vec<ServiceRoute>`

- **`RuntimeServiceSeparation`** — explicit ownership split between
  `biors-core` and the embedding service host.
  - `pub core_contract_owner: String`
  - `pub service_runtime_owner: String`
  - `pub permitted_in_core: Vec<String>`
  - `pub forbidden_in_core: Vec<String>`

- **`OpenApiDirection`** — offline OpenAPI generation guidance.
  - `pub status: String`
  - `pub title: String`
  - `pub version: String`
  - `pub schema_base_uri: String`
  - `pub notes: Vec<String>`

- **`ServiceRoute`** — one deterministic service operation contract.
  - `pub operation_id: String`
  - `pub domain: String`
  - `pub method: String`
  - `pub path: String`
  - `pub request_schema: String`
  - `pub response_schema: String`
  - `pub deterministic: bool`
  - `pub idempotent: bool`
  - `pub file_access: String`
  - `pub runtime_boundary: String`

#### Functions

- `pub fn current_service_interface_document() -> ServiceInterfaceDocument`
  Build the current service interface document using the crate version.

- `pub fn service_interface_document(version: impl Into<String>) -> ServiceInterfaceDocument`
  Build a service interface document for a provided version string.

- `pub fn service_routes() -> Vec<ServiceRoute>`
  Return the stable v0 operation list for service hosts.

### Module: `sequence`

The `sequence` module handles biological sequence types, normalization, alphabet policies, and validation reports for protein, DNA, and RNA.

#### Types

- **`ProteinSequence`** — Named sequence record using the legacy public type
  name retained for compatibility.
  - `pub id: String`
  - `pub sequence: Vec<u8>` — normalized, whitespace removed, ASCII uppercased
  - `pub fn new_normalized(id: impl Into<String>, sequence: impl AsRef<str>) -> Self`
  - Directly constructed values are normalized by validation and tokenization before residue classification.

- **`BiologicalSequence`** / **`FastaSequence`** — Sequence-generic aliases
  for `ProteinSequence`. Prefer these aliases in new examples when the code is
  not protein-specific.

- **`SequenceKind`** — Biological alphabet family.
  - `Protein`, `Dna`, `Rna`
  - `pub const fn alphabet_name(self) -> &'static str`
  - `pub const fn display_name(self) -> &'static str`

- **`SequenceKindSelection`** — User selection for kind-aware validation.
  - `Auto`, `Explicit(SequenceKind)`
  - `pub const fn explicit_kind(self) -> Option<SequenceKind>`

- **`ResidueIssue`** — Residue-level warning or error.
  - `pub residue: char`
  - `pub position: usize` — one-based

- **`ValidatedSequence`** — Validation result for one legacy protein-default sequence path.
  - `pub id: String`, `pub sequence: String`, `pub alphabet: String`, `pub valid: bool`, `pub warnings: Vec<ResidueIssue>`, `pub errors: Vec<ResidueIssue>`

- **`SequenceValidationReport`** — Aggregate report for protein batches.
  - `pub records: usize`, `pub valid_records: usize`, `pub warning_count: usize`, `pub error_count: usize`, `pub sequences: Vec<ValidatedSequence>`

- **`SequenceRecord`** — Normalized sequence with assigned kind.
  - `pub id: String`, `pub sequence: String`, `pub kind: SequenceKind`
  - `pub fn new(id: impl Into<String>, sequence: impl AsRef<str>, kind: SequenceKind) -> Self`

- **`SequenceValidationIssueCode`** — `AmbiguousSymbol`, `InvalidSymbol` with `pub const fn as_str(self) -> &'static str`; returns the same stable values serialized in payloads: `ambiguous_symbol` and `invalid_symbol`
- **`SequenceValidationIssue`** — Kind-aware issue with `pub symbol: char`, `pub position: usize`, `pub kind: SequenceKind`, `pub code: SequenceValidationIssueCode`, `pub message: String`
  - `pub fn ambiguous(symbol: char, position: usize, kind: SequenceKind) -> Self`
  - `pub fn invalid(symbol: char, position: usize, kind: SequenceKind) -> Self`

- **`SequenceKindDetection`** — Auto-detection metadata for kind-aware validation.
  - `pub selected_kind: SequenceKind`, `pub candidate_kinds: Vec<SequenceKind>`, `pub ambiguous: bool`

- **`ValidatedSequenceRecord`** — Kind-aware validation result.
  - `pub id: String`, `pub sequence: String`, `pub kind: SequenceKind`, `pub alphabet: String`, `pub auto_detection: Option<SequenceKindDetection>`, `pub valid: bool`, `pub warnings: Vec<SequenceValidationIssue>`, `pub errors: Vec<SequenceValidationIssue>`

- **`SequenceKindCounts`** — `pub protein: usize`, `pub dna: usize`, `pub rna: usize` with `pub fn increment(&mut self, kind: SequenceKind)`

- **`KindAwareSequenceValidationReport`** — Mixed batch report.
  - `pub records: usize`, `pub valid_records: usize`, `pub warning_count: usize`, `pub error_count: usize`, `pub kind_counts: SequenceKindCounts`, `pub sequences: Vec<ValidatedSequenceRecord>`

- **`KindAwareSequenceValidationSummary`** — Mixed batch summary without per-record payloads.
  - Same fields as report minus `sequences`
  - `pub fn add_record(&mut self, record: &ValidatedSequenceRecord)`

- **`AlphabetPolicy`** — Policy for one sequence kind.
  - `pub const fn for_kind(kind: SequenceKind) -> Self`
  - `pub const fn kind(self) -> SequenceKind`
  - `pub const fn name(self) -> &'static str`
  - `pub fn classify(self, symbol: char) -> SymbolClass`
  - `pub fn classify_byte(self, symbol: u8) -> SymbolClass`

- **`SymbolClass`** — `Standard`, `Ambiguous`, `Invalid`

**Constants:**

- `pub const PROTEIN_20: &str = "protein-20"`
- `pub const PROTEIN_20_RESIDUES: [char; 20]`
- `pub const AMBIGUOUS_RESIDUES: [char; 6]`

#### Functions

- `pub fn normalize_sequence(sequence: &str) -> String`
  Normalize sequence text by removing whitespace and uppercasing ASCII letters.

- `pub fn detect_sequence_kind(sequence: &str) -> SequenceKind`
  Detect the most likely sequence kind from normalized symbols.

- `pub fn detect_sequence_kind_with_metadata(sequence: &str) -> SequenceKindDetection`
  Detect the most likely sequence kind and return candidate-kind ambiguity metadata.

- `pub fn validate_protein_sequence(protein: &ProteinSequence) -> ValidatedSequence`
  Normalize and validate one protein against the protein-20 policy.

- `pub fn validate_sequence_record(record: &SequenceRecord) -> ValidatedSequenceRecord`
  Validate one sequence against its assigned alphabet policy.

- `pub fn summarize_validated_sequences(sequences: Vec<ValidatedSequence>) -> SequenceValidationReport`
  Aggregate protein validation results.

- `pub fn summarize_validated_sequence_records(sequences: Vec<ValidatedSequenceRecord>) -> KindAwareSequenceValidationReport`
  Aggregate kind-aware validation results.

- `pub fn validate_fasta_input_with_kind(input: &str, selection: SequenceKindSelection) -> Result<KindAwareSequenceValidationReport, BioRsError>`
  Validate FASTA text with explicit or auto-detected kinds.

- `pub fn validate_fasta_reader_with_kind<R: BufRead>(reader: R, selection: SequenceKindSelection) -> Result<KindAwareSequenceValidationReport, FastaReadError>`
  Reader-based kind-aware validation.

- `pub fn validate_fasta_reader_with_kind_and_hash<R: BufRead>(reader: R, selection: SequenceKindSelection) -> Result<ValidatedKindAwareFastaInput, FastaReadError>`
  Reader-based validation with input hash.

- `pub fn validate_fasta_reader_summary_with_kind_and_hash<R: BufRead>(reader: R, selection: SequenceKindSelection) -> Result<ValidatedKindAwareFastaSummaryInput, FastaReadError>`
  Reader-based summary without per-record payloads.

### Module: `structure`

The `structure` module provides macromolecular structure records, PDB parsing,
structure validation, chain extraction, sequence extraction, and
protein-sequence-to-structure mapping.

#### Types

- **`StructureRecord`** — Parsed structure with `format`, optional `id`,
  `metadata`, and `chains`.
- **`StructureMetadata`** — Source and aggregate counts: title, line count,
  model count, ATOM count, HETATM count, SEQRES chain count, and missing
  residue count.
- **`Chain`** — Chain identifier, coordinate-bearing residues,
  coordinate-derived protein sequence, optional SEQRES sequence, and
  missing-residue annotations.
- **`Residue3D`** — Residue name, sequence number, optional insertion code,
  HETATM marker, optional one-letter protein code, and atoms.
- **`Atom`** — Atom serial, name, alternate location, element, coordinate,
  occupancy, and temperature factor.
- **`Coordinate`** — Cartesian `x`, `y`, and `z` values in Angstroms.
- **`MissingResidue`** — `REMARK 465` residue name, chain ID, sequence number,
  and optional insertion code.
- **`StructureValidationReport`** — Aggregate validation result with chain
  reports, warning count, error count, and structured issue lists.
- **`StructureChainReport`** — Per-chain residue/atom counts, sequence lengths,
  missing-residue count, and `ProteinStructureMapping`.
- **`StructureSequenceOutput`** / **`StructureSequenceChain`** — Sequence
  extraction payload for PDB structure commands and Rust callers.
- **`ProteinStructureMapping`** — Mapping status, message, and one-based
  coordinate-to-SEQRES positions.
- **`ProteinStructureMappingStatus`** — `Exact`, `CoordinateSubsequence`,
  `MissingSeqres`, or `Mismatch`.
- **`StructureValidationIssueCode`** — `NoCoordinateChains`,
  `InvalidCoordinate`, `InvalidOccupancy`, `SuspiciousOccupancy`,
  `MissingElement`, `MissingResidue`, `UnknownResidue`, or `SequenceMismatch`.
- **`PdbParseError`** — `EmptyInput`, `MissingAtomField`, or
  `InvalidAtomField`, with stable `pdb.*` codes.
- **`StructureReadError`** — streaming PDB reader error:
  `PdbParse(PdbParseError)` or `Io(std::io::Error)`.
- **`ParsedStructureInput`** — `pub input_hash: String`,
  `pub record: StructureRecord`.
- **`ValidatedStructureInput`** — `pub input_hash: String`,
  `pub report: StructureValidationReport`.

#### Functions

- `pub fn parse_pdb_record(input: &str) -> Result<StructureRecord, PdbParseError>`
  Parse in-memory PDB text.

- `pub fn parse_pdb_record_reader<R: BufRead>(reader: R) -> Result<ParsedStructureInput, StructureReadError>`
  Parse PDB from a buffered reader and return a stable raw input hash.

- `pub fn validate_pdb_reader<R: BufRead>(reader: R) -> Result<StructureValidationReport, StructureReadError>`
  Validate PDB from a buffered reader and discard the raw input hash.

- `pub fn validate_pdb_reader_with_hash<R: BufRead>(reader: R) -> Result<ValidatedStructureInput, StructureReadError>`
  Validate PDB from a buffered reader and include the stable raw input hash.

- `pub fn validate_structure_record(record: &StructureRecord) -> StructureValidationReport`
  Validate an already parsed structure record.

- `pub fn summarize_structure_record(record: &StructureRecord) -> StructureValidationReport`
  Alias for structure validation summaries.

- `pub fn extract_structure_sequences(record: &StructureRecord) -> StructureSequenceOutput`
  Extract per-chain coordinate and SEQRES sequences plus mapping metadata.

### Module: `molecule`

The `molecule` module provides molecular records, SMILES/SDF/MOL2 parsing,
graph validation, conservative valence checks, deterministic graph keys,
descriptors, and hashed fingerprints.

#### Types

- **`MoleculeRecord`** — Parsed molecule with `format`, optional `id`, source
  label, `MoleculeMetadata`, `MolecularGraph`, and preserved source
  `MoleculeProperty` values.
- **`MoleculeMetadata`** — Source line metadata plus atom count, bond count,
  branch count, ring-closure count, disconnected component count, and aromatic
  atom count.
- **`MolecularGraph`** — Split `AtomGraph` and `BondGraph`.
- **`MoleculeAtom`** — Atom index, element, source token/name, aromatic and
  bracket flags, isotope, explicit hydrogens, charge, chirality, atom class,
  optional coordinate, atom type, partial charge, and substructure metadata.
- **`MoleculeBond`** — Bond index, source/target atom indices, `BondOrder`,
  ring-closure marker, and optional stereochemistry marker.
- **`MoleculeCoordinate`** — Optional molecule-space `x`, `y`, `z`
  coordinate.
- **`MoleculeProperty`** — Name/value property preserved from SDF and MOL2
  sources.
- **`MoleculeValidationReport`** — Aggregate molecule validation result.
- **`MoleculeValidationRecord`** — Per-record validation result with derived
  features and structured issue lists.
- **`MoleculeDerivedFeatures`** — `canonical_graph_key`, formula, exact mass,
  heavy/hetero atom counts, ring bond count, rotatable bond count, donor and
  acceptor counts, formal charge, and fingerprint.
- **`MoleculeFingerprint`** — deterministic `biors-ecfp-lite-v0` fingerprint
  with bit count and set bit positions.
- **`MoleculeValidationIssueCode`** — `AromaticityNotVerified`,
  `ValenceExceeded`, or `UnknownValenceModel`.
- **`SmilesParseError`**, **`SdfParseError`**, and **`Mol2ParseError`** —
  format-specific parse errors with stable `smiles.*`, `sdf.*`, and `mol2.*`
  codes.
- **`MoleculeReadError`** — streaming molecule reader error wrapping
  SMILES/SDF/MOL2 parse failures or I/O failures.
- **`ParsedMoleculeInput`** — `pub input_hash: String`,
  `pub records: Vec<MoleculeRecord>`.
- **`ValidatedMoleculeInput`** — `pub input_hash: String`,
  `pub report: MoleculeValidationReport`.

#### Functions

- `pub fn parse_smiles_records(input: &str) -> Result<Vec<MoleculeRecord>, SmilesParseError>`
  Parse line-oriented SMILES records.

- `pub fn parse_smiles_records_reader<R: BufRead>(reader: R) -> Result<ParsedMoleculeInput, MoleculeReadError>`
  Parse SMILES records from a buffered reader and return a stable raw input hash.

- `pub fn parse_sdf_records(input: &str) -> Result<Vec<MoleculeRecord>, SdfParseError>`
  Parse SDF/MOLfile records.

- `pub fn parse_sdf_records_reader<R: BufRead>(reader: R) -> Result<ParsedMoleculeInput, MoleculeReadError>`
  Parse SDF records from a buffered reader and return a stable raw input hash.

- `pub fn parse_mol2_records(input: &str) -> Result<Vec<MoleculeRecord>, Mol2ParseError>`
  Parse Tripos MOL2 records.

- `pub fn parse_mol2_records_reader<R: BufRead>(reader: R) -> Result<ParsedMoleculeInput, MoleculeReadError>`
  Parse MOL2 records from a buffered reader and return a stable raw input hash.

- `pub fn validate_smiles_reader<R: BufRead>(reader: R) -> Result<MoleculeValidationReport, MoleculeReadError>`
  Parse and validate SMILES records.

- `pub fn validate_smiles_reader_with_hash<R: BufRead>(reader: R) -> Result<ValidatedMoleculeInput, MoleculeReadError>`
  Validate SMILES records and include the raw input hash.

- `pub fn validate_molecule_records(records: &[MoleculeRecord]) -> MoleculeValidationReport`
  Validate already parsed molecule records from SMILES, SDF, or MOL2.

- `pub fn summarize_molecule_records(records: &[MoleculeRecord]) -> MoleculeValidationReport`
  Alias for molecule validation summaries.

- `pub fn derive_molecule_features(record: &MoleculeRecord) -> MoleculeDerivedFeatures`
  Compute deterministic graph keys, descriptors, and hashed fingerprints.

### Module: `templates`

The `templates` module exposes local task contracts for common bio-AI workflow
families. Templates are metadata only: they describe required inputs,
validations, model-ready fields, output expectations, and execution
assumptions without running inference, uploading data, or opening network
connections.

#### Types

- **`TaskTemplate`** — Full template contract with `schema_version`, `id`,
  kind, title, summary, supported inputs, validations, model fields, output
  expectations, and execution assumptions.
- **`TaskTemplateKind`** — `ProteinClassification`,
  `ProteinEmbeddingGeneration`, `VariantEffectPrediction`,
  `MoleculePropertyPrediction`, `StructureValidation`, or
  `SequenceSimilarityPreprocess`.
- **`TemplateEntity`** — `ProteinSequence`, `ProteinVariant`, `Molecule`,
  `ProteinStructure`, or `SequenceSet`.
- **`TemplateInputFormat`** — Input format plus core reader support marker.
- **`CoreReaderSupport`** — `Executable` for checked-in parsers/validators or
  `ContractOnly` for normalized field contracts without an executable reader.
- **`TemplateInput`** — Required entity, accepted formats, and normalized field
  names.
- **`TemplateValidation`** — Stable validation id and description.
- **`TemplateModelField`** — Model-ready field name, description, and required
  flag.
- **`TemplateOutputExpectation`** — Output field name, description, and
  required flag.
- **`TemplateExecutionAssumptions`** — Local execution boundary. Current
  templates set `network_access` to `none`, `external_model_calls` to `false`,
  and `uploads_input_data` to `false`.

#### Constants

- **`TASK_TEMPLATE_SCHEMA_VERSION`** — `biors.task_template.v0`.

#### Functions

- `pub fn task_templates() -> &'static [TaskTemplate]`
  Return the stable built-in template catalog.
- `pub fn task_template_ids() -> &'static [&'static str]`
  Return stable template ids in display order.
- `pub fn find_task_template(id: &str) -> Option<&'static TaskTemplate>`
  Look up one built-in template by id.

### Module: `tokenizer`

The `tokenizer` module converts biological sequences into stable token IDs. It supports built-in protein, DNA, and RNA profiles.

#### Types

- **`ProteinTokenizerProfile`** — Built-in profiles.
  - `Protein20`, `Protein20Special`, `DnaIupac`, `DnaIupacSpecial`,
    `RnaIupac`, `RnaIupacSpecial`
  - `pub const fn as_str(self) -> &'static str`
  - `pub const fn default_add_special_tokens(self) -> bool`
  - `pub const fn sequence_kind(self) -> SequenceKind`

- **`ProteinTokenizerConfig`** — JSON tokenizer configuration.
  - `pub profile: ProteinTokenizerProfile`
  - `pub add_special_tokens: bool`

- **`SpecialToken`** — `pub token: String`, `pub token_id: u8`
- **`SpecialTokenSet`** — `pub unk: SpecialToken`, `pub pad: SpecialToken`, `pub cls: SpecialToken`, `pub sep: SpecialToken`, `pub mask: SpecialToken`

- **`ProteinTokenizerInspection`** — Machine-readable inspection output.
  - `pub profile: ProteinTokenizerProfile`
  - `pub config: ProteinTokenizerConfig`
  - `pub vocabulary: Vocabulary`
  - `pub unknown_token_policy: UnknownTokenPolicy`
  - `pub unknown_token_id: u8`
  - `pub special_tokens: SpecialTokenSet`

- **`Tokenizer` trait** — Generic tokenizer interface.
  - `fn alphabet(&self) -> &'static str`
  - `fn vocabulary(&self) -> Vocabulary`
  - `fn tokenize(&self, protein: &ProteinSequence) -> TokenizedProtein`

- **`ProteinTokenizer`** — Default protein tokenizer.
  - `pub fn vocabulary_ref(&self) -> &'static Vocabulary`

- **`TokenizedProtein`** — Tokenized sequence. The legacy type name is retained
  for API compatibility; its `alphabet` field identifies protein, DNA, or RNA
  profiles.
  - `pub id: String`, `pub length: usize`, `pub alphabet: String`, `pub valid: bool`, `pub tokens: Vec<u8>`, `pub warnings: Vec<ResidueIssue>`, `pub errors: Vec<ResidueIssue>`

- **`TokenizedSequence`** — Sequence-generic alias for `TokenizedProtein`.
- **`TokenizerConfig`** — Sequence-generic alias for `ProteinTokenizerConfig`.
- **`TokenizerProfile`** — Sequence-generic alias for `ProteinTokenizerProfile`.
- **`SequenceBatchSummary`** — Sequence-generic alias for `ProteinBatchSummary`.

- **`ProteinBatchSummary`** — Aggregate summary.
  - `pub records: usize`, `pub total_length: usize`, `pub valid_records: usize`, `pub warning_count: usize`, `pub error_count: usize`

- **`TokenizedFastaInput`** — Reader output with hash.
  - `pub input_hash: String`, `pub records: Vec<TokenizedProtein>`

- **`SummarizedFastaInput`** — Reader summary with hash.
  - `pub input_hash: String`, `pub summary: ProteinBatchSummary`

- **`TokenizerError`** — `InvalidVocabJson(String)`
- **`Vocabulary`** — `pub name: String`, `pub tokens: Vec<VocabToken>`, `pub unknown_token_id: u8`, `pub unknown_token_policy: UnknownTokenPolicy`
- **`VocabToken`** — `pub residue: char`, `pub token_id: u8`
- **`UnknownTokenPolicy`** — `WarnOrErrorWithUnknownToken`

**Constants:**

- `pub const PROTEIN_20_UNKNOWN_TOKEN_ID: u8 = 20`

#### Functions

- `pub fn tokenize_protein(protein: &ProteinSequence) -> TokenizedProtein`
  Tokenize with the default protein-20 profile.

- `pub fn tokenize_protein_with_config(protein: &ProteinSequence, config: &ProteinTokenizerConfig) -> TokenizedProtein`
  Tokenize with an explicit profile config.

- `pub fn tokenize_fasta_records(input: &str) -> Result<Vec<TokenizedProtein>, BioRsError>`
  Tokenize FASTA text.

- `pub fn tokenize_fasta_records_reader<R: BufRead>(reader: R) -> Result<TokenizedFastaInput, FastaReadError>`
  Tokenize from a reader with default config.

- `pub fn tokenize_fasta_records_reader_with_config<R: BufRead>(reader: R, config: &ProteinTokenizerConfig) -> Result<TokenizedFastaInput, FastaReadError>`
  Tokenize from a reader with explicit config.

- `pub fn summarize_fasta_records_reader<R: BufRead>(reader: R) -> Result<SummarizedFastaInput, FastaReadError>`
  Summarize without materializing token vectors.

- `pub fn summarize_tokenized_proteins(proteins: &[TokenizedProtein]) -> ProteinBatchSummary`
  Summarize a slice of tokenized proteins.

- `pub fn load_protein_20_vocab() -> &'static Vocabulary`
  Borrow the cached built-in protein-20 vocabulary.

- `pub fn protein_20_vocabulary() -> &'static Vocabulary`
  Same as above.

- `pub fn protein_20_vocab_tokens() -> &'static [VocabToken; 20]`
  Borrow static token definitions.

- `pub fn load_vocab_json(input: &str) -> Result<Vocabulary, TokenizerError>`
  Load vocabulary from JSON.

- `pub fn protein_20_unknown_token_policy() -> UnknownTokenPolicy`
  Return the built-in unknown-token policy.

- `pub fn protein_tokenizer_config_for_profile(profile: ProteinTokenizerProfile) -> ProteinTokenizerConfig`
  Default config for a profile.

- `pub fn load_protein_tokenizer_config_json(input: &str) -> Result<ProteinTokenizerConfig, serde_json::Error>`
  Load config from JSON.

- `pub fn inspect_protein_tokenizer_config(config: &ProteinTokenizerConfig) -> ProteinTokenizerInspection`
  Build inspection output.

### Module: `verification`

The `verification` module handles stable input hashing, content diffs, and package fixture verification.

#### Types

- **`StableInputHasher`** — Incremental FNV-1a hasher.
  - `pub const fn new() -> Self`
  - `pub fn update(&mut self, bytes: &[u8])`
  - `pub fn finalize(self) -> String` — returns `fnv1a64:<hex>`
- `pub fn is_stable_input_hash(value: &str) -> bool` — validates the
  `fnv1a64:<16 lowercase hex>` provenance hash shape used by workflow schemas.

- **`OutputDiffReport`** — Canonical diff between two outputs.
  - `pub expected_path: String`, `pub observed_path: String`, `pub expected_sha256: String`, `pub observed_sha256: String`, `pub matches: bool`, `pub content_diff: Option<ContentMismatchDiff>`

- **`ContentMismatchDiff`** — Byte-level mismatch details.
  - `pub expected_path: String`, `pub observed_path: String`, `pub expected_len: usize`, `pub observed_len: usize`, `pub first_difference: Option<FirstDifference>`

- **`FirstDifference`** — `pub byte_offset: usize`, `pub expected_byte: Option<u8>`, `pub observed_byte: Option<u8>`

- **`FixtureObservation`** — `pub name: String`, `pub path: String`

- **`PackageVerificationReport`** — `pub package: String`, `pub fixtures: usize`, `pub passed: usize`, `pub failed: usize`, `pub results: Vec<FixtureVerificationResult>`

- **`FixtureVerificationResult`** — Per-fixture result.
  - `pub name: String`, `pub input_path: String`, `pub expected_output_path: String`, `pub observed_output_path: Option<String>`, `pub expected_output_hash: Option<String>`, `pub observed_output_hash: Option<String>`, `pub status: VerificationStatus`, `pub checksum_mismatch: bool`, `pub content_mismatch: bool`, `pub issue_code: Option<VerificationIssueCode>`, `pub content_diff: Option<ContentMismatchDiff>`, `pub issue: Option<String>`

- **`VerificationStatus`** — `Failed`, `Missing`, `Passed`
- **`VerificationIssueCode`** — Various mismatch and read failure codes.

#### Functions

- `pub fn stable_input_hash(input: &str) -> String`
  Compute stable FNV-1a hash.

- `pub fn diff_output_bytes(expected_path: &str, observed_path: &str, expected: &[u8], observed: &[u8]) -> OutputDiffReport`
  Build canonical diff. JSON is normalized before comparison.

- `pub fn verify_package_outputs(manifest: &PackageManifest, observations: &[FixtureObservation], manifest_base_dir: &Path) -> PackageVerificationReport`
  Verify fixtures using the package directory for both manifest and observations.

- `pub fn verify_package_outputs_with_observation_base(manifest: &PackageManifest, observations: &[FixtureObservation], manifest_base_dir: &Path, observations_base_dir: &Path) -> PackageVerificationReport`
  Verify with separate base directories.

### Module: `versioning`

The `versioning` module defines schema versioning policies for package and pipeline contracts.

#### Types

- **`SchemaContractPolicy`** — Full policy for one contract.
  - `pub contract: String`, `pub current: String`, `pub supported_versions: Vec<SupportedSchemaVersion>`, `pub deprecation: DeprecationPolicy`, `pub breaking_changes: BreakingChangePolicy`, `pub backward_compatibility_rules: Vec<String>`, `pub migration: MigrationPolicy`

- **`SupportedSchemaVersion`** — `pub version: String`, `pub status: SchemaLifecycleStatus`, `pub introduced_in: String`, optional `deprecated_after`, optional `removed_after`
- **`SchemaLifecycleStatus`** — `Current`, `Supported`, `Deprecated`, `Removed`
- **`DeprecationPolicy`** — `pub notice_minor_releases: u8`, `pub requires_docs: bool`, `pub requires_validator_warning_before_removal: bool`
- **`BreakingChangePolicy`** — `pub require_new_schema_version: bool`, `pub forbidden_in_patch_release: bool`, `pub requires_migration_notes: bool`
- **`MigrationPolicy`** — `pub strategy: String`, `pub automatic_rewrite_allowed: bool`, `pub notes: Vec<String>`
- **`SchemaMigrationPlan`** — `pub from: String`, `pub to: String`, `pub compatibility: Compatibility`, `pub automatic: bool`, `pub required_steps: Vec<String>`
- **`Compatibility`** — `BackwardCompatible`, `MigrationRequired`, `Unsupported`

#### Functions

- `pub fn package_manifest_policy() -> SchemaContractPolicy`
  Return the policy for package manifest schemas.

- `pub fn pipeline_config_policy() -> SchemaContractPolicy`
  Return the policy for pipeline config schemas.

- `pub fn manifest_schema_compatibility(from: SchemaVersion, to: SchemaVersion) -> Compatibility`
  Check compatibility between two manifest schema versions.

- `pub fn manifest_schema_migration_plan(from: SchemaVersion, to: SchemaVersion) -> Option<SchemaMigrationPlan>`
  Build a migration plan, or return `None` if unsupported.

### Module: `workflow`

The `workflow` module orchestrates the end-to-end pipeline: validation, tokenization, and model input construction.

#### Types

- **`SequenceWorkflowOutput`** — End-to-end workflow result.
  - `pub workflow: String`
  - `pub model_ready: bool`
  - `pub provenance: SequenceWorkflowProvenance`
  - `pub validation: SequenceValidationReport`
  - `pub tokenization: TokenizationWorkflowOutput`
  - `pub model_input: Option<ModelInput>`
  - `pub readiness_issues: Vec<SequenceWorkflowReadinessIssue>`

- **`SequenceWorkflowProvenance`** — Reproducibility metadata.
  - `pub biors_core_version: String`
  - `pub invocation: SequenceWorkflowInvocation`
  - `pub input_hash: String`
  - `pub normalization: String`
  - `pub validation_alphabet: String`
  - `pub tokenizer: WorkflowTokenizerMetadata`
  - `pub model_input_policy: ModelInputPolicy`
  - `pub hashes: SequenceWorkflowHashes`

- **`SequenceWorkflowInvocation`** — Captured command or API call.
  - `pub command: String`, `pub arguments: Vec<String>`

- **`SequenceWorkflowHashes`** — `pub vocabulary_sha256: String`, `pub output_data_sha256: String`

- **`WorkflowTokenizerMetadata`** — `pub name: String`, `pub vocab_size: usize`, `pub unknown_token_id: u8`, `pub unknown_token_policy: UnknownTokenPolicy`

- **`TokenizationWorkflowOutput`** — `pub summary: ProteinBatchSummary`, `pub records: Vec<TokenizedProtein>`

- **`SequenceWorkflowReadinessIssue`** — Reason a record is not model-ready.
  - `pub code: String`, `pub id: String`, `pub warning_count: usize`, `pub error_count: usize`, `pub message: String`

#### Functions

- `pub fn prepare_protein_model_input_workflow(input_hash: String, records: &[ProteinSequence], policy: ModelInputPolicy) -> Result<SequenceWorkflowOutput, ModelInputBuildError>`
  Build the default protein validation-to-tokenization-to-model-input workflow.
  Manual `input_hash` values must match `fnv1a64:<16 lowercase hex>`.

- `pub fn prepare_model_input_workflow_with_config(input_hash: String, records: &[ProteinSequence], policy: ModelInputPolicy, tokenizer_config: ProteinTokenizerConfig, invocation: SequenceWorkflowInvocation) -> Result<SequenceWorkflowOutput, ModelInputBuildError>`
  Build the profile-aware validation-to-tokenization-to-model-input workflow for
  protein, DNA, or RNA profiles. Manual
  `input_hash` values must match `fnv1a64:<16 lowercase hex>`.

- `pub fn prepare_protein_model_input_workflow_with_invocation(input_hash: String, records: &[ProteinSequence], policy: ModelInputPolicy, invocation: SequenceWorkflowInvocation) -> Result<SequenceWorkflowOutput, ModelInputBuildError>`
  Build the default protein workflow while capturing the invocation in
  provenance and validating the same stable input-hash shape before
  constructing workflow output.

## Usage Examples

### Parse and validate a protein FASTA string

```rust
use biors_core::fasta::parse_fasta_records;
use biors_core::sequence::validate_protein_sequence;

let input = ">sp|P12345|EXAMPLE\nACDEFGHIKLMNPQRSTVWY\n";
let records = parse_fasta_records(input).expect("valid FASTA");

for protein in &records {
    let report = validate_protein_sequence(protein);
    println!("{}: valid={}", protein.id, report.valid);
}
```

### Tokenize with the protein-20-special profile

```rust
use biors_core::tokenizer::{
    tokenize_protein_with_config, ProteinTokenizerConfig, ProteinTokenizerProfile,
};
use biors_core::sequence::ProteinSequence;

let protein = ProteinSequence::new_normalized("seq1", "ACDE");

let config = ProteinTokenizerConfig {
    profile: ProteinTokenizerProfile::Protein20Special,
    add_special_tokens: true,
};

let tokenized = tokenize_protein_with_config(&protein, &config);
println!("tokens: {:?}", tokenized.tokens);
```

### Build model input with fixed-length padding

```rust
use biors_core::model_input::{
    build_model_inputs_checked, ModelInputPolicy, PaddingPolicy,
};
use biors_core::tokenizer::tokenize_protein;
use biors_core::sequence::ProteinSequence;

let protein = ProteinSequence::new_normalized("seq1", "ACDEFGHIKLMNPQRSTVWY");
let tokenized = tokenize_protein(&protein);

let policy = ModelInputPolicy {
    max_length: 8,
    pad_token_id: 21,
    padding: PaddingPolicy::FixedLength,
};

let model_input = build_model_inputs_checked(&[tokenized], policy).expect("model ready");
println!("input_ids: {:?}", model_input.records[0].input_ids);
```

### Run the full workflow from a reader

```rust
use biors_core::fasta::parse_fasta_records_reader;
use biors_core::workflow::prepare_protein_model_input_workflow;
use biors_core::model_input::{ModelInputPolicy, PaddingPolicy};
use std::io::Cursor;

let data = b">seq1\nACDE\n>seq2\nFGHI\n";
let parsed = parse_fasta_records_reader(Cursor::new(data)).expect("parse ok");

let policy = ModelInputPolicy {
    max_length: 6,
    pad_token_id: 21,
    padding: PaddingPolicy::FixedLength,
};

let output = prepare_protein_model_input_workflow(
    parsed.input_hash,
    &parsed.records,
    policy,
).expect("workflow ok");

println!("model_ready: {}", output.model_ready);
```

### Validate a package manifest

```rust
use biors_core::package::{
    validate_package_manifest, PackageManifest,
};

let manifest: PackageManifest = serde_json::from_str(manifest_json).expect("parse");
let report = validate_package_manifest(&manifest);
println!("valid: {}", report.valid);
```

### Verify package fixtures

```rust
use biors_core::package::{PackageManifest, PackageFixture};
use biors_core::verification::{
    verify_package_outputs, FixtureObservation,
};
use std::path::Path;

let manifest: PackageManifest = serde_json::from_str(json).expect("parse");
let observations = vec![FixtureObservation {
    name: "fixture1".into(),
    path: "observed/output1.json".into(),
}];

let report = verify_package_outputs(&manifest, &observations, Path::new("./package"));
println!("passed: {}/{}", report.passed, report.fixtures);
```

## Migration Guide: JSON Boundary to Direct Library Usage

Many bio-rs users start with the CLI and its JSON envelopes. Moving to direct `biors-core` usage removes serialization overhead and gives you compile-time type safety.

### Before: JSON boundary

```rust
// Spawn CLI, parse JSON stdout
let json = std::process::Command::new("biors")
    .args(["tokenize", "input.fasta"])
    .output()?;
let tokens: serde_json::Value = serde_json::from_slice(&json.stdout)?;
```

### After: Direct library usage

```rust
use biors_core::tokenizer::tokenize_fasta_records;

let input = std::fs::read_to_string("input.fasta")?;
let records = tokenize_fasta_records(&input)?;

// records is Vec<TokenizedProtein> — no JSON, no subprocess
```

### Key differences

| Aspect | JSON boundary | Direct library |
|---|---|---|
| Error handling | Parse CLI exit codes and JSON | Use `Result` with `BioRsError` or `FastaReadError` |
| Types | `serde_json::Value` or custom structs | Native `ProteinSequence`, `TokenizedProtein`, etc. |
| Performance | Includes JSON serialization + CLI startup | Zero overhead beyond the core algorithm |
| Provenance | CLI adds input hashes automatically | Call `StableInputHasher` or use reader APIs |
| Stability | CLI contract is stable per minor version | Rust API may change across minor versions |

### Porting checklist

1. Replace `biors tokenize` with `tokenize_fasta_records` or `tokenize_fasta_records_reader`.
2. Replace `biors validate` with `validate_fasta_reader` or `validate_fasta_reader_with_kind`.
3. Replace `biors workflow` with `prepare_protein_model_input_workflow`.
4. If you relied on JSON envelopes, build your own serializable wrapper around the native types.
5. For input hashing, use `StableInputHasher` directly when you need reproducibility.

## WASM and Python Binding Compatibility

### WASM

`biors-core` compiles for `wasm32-unknown-unknown`. The crate has no `std::fs` dependencies in its public API except through the `package` module's artifact validation functions. If you target WASM:

- FASTA parsing, validation, tokenization, and model input construction work without changes.
- The `package` module's `read_package_file` and `validate_package_manifest_artifacts` functions require filesystem access and will not work in a browser sandbox.
- The `runtime` module's `ExternalProcessBackend` is not available in WASM.
- `sha2` supports WASM, so checksum computation works everywhere.

The repository CI builds `biors-core` for `wasm32-unknown-unknown` on every commit to guarantee this compatibility.

### Python bindings

The `biors-python` crate exposes a PyO3 binding layer over the sequence,
tokenizer, model-input, workflow, package validation, and runtime bridge
planning contracts. The JSON CLI boundary remains the most stable integration
path for production automation; tag releases build and publish Python
distributions alongside the Rust crates.

---

This document reflects the public API of `biors-core` as of version 0.51.0. If you find a discrepancy between this reference and the source, the source is the authority.

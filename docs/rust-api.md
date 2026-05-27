# biors-core Rust API Reference

Version: 0.47.4

This document is the comprehensive public API reference for `biors-core`, the Rust engine behind bio-rs. It covers every public module, type, trait, and function exposed by the crate.

## Table of Contents

- [Overview](#overview)
- [Adding biors-core as a Dependency](#adding-biors-core-as-a-dependency)
- [Stability Guarantees](#stability-guarantees)
- [Feature Flags](#feature-flags)
- [Module Reference](#module-reference)
  - [`error`](#module-error)
  - [`fasta`](#module-fasta)
  - [`fasta_scan`](#module-fasta_scan)
  - [`hash`](#module-hash)
  - [`model_input`](#module-model_input)
  - [`package`](#module-package)
  - [`runtime`](#module-runtime)
  - [`sequence`](#module-sequence)
  - [`service`](#module-service)
  - [`tokenizer`](#module-tokenizer)
  - [`verification`](#module-verification)
  - [`versioning`](#module-versioning)
  - [`workflow`](#module-workflow)
- [Usage Examples](#usage-examples)
- [Migration Guide: JSON Boundary to Direct Library Usage](#migration-guide-json-boundary-to-direct-library-usage)
- [WASM and Python Binding Compatibility](#wasm-and-python-binding-compatibility)

## Overview

`biors-core` is the Rust library that powers bio-rs. It handles biological sequence parsing, validation, tokenization, model input construction, package manifest management, and fixture verification. The crate is designed to be dependency-light and deterministic. It uses `serde` for serialization, `sha2` for checksums, and keeps everything `no_std`-friendly where possible.

The library is organized into focused modules. Each module owns one responsibility: FASTA parsing lives in `fasta`, tokenization lives in `tokenizer`, and package management lives in `package`. This makes the API easy to navigate and test.

## Adding biors-core as a Dependency

Add this to your `Cargo.toml`:

```toml
[dependencies]
biors-core = "0.47.4"
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

`biors-core` currently has no Cargo feature flags. All public APIs are always available.

## Module Reference

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
  Parse FASTA text into normalized protein sequence records.

- `pub fn parse_fasta_records_reader<R: BufRead>(reader: R) -> Result<ParsedFastaInput, FastaReadError>`
  Parse FASTA records from a buffered reader without preloading the full input.

- `pub fn validate_fasta_input(input: &str) -> Result<SequenceValidationReport, BioRsError>`
  Validate FASTA text and return aggregate sequence validation details.

- `pub fn validate_fasta_reader<R: BufRead>(reader: R) -> Result<SequenceValidationReport, FastaReadError>`
  Validate FASTA from a buffered reader and discard the raw input hash.

- `pub fn validate_fasta_reader_with_hash<R: BufRead>(reader: R) -> Result<ValidatedFastaInput, FastaReadError>`
  Validate FASTA from a buffered reader and include a stable raw input hash.

### Module: `fasta_scan`

The `fasta_scan` module contains the ASCII byte-level FASTA scanner. It is the internal engine used by `fasta` and `tokenizer`. As of 0.47.4, this module has no public API. All items are `pub(crate)`. Callers should use the higher-level `fasta` and `tokenizer` entry points instead.

### Module: `hash`

The `hash` module provides canonical SHA-256 digest utilities.

#### Functions

- `pub fn sha256_digest(bytes: &[u8]) -> String`
  Compute a canonical SHA-256 digest in `sha256:<hex>` form. JSON inputs are normalized before hashing so semantically equivalent JSON produces the same digest.

- `pub fn is_sha256_checksum(checksum: &str) -> bool`
  Return true when a checksum uses the supported `sha256:<64 hex>` format.

### Module: `model_input`

The `model_input` module converts tokenized proteins into model-ready arrays with truncation, padding, and attention masks.

#### Types

- **`ModelInputPolicy`** — Policy for converting tokenized proteins into model-ready arrays.
  - `pub max_length: usize`
  - `pub pad_token_id: u8`
  - `pub padding: PaddingPolicy`

- **`PaddingPolicy`** — Padding strategy.
  - `FixedLength` — pad every record to `max_length`
  - `NoPadding` — preserve each record's truncated length without padding

- **`ModelInput`** — Batch of model-ready input records.
  - `pub policy: ModelInputPolicy`
  - `pub records: Vec<ModelInputRecord>`

- **`ModelInputRecord`** — Model-ready representation of one tokenized protein.
  - `pub id: String`
  - `pub input_ids: Vec<u8>`
  - `pub attention_mask: Vec<u8>` — `1` for real tokens, `0` for padding
  - `pub truncated: bool`

- **`ModelInputBuildError`** — Errors from checked builders.
  - `InvalidPolicy { message }`
  - `InvalidTokenizedSequence { id, warning_count, error_count }`

#### Functions

- `pub fn build_model_inputs_unchecked(tokenized: &[TokenizedProtein], policy: ModelInputPolicy) -> ModelInput`
  Build model input without rejecting unresolved tokenization warnings or errors.

- `pub fn build_model_inputs_checked(tokenized: &[TokenizedProtein], policy: ModelInputPolicy) -> Result<ModelInput, ModelInputBuildError>`
  Build model input after rejecting invalid policies and unresolved residue issues.

- `pub fn validate_model_input_policy(policy: &ModelInputPolicy) -> Result<(), ModelInputBuildError>`
  Validate a model-input policy without building records.

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
- **`RuntimeBridgeReport`** — Runtime bridge readiness report.
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

- `pub fn sha256_digest(bytes: &[u8]) -> String`
  Canonical SHA-256 digest.

- `pub fn is_sha256_checksum(checksum: &str) -> bool`
  Validate checksum format.

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

- **`ProteinSequence`** — Named protein sequence.
  - `pub id: String`
  - `pub sequence: Vec<u8>` — normalized, whitespace removed, ASCII uppercased

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

- **`ValidatedSequence`** — Validation result for one protein sequence.
  - `pub id: String`, `pub sequence: String`, `pub alphabet: String`, `pub valid: bool`, `pub warnings: Vec<ResidueIssue>`, `pub errors: Vec<ResidueIssue>`

- **`SequenceValidationReport`** — Aggregate report for protein batches.
  - `pub records: usize`, `pub valid_records: usize`, `pub warning_count: usize`, `pub error_count: usize`, `pub sequences: Vec<ValidatedSequence>`

- **`SequenceRecord`** — Normalized sequence with assigned kind.
  - `pub id: String`, `pub sequence: String`, `pub kind: SequenceKind`
  - `pub fn new(id: impl Into<String>, sequence: impl AsRef<str>, kind: SequenceKind) -> Self`

- **`SequenceValidationIssueCode`** — `AmbiguousSymbol`, `InvalidSymbol` with `pub const fn as_str(self) -> &'static str`
- **`SequenceValidationIssue`** — Kind-aware issue with `pub symbol: char`, `pub position: usize`, `pub kind: SequenceKind`, `pub code: SequenceValidationIssueCode`, `pub message: String`
  - `pub fn ambiguous(symbol: char, position: usize, kind: SequenceKind) -> Self`
  - `pub fn invalid(symbol: char, position: usize, kind: SequenceKind) -> Self`

- **`ValidatedSequenceRecord`** — Kind-aware validation result.
  - `pub id: String`, `pub sequence: String`, `pub kind: SequenceKind`, `pub alphabet: String`, `pub valid: bool`, `pub warnings: Vec<SequenceValidationIssue>`, `pub errors: Vec<SequenceValidationIssue>`

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

- `pub fn validate_protein_sequence(protein: &ProteinSequence) -> ValidatedSequence`
  Validate one protein against the protein-20 policy.

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

### Module: `tokenizer`

The `tokenizer` module converts protein sequences into stable token IDs. It supports the built-in `protein-20` and `protein-20-special` profiles.

#### Types

- **`ProteinTokenizerProfile`** — Built-in profiles.
  - `Protein20`, `Protein20Special`
  - `pub const fn as_str(self) -> &'static str`
  - `pub const fn default_add_special_tokens(self) -> bool`

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

- **`TokenizedProtein`** — Tokenized sequence.
  - `pub id: String`, `pub length: usize`, `pub alphabet: String`, `pub valid: bool`, `pub tokens: Vec<u8>`, `pub warnings: Vec<ResidueIssue>`, `pub errors: Vec<ResidueIssue>`

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
  Tokenize with an explicit config.

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
  Build the stable validation-to-tokenization-to-model-input workflow.

- `pub fn prepare_protein_model_input_workflow_with_invocation(input_hash: String, records: &[ProteinSequence], policy: ModelInputPolicy, invocation: SequenceWorkflowInvocation) -> Result<SequenceWorkflowOutput, ModelInputBuildError>`
  Same as above but captures the invocation in provenance.

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

let protein = ProteinSequence {
    id: "seq1".into(),
    sequence: b"ACDE".to_vec(),
};

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

let protein = ProteinSequence {
    id: "seq1".into(),
    sequence: b"ACDEFGHIKLMNPQRSTVWY".to_vec(),
};
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

This document reflects the public API of `biors-core` as of version 0.47.4. If you find a discrepancy between this reference and the source, the source is the authority.

# 1.0 Public Contract Candidates

The following surfaces are candidates for stabilization before the first stable
release. This document is intentionally narrower than the full generated Rust
API reference: internal parser helpers, low-level scanner internals, backend
implementation details, and experimental provider APIs are not stable
candidates unless they are listed in the stable sections below.

## Stable-Candidate Core Contracts

- FASTA parsing and validation:
  - `parse_fasta_records`
  - `parse_fasta_records_reader`
  - `validate_fasta_input`
  - `validate_fasta_input_with_kind`
  - `validate_fasta_reader`
  - `validate_fasta_reader_with_hash`
  - `validate_fasta_reader_with_kind`
  - `validate_fasta_reader_with_kind_and_hash`
  - `validate_fasta_reader_summary_with_kind_and_hash`
- Sequence kind and validation reports:
  - `SequenceKind`, `SequenceKindSelection`, `AlphabetPolicy`
  - `KindAwareSequenceValidationReport`
  - `KindAwareSequenceValidationSummary`
  - `ValidatedSequenceRecord`
  - `SequenceValidationIssue`
- Protein tokenization:
  - `tokenize_fasta_records`
  - `tokenize_fasta_records_reader`
  - `load_vocab_json`
  - `load_protein_tokenizer_config_json`
  - `protein_20_vocab_tokens`
  - `protein_20_vocabulary`
  - `ProteinTokenizer`, `Tokenizer`
  - `ProteinTokenizerProfile`, `ProteinTokenizerConfig`
  - `ProteinTokenizerInspection`
  - `tokenize_protein_with_config`
  - `tokenize_fasta_records_reader_with_config`
- Model input and workflow:
  - `ModelInput`, `ModelInputPolicy`, `PaddingPolicy`
  - `build_model_inputs_checked`
  - `build_model_inputs_unchecked`
  - `prepare_protein_model_input_workflow`
  - `prepare_protein_model_input_workflow_with_invocation`
  - `SequenceWorkflowOutput`
  - `SequenceWorkflowProvenance`
  - `SequenceWorkflowInvocation`
  - `SequenceWorkflowHashes`
  - `SequenceWorkflowReadinessIssue`
- Output diff and package verification:
  - `diff_output_bytes`, `OutputDiffReport`
  - `PackageVerificationReport`
  - `FixtureObservation`
  - `VerificationIssueCode`
  - `ContentMismatchDiff`
- Package manifest and validation:
  - `validate_package_manifest_artifacts`
  - `PackageManifest`
  - `ModelArtifactMetadata`
  - `ModelArtifactMetadataSummary`
  - `PipelineConfigArtifact`
  - `PackageValidationIssue`
  - `PackageValidationReport`
  - `RuntimeBridgeReport`
  - `BackendCompatibilityCheck`
- Transport-agnostic service contract:
  - `SERVICE_INTERFACE_SCHEMA_VERSION`
  - `ServiceInterfaceDocument`
  - `RuntimeServiceSeparation`
  - `OpenApiDirection`
  - `ServiceRoute`
  - `current_service_interface_document`
  - `service_interface_document`
  - `service_routes`
- Error code surface:
  - `BioRsError::code`

## CLI And JSON Schemas

- CLI success envelope: `ok`, `biors_version`, optional `input_hash`, `data`
- CLI error envelope: `ok=false`, `error.code`, `error.message`,
  `error.location`
- exit code policy
- command list in `docs/cli-contract.md`
- checksum policy: FASTA uses `fnv1a64`, package assets and fixtures use
  `sha256`

Schema candidates:

- `schemas/cli-success.v0.json`
- `schemas/cli-error.v0.json`
- `schemas/tokenize-output.v0.json`
- `schemas/inspect-output.v0.json`
- `schemas/model-input-output.v0.json`
- `schemas/output-diff.v0.json`
- `schemas/dataset-inspect-output.v0.json`
- `schemas/cache-output.v0.json`
- `schemas/pipeline-output.v0.json`
- `schemas/pipeline-config.v0.json`
- `schemas/pipeline-lock.v0.json`
- `schemas/batch-validation-output.v0.json`
- `schemas/sequence-debug-output.v0.json`
- `schemas/tokenizer-inspect-output.v0.json`
- `schemas/tokenizer-conversion-output.v0.json`
- `schemas/sequence-workflow-output.v0.json`
- `schemas/doctor-output.v0.json`
- `schemas/fasta-validation-output.v0.json`
- `schemas/package-inspect-output.v0.json`
- `schemas/package-bridge-output.v0.json`
- `schemas/package-verify-output.v0.json`
- `schemas/package-conversion-output.v0.json`
- `schemas/package-skeleton-output.v0.json`
- `schemas/package-migration-output.v0.json`
- `schemas/package-compatibility-output.v0.json`
- `schemas/package-diff-output.v0.json`
- `schemas/package-manifest.v0.json`
- `schemas/package-manifest.v1.json`
- `schemas/package-validation-report.v0.json`
- `schemas/service-interface-output.v0.json`
- `schemas/service-sequence-validate-request.v0.json`
- `schemas/service-sequence-inspect-request.v0.json`
- `schemas/service-sequence-tokenize-request.v0.json`
- `schemas/service-model-input-request.v0.json`
- `schemas/service-package-request.v0.json`
- `schemas/service-package-compatibility-request.v0.json`

## Binding Contracts

These are promoted surfaces, but their 1.0 shape is still conditional on schema
parity, provenance, validation, and model-input readiness hardening.

- Python package: `packages/rust/biors-python`
  - Python classes and helpers documented in `docs/python-api.md`
  - JSON-returning package helpers should match the shared CLI schemas or
    document a binding-specific schema.
- WASM/npm package: `packages/rust/biors-wasm`
  - TypeScript declarations in `index.d.ts`
  - JavaScript-facing workflow/model-input contracts documented in
    `docs/wasm-api.md`
  - WASM JSON payloads should match shared workflow/model-input schemas where
    they claim parity.
- MCP server: `packages/rust/biors-mcp-server`
  - tool names and JSON payload contracts covered by MCP tests
  - MCP output should either validate against shared schemas or document a
    distinct MCP schema boundary.

## Experimental Runtime And Integration Contracts

The runtime traits and provider integrations are useful extension points, but
they are not stable 1.0 candidates yet unless promoted by a future contract
review.

- Runtime execution traits and reports:
  - `Backend`
  - `BackendConfig`
  - `BackendCapabilities`
  - `BackendCompatibilityReport`
  - `RuntimeCompatibilityIssue`
  - `RuntimeCompatibilityIssueCode`
  - `ExecutionContext`
  - `ExecutionResult`
  - `BackendExecutionError`
- External process adapter:
  - `ExternalProcessBackend`
  - `ExternalProcessConfig`
- Optional Candle backend crate:
  - `CandleBackend`
  - `CandleBackendConfig`
  - `CandleDevice`
  - `CandleInferenceOutput`
  - `CANDLE_MODEL_INPUT_FORMAT`
  - `CANDLE_OUTPUT_FORMAT`
- package runtime bridge provider expansion beyond the current planning targets
- concrete runtime backend implementations beyond smoke-tested local adapters
- larger fixture verification formats
- final shape of schema version migration helpers beyond the current
  `biors_core::versioning` policy API

## Pre-1.0 Unstable Public Rust APIs

The Rust API reference documents every current public `biors-core` item. Public
items not listed in the stable-candidate sections above are still available in
pre-1.0 releases, but they are explicitly unstable until a future contract
review promotes them, hides them, or moves them behind a feature/module boundary.

- Package authoring, migration, and filesystem helpers:
  - `validate_package_manifest`
  - `inspect_package_manifest`
  - `plan_runtime_bridge`
  - `convert_package_manifest`
  - `diff_package_manifests`
  - `plan_package_schema_migration`
  - `read_package_file`
  - `resolve_package_asset_path`
  - `PackageArtifactError`
- Fixture verification and hashing helpers:
  - `stable_input_hash`
  - `StableInputHasher`
  - `verify_package_outputs`
  - `verify_package_outputs_with_observation_base`
  - `FixtureVerificationResult`
  - `VerificationStatus`
- Lower-level model-input structs and validation errors:
  - `validate_model_input_policy`
  - `validate_model_input_payload`
  - `ModelInputRecord`
  - `ModelInputBuildError`
  - `ModelInputPayloadError`
- Lower-level tokenizer helpers and data types:
  - `tokenize_protein`
  - `summarize_fasta_records_reader`
  - `summarize_tokenized_proteins`
  - `load_protein_20_vocab`
  - `protein_20_unknown_token_policy`
  - `SpecialTokenSet`
  - `TokenizedProtein`
  - `Vocabulary`
  - `TokenizerError`
- Versioning policy helpers:
  - `package_manifest_policy`
  - `pipeline_config_policy`
  - `manifest_schema_compatibility`
  - `manifest_schema_migration_plan`
  - versioning policy types

## Not Yet Stable

- internal scanner modules and low-level byte parsing helpers
- benchmark claims beyond the recorded baseline workload
- independent `biors-core` and `biors` versioning outside isolated post-1.0
  patch releases

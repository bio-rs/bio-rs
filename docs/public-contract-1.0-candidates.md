# 1.0 Public Contract Candidates

The following surfaces are candidates for stabilization before the first stable release.

## Rust API

- `parse_fasta_records`
- `parse_fasta_records_reader`
- `validate_fasta_input`
- `validate_fasta_input_with_kind`
- `validate_fasta_reader`
- `validate_fasta_reader_with_hash`
- `validate_fasta_reader_with_kind`
- `validate_fasta_reader_with_kind_and_hash`
- `SequenceKind`, `SequenceKindSelection`, `AlphabetPolicy`
- `KindAwareSequenceValidationReport`, `ValidatedSequenceRecord`, `SequenceValidationIssue`
- `tokenize_fasta_records`
- `tokenize_fasta_records_reader`
- `load_vocab_json`
- `protein_20_vocab_tokens`
- `protein_20_vocabulary`
- `ProteinTokenizer` and `Tokenizer`
- `ModelInput`, `ModelInputPolicy`, `PaddingPolicy`
- `build_model_inputs_checked`
- `build_model_inputs_unchecked`
- `prepare_protein_model_input_workflow`
- `SequenceWorkflowOutput`, `SequenceWorkflowProvenance`, `SequenceWorkflowReadinessIssue`
- `validate_package_manifest_artifacts`
- `PackageManifest`, `PackageValidationIssue`, `PackageValidationReport`, `RuntimeBridgeReport`
- `PackageVerificationReport`, `FixtureObservation`, `VerificationIssueCode`, `ContentMismatchDiff`
- `BioRsError::code`

## CLI

- success envelope: `ok`, `biors_version`, optional `input_hash`, `data`
- error envelope: `ok=false`, `error.code`, `error.message`, `error.location`
- exit code policy
- command list in `docs/cli-contract.md`
- `doctor` diagnostic payload in `schemas/doctor-output.v0.json`
- checksum policy: FASTA uses `fnv1a64`, package assets and fixtures use `sha256`

## Schemas

- `schemas/cli-success.v0.json`
- `schemas/cli-error.v0.json`
- `schemas/tokenize-output.v0.json`
- `schemas/inspect-output.v0.json`
- `schemas/model-input-output.v0.json`
- `schemas/sequence-workflow-output.v0.json`
- `schemas/doctor-output.v0.json`
- `schemas/fasta-validation-output.v0.json`
- `schemas/package-inspect-output.v0.json`
- `schemas/package-bridge-output.v0.json`
- `schemas/package-verify-output.v0.json`
- `schemas/package-manifest.v0.json`
- `schemas/package-validation-report.v0.json`

## Not Yet Stable

- runtime bridge provider expansion beyond `onnx-webgpu`
- larger fixture verification formats
- benchmark claims beyond the recorded baseline workload
- independent `biors-core` and `biors` versioning outside isolated post-1.0 patch releases

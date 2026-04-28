# 1.0 Public Contract Candidates

The following surfaces are candidates for stabilization before `1.0.0`.

## Rust API

- `parse_fasta_records`
- `validate_fasta_input`
- `tokenize_fasta_records`
- `load_vocab_json`
- `ProteinTokenizer` and `Tokenizer`
- `ModelInput`, `ModelInputPolicy`, `PaddingPolicy`
- `validate_package_manifest_artifacts`
- `PackageManifest`, `PackageValidationReport`, `RuntimeBridgeReport`
- `PackageVerificationReport`, `FixtureObservation`
- `BioRsError::code`

## CLI

- success envelope: `ok`, `biors_version`, optional `input_hash`, `data`
- error envelope: `ok=false`, `error.code`, `error.message`, `error.location`
- exit code policy
- command list in `docs/cli-contract.md`
- checksum policy: FASTA uses `fnv1a64`, package assets and fixtures use `sha256`

## Schemas

- `schemas/cli-success.v0.json`
- `schemas/cli-error.v0.json`
- `schemas/tokenize-output.v0.json`
- `schemas/inspect-output.v0.json`
- `schemas/model-input-output.v0.json`
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

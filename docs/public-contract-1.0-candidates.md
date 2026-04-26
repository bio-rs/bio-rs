# 1.0 Public Contract Candidates

The following surfaces are candidates for stabilization before `1.0.0`.

## Rust API

- `parse_fasta_records`
- `validate_fasta_input`
- `tokenize_fasta_records`
- `ProteinTokenizer` and `Tokenizer`
- `ModelInput`, `ModelInputPolicy`, `PaddingPolicy`
- `PackageManifest`, `PackageValidationReport`, `RuntimeBridgeReport`
- `PackageVerificationReport`
- `BioRsError::code`

## CLI

- success envelope: `ok`, `biors_version`, optional `input_hash`, `data`
- error envelope: `ok=false`, `error.code`, `error.message`, `error.location`
- exit code policy
- command list in `docs/cli-contract.md`

## Schemas

- `schemas/cli-success.v0.json`
- `schemas/cli-error.v0.json`
- `schemas/tokenize-output.v0.json`
- `schemas/inspect-output.v0.json`
- `schemas/package-manifest.v0.json`
- `schemas/package-validation-report.v0.json`

## Not Yet Stable

- exact checksum algorithms for package artifacts
- runtime bridge provider expansion beyond `onnx-webgpu`
- larger fixture verification formats
- benchmark claims beyond the recorded baseline workload

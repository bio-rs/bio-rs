# Error Code Registry

Error codes are stable identifiers for CLI JSON error mode.

## FASTA

- `fasta.empty_input`: input is blank after whitespace trimming
- `fasta.missing_identifier`: a FASTA header did not include a non-empty record identifier
- `fasta.missing_header`: non-empty FASTA input did not start with `>`
- `fasta.missing_sequence`: a FASTA record header had no sequence body

## Sequence Validation

Sequence validation warnings and errors are reported inside successful FASTA or
`seq validate` payloads, not as top-level CLI failures.

- `ambiguous_symbol`: a supported ambiguous IUPAC symbol was accepted with a warning
- `invalid_symbol`: a symbol is not supported by the selected Protein, DNA, or RNA policy

## JSON

- `json.invalid`: JSON input could not be decoded
- `json.serialization_failed`: CLI output could not be serialized

## Model Input

- `model_input.invalid_sequence`: a tokenized sequence still contains warnings or errors and cannot be emitted as model-ready input safely
- `model_input.invalid_policy`: model input policy values are invalid, such as `max_length=0`

## Batch

- `batch.no_inputs`: batch validation did not resolve any FASTA files from the provided paths, directories, or glob patterns
- `batch.invalid_glob`: a glob pattern could not be interpreted as a UTF-8 file pattern

## I/O

- `io.read_failed`: input path or stdin could not be read
- `io.write_failed`: CLI output could not be written to stdout

## Package

Package validation also emits typed `structured_issues[*].code` values in validation reports. CLI JSON error codes remain the top-level command failure category.

- `package.invalid_checksum_format`: a package checksum field does not use `sha256:<64hex>`
- `package.checksum_mismatch`: a manifest or verification checksum does not match the file on disk
- `package.invalid_asset_path`: a manifest or observation path is absolute or escapes the package root
- `package.asset_read_failed`: a manifest-relative asset path could not be read
- `package.layout_mismatch`: a manifest v1 asset path is outside the declared package layout
- `package.observed_output_missing`: a verification observation is missing or its output artifact could not be read
- `package.output_content_mismatch`: observed output content does not match the expected output artifact
- `package.validation_failed`: package manifest validation failed
- `package.bridge_not_ready`: package runtime bridge planning found blocking issues
- `package.verification_failed`: fixture observations did not match expected outputs
- `package.migration_unsupported`: no migration plan exists for the requested manifest schema transition
- `package.conversion_missing_metadata`: conversion to manifest v1 is missing required research metadata
- `package.conversion_layout_conflict`: conversion could not infer a v1 package layout that contains the existing artifact paths
- `package.conversion_unsupported`: no conversion exists for the requested manifest schema transition

Package verification reports also expose per-fixture `issue_code` values such
as `observation_missing`, `output_checksum_mismatch`, and
`output_content_mismatch` so callers can inspect fixture-level failures without
parsing the human-readable `issue` field.

## Pipeline

- `pipeline.invalid_config`: a pipeline config is malformed, unsupported, or missing required legacy no-config arguments
- `pipeline.invalid_lock_package`: a package manifest supplied for pipeline lock generation failed package validation
- `pipeline.lock_requires_model_checksum`: a package manifest supplied for pipeline lock generation did not declare `model.checksum`

## Taxonomy

- `fasta.*`: sequence file envelope and record parsing errors
- sequence issue codes: per-record biological sequence validation diagnostics
- `batch.*`: batch input expansion failures
- `json.*`: machine-readable input or output failures
- `io.*`: local filesystem or stdin failures
- `package.*`: portable package contract, runtime, or fixture failures

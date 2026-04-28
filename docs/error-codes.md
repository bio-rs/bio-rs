# Error Code Registry

Error codes are stable identifiers for CLI JSON error mode.

## FASTA

- `fasta.empty_input`: input is blank after whitespace trimming
- `fasta.missing_identifier`: a FASTA header did not include a non-empty record identifier
- `fasta.missing_header`: non-empty FASTA input did not start with `>`
- `fasta.missing_sequence`: a FASTA record header had no sequence body

## JSON

- `json.invalid`: JSON input could not be decoded
- `json.serialization_failed`: CLI output could not be serialized

## Model Input

- `model_input.invalid_sequence`: a tokenized sequence still contains warnings or errors and cannot be emitted as model-ready input safely
- `model_input.invalid_policy`: model input policy values are invalid, such as `max_length=0`

## I/O

- `io.read_failed`: input path or stdin could not be read

## Package

Package validation also emits typed `structured_issues[*].code` values in validation reports. CLI JSON error codes remain the top-level command failure category.

- `package.invalid_checksum_format`: a package checksum field does not use `sha256:<64hex>`
- `package.checksum_mismatch`: a manifest or verification checksum does not match the file on disk
- `package.invalid_asset_path`: a manifest or observation path is absolute or escapes the package root
- `package.asset_read_failed`: a manifest-relative asset path could not be read
- `package.observed_output_missing`: a verification observation is missing or its output artifact could not be read
- `package.output_content_mismatch`: observed output content does not match the expected output artifact
- `package.validation_failed`: package manifest validation failed
- `package.bridge_not_ready`: package runtime bridge planning found blocking issues
- `package.verification_failed`: fixture observations did not match expected outputs

## Taxonomy

- `fasta.*`: sequence file envelope and record parsing errors
- `json.*`: machine-readable input or output failures
- `io.*`: local filesystem or stdin failures
- `package.*`: portable package contract, runtime, or fixture failures

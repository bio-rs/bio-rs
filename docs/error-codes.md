# Error Code Registry

Error codes are stable identifiers for CLI JSON error mode.

## FASTA

- `fasta.empty_input`: input is blank after whitespace trimming
- `fasta.missing_header`: non-empty FASTA input did not start with `>`
- `fasta.missing_sequence`: a FASTA record header had no sequence body

## JSON

- `json.invalid`: JSON input could not be decoded
- `json.serialization_failed`: CLI output could not be serialized

## I/O

- `io.read_failed`: input path or stdin could not be read

## Package

- `package.validation_failed`: package manifest validation failed
- `package.bridge_not_ready`: package runtime bridge planning found blocking issues
- `package.verification_failed`: fixture observations did not match expected outputs

## Taxonomy

- `fasta.*`: sequence file envelope and record parsing errors
- `json.*`: machine-readable input or output failures
- `io.*`: local filesystem or stdin failures
- `package.*`: portable package contract, runtime, or fixture failures

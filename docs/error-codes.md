# Error Code Registry

Error codes are stable identifiers for CLI JSON error mode and structured
validation issues.

## FASTA

- `fasta.empty_input`: input is blank after whitespace trimming
- `fasta.missing_identifier`: a FASTA header did not include a non-empty record identifier
- `fasta.missing_header`: non-empty FASTA input did not start with `>`
- `fasta.missing_sequence`: a FASTA record header had no sequence body

## FASTQ

- `fastq.empty_input`: input contained no FASTQ records
- `fastq.missing_header`: a FASTQ record did not start with an `@` header
- `fastq.missing_identifier`: a FASTQ header did not include a non-empty record identifier
- `fastq.missing_separator`: a FASTQ record ended before a `+` separator line
- `fastq.missing_sequence`: a FASTQ record had no sequence body before the separator
- `fastq.separator_identifier_mismatch`: an optional `+` separator identifier did not match the header identifier
- `fastq.missing_quality`: a FASTQ record ended before enough quality symbols were read
- `fastq.quality_length_mismatch`: a FASTQ record quality string length did not match the sequence length

## PDB

- `pdb.empty_input`: input contained no PDB lines
- `pdb.missing_atom_field`: a required fixed-column ATOM/HETATM field was blank or unavailable
- `pdb.invalid_atom_field`: a fixed-column ATOM/HETATM field could not be parsed as the required numeric or text value

## Molecule Parsing

- `smiles.empty_input`: input contained no SMILES records
- `smiles.missing_smiles`: a SMILES record line did not contain a SMILES token
- `smiles.missing_atom`: a SMILES atom token was expected
- `smiles.unexpected_character`: a SMILES token contained an unsupported character
- `smiles.dangling_bond`: a SMILES bond marker had no following atom or ring closure
- `smiles.invalid_branch`: a SMILES branch opened before any atom was available
- `smiles.unclosed_branch`: a SMILES branch was opened but never closed
- `smiles.unmatched_branch`: a SMILES branch close had no corresponding open branch
- `smiles.unclosed_ring`: a SMILES ring closure was opened but never closed
- `smiles.invalid_ring_closure`: a SMILES ring closure appeared before any atom
- `smiles.invalid_bracket_atom`: a SMILES bracket atom could not be parsed
- `sdf.empty_input`: input contained no SDF records
- `sdf.missing_counts_line`: an SDF record is missing a counts line
- `sdf.invalid_counts_line`: an SDF counts line could not be parsed
- `sdf.invalid_atom_line`: an SDF atom line could not be parsed
- `sdf.invalid_bond_line`: an SDF bond line could not be parsed
- `sdf.unsupported_bond_type`: an SDF bond type is not supported by the graph contract
- `sdf.invalid_v3000_line`: an SDF V3000 atom or bond line could not be parsed
- `mol2.empty_input`: input contained no MOL2 records
- `mol2.missing_molecule_section`: a MOL2 record is missing `@<TRIPOS>MOLECULE`
- `mol2.missing_molecule_name`: a MOL2 record is missing a molecule name
- `mol2.missing_counts_line`: a MOL2 record is missing or mismatches its counts line
- `mol2.invalid_atom_line`: a MOL2 atom line could not be parsed
- `mol2.invalid_bond_line`: a MOL2 bond line could not be parsed
- `mol2.unsupported_bond_type`: a MOL2 bond type is not supported by the graph contract

## Sequence Validation

Sequence validation warnings and errors are reported inside successful FASTA or
`seq validate` payloads, and inside successful FASTQ validation payloads, not as
top-level CLI failures. The same values are used by Rust `Diagnostic::code()`,
CLI JSON payloads, schemas, and WASM TypeScript declarations.

- `ambiguous_symbol`: a supported ambiguous IUPAC symbol was accepted with a warning
- `invalid_symbol`: a symbol is not supported by the selected Protein, DNA, or RNA policy
- `invalid_quality_character`: a FASTQ quality symbol is outside printable Phred+33 ASCII

## Structure Validation

Structure validation warnings and errors are reported inside successful
`structure validate --format pdb` payloads, not as top-level CLI failures.

- `no_coordinate_chains`: the structure contains no coordinate-bearing chains
- `invalid_coordinate`: a coordinate value is not finite
- `invalid_occupancy`: an atom occupancy value is structurally impossible, such as a negative value
- `suspicious_occupancy`: an atom occupancy value is unusual but not fatal, such as a value greater than `1.0`
- `missing_element`: an atom record omitted the element symbol
- `missing_residue`: a residue is annotated as missing from coordinates
- `unknown_residue`: a coordinate residue could not be mapped to a standard protein one-letter code
- `sequence_mismatch`: coordinate-derived protein sequence could not be mapped to SEQRES

## Molecule Validation

Molecule validation warnings and errors are reported inside successful
`molecule validate --format smiles|sdf|mol2` payloads, not as top-level CLI
failures.

- `aromaticity_not_verified`: aromatic source notation was preserved but not independently perceived from first principles
- `valence_exceeded`: a parsed atom exceeds the conservative valence model used by bio-rs
- `unknown_valence_model`: a parsed atom has no configured conservative valence model

## JSON

- `json.invalid`: JSON input could not be decoded
- `json.serialization_failed`: CLI output could not be serialized

## Reports

- `report.invalid_json`: report input could not be parsed as JSON
- `report.output_stdout_ambiguous`: Markdown report output was set to `-`, but stdout is reserved for the CLI JSON envelope
- `report.shareable_json_stdout_ambiguous`: shareable report JSON output was set to `-`, but stdout is reserved for the CLI JSON envelope

## Model Input

- `model_input.invalid_sequence`: a tokenized sequence still contains warnings or errors and cannot be emitted as model-ready input safely
- `model_input.invalid_policy`: model input policy values are invalid, such as `max_length=0`
- `model_input.fixed_length_mismatch`: fixed-length model-input payload record length does not equal `policy.max_length`
- `model_input.no_padding_length_exceeded`: no-padding model-input payload record length exceeds `policy.max_length`
- `model_input.length_mismatch`: model-input `input_ids` and `attention_mask` lengths differ
- `model_input.non_binary_attention_mask`: model-input `attention_mask` contains a value other than `0` or `1`
- `model_input.empty_attention_mask`: model-input record has no unmasked token
- `workflow.invalid_input_hash`: workflow provenance input hash does not match `fnv1a64:<16 lowercase hex>`

## Batch

- `batch.no_inputs`: batch validation did not resolve any FASTA files from the provided paths, directories, or glob patterns
- `batch.invalid_glob`: a glob pattern could not be interpreted as a UTF-8 file pattern

## Dataset

- `dataset.no_inputs`: dataset inspection did not resolve any FASTA files from the provided paths, directories, or glob patterns
- `dataset.invalid_glob`: a dataset glob pattern could not be interpreted as a UTF-8 file pattern
- `dataset.invalid_metadata`: dataset metadata was not supplied as non-empty `key=value`
- `dataset.duplicate_metadata_key`: dataset metadata supplied the same normalized key more than once

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
- `package.invalid_pipeline_config`: a manifest-referenced pipeline config artifact is present but cannot be parsed or validated
- `package.invalid_tokenizer_config`: a manifest-referenced tokenizer config artifact is present but cannot be parsed or does not match manifest tokenizer metadata
- `package.invalid_vocab_config`: a manifest-referenced vocabulary artifact is present but cannot be parsed or does not match manifest vocab metadata
- `package.observed_output_missing`: a verification observation is missing or its output artifact could not be read
- `package.output_content_mismatch`: observed output content does not match the expected output artifact
- `package.validation_failed`: package manifest validation failed
- `package.bridge_not_ready`: package runtime bridge planning found blocking issues
- `package.verification_failed`: fixture observations did not match expected outputs
- `package.migration_unsupported`: no migration plan exists for the requested manifest schema transition
- `package.conversion_missing_metadata`: conversion to manifest v1 is missing required research metadata
- `package.conversion_layout_conflict`: conversion could not infer a v1 package layout that contains the existing artifact paths
- `package.conversion_unsupported`: no conversion exists for the requested manifest schema transition
- `package.init_exists`: package initialization would overwrite an existing manifest without `--force`
- `package.init_invalid_path`: package initialization received an asset path that cannot be represented inside the package layout
- `package.init_missing_metadata`: package initialization is missing required research metadata
- `package.project_model_missing`: Python project conversion could not find an ONNX model and no `--model` override was supplied
- `package.project_tokenizer_config_ambiguous`: Python project conversion found multiple non-generated `tokenizer_config.json` candidates and requires `--tokenizer-config`

Package verification reports also expose per-fixture `issue_code` values such
as `observation_missing`, `output_checksum_mismatch`, and
`output_content_mismatch` so callers can inspect fixture-level failures without
parsing the human-readable `issue` field.

## Runtime

Runtime errors are Rust API errors in the `biors_core::runtime` abstraction.
They are not emitted by the CLI until a concrete backend command exists.

- `runtime.unsupported_input`: a backend does not accept the supplied input payload format
- `runtime.unsupported_output`: a backend does not produce the requested output payload format
- `runtime.output_format_mismatch`: a backend returned a supported output format different from the requested output format
- `runtime.payload_too_large`: a backend payload exceeds the declared byte limit
- `runtime.execution_failed`: a backend failed while handling an execution context
- `runtime.process_spawn_failed`: an external process backend could not start the configured program
- `runtime.process_io_failed`: an external process backend failed while writing stdin, reading output, or waiting for the child
- `runtime.process_timeout`: an external process exceeded the configured wall-clock timeout and was terminated
- `runtime.process_exit_failed`: an external process exited with a non-zero status
- `runtime.process_stdout_too_large`: an external process wrote more stdout than the configured result limit
- `runtime.process_stderr_too_large`: an external process wrote more stderr than the configured diagnostic limit
- `runtime.process_invalid_output`: an external process stdout payload was not a valid `ExecutionResult` JSON document

## Candle Backend

Candle backend codes are crate-local Rust API diagnostics from
`biors-backend-candle`. When the backend is called through the generic runtime
trait, these lower-level messages are wrapped by `runtime.execution_failed`.
Model-input validation failures keep the shared `model_input.*` codes.

- `candle.load_failed`: safetensors weights could not be loaded
- `candle.missing_tensor`: a configured tensor name was not present
- `candle.invalid_shape`: embedding, projection, or bias tensor shape is not supported
- `candle.invalid_dtype`: a configured tensor is not floating point
- `candle.token_id_out_of_range`: an unmasked token id exceeds the embedding vocabulary
- `candle.tensor_failed`: Candle tensor construction failed before inference
- `candle.inference_failed`: Candle embedding, pooling, projection, or bias execution failed
- `candle.output_failed`: Candle output conversion failed

## Pipeline

- `pipeline.invalid_config`: a pipeline config is malformed, unsupported, or missing required legacy no-config arguments
- `pipeline.invalid_lock_package`: a package manifest supplied for pipeline lock generation failed package validation
- `pipeline.lock_config_not_in_package`: `--package` was supplied for lock generation but `--config` is not one of the package-declared pipeline config artifacts
- `pipeline.lock_requires_model_checksum`: a package manifest supplied for pipeline lock generation did not declare `model.checksum`

## Tokenizer

- `tokenizer.conversion_invalid_config`: a tokenizer conversion input was not a supported Hugging Face tokenizer config object

## Taxonomy

- `fasta.*`: sequence file envelope and record parsing errors
- `fastq.*`: FASTQ envelope and record parsing errors
- `pdb.*`: PDB coordinate record parsing errors
- sequence issue codes: per-record biological sequence validation diagnostics
- structure issue codes: per-chain and per-atom structure validation diagnostics
- `batch.*`: batch input expansion failures
- `dataset.*`: shared dataset/file input resolution failures
- `json.*`: machine-readable input or output failures
- `report.*`: reproducible report input or output option failures
- `io.*`: local filesystem or stdin failures
- `package.*`: portable package contract, runtime, or fixture failures
- `runtime.*`: backend execution abstraction failures

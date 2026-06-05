use std::io::BufRead;

use crate::structure::types::{StructureValidationReport, ValidatedStructureInput};
use crate::structure::validation::validate_structure_record;

use super::error::StructureReadError;
use super::parse::parse_pdb_stream;

/// Validate a PDB record from a buffered reader and discard the raw input hash.
pub fn validate_pdb_reader<R: BufRead>(
    reader: R,
) -> Result<StructureValidationReport, StructureReadError> {
    Ok(validate_pdb_reader_with_hash(reader)?.report)
}

/// Validate a PDB record from a buffered reader and include a stable raw input hash.
pub fn validate_pdb_reader_with_hash<R: BufRead>(
    reader: R,
) -> Result<ValidatedStructureInput, StructureReadError> {
    let parsed = parse_pdb_stream(reader)?;
    Ok(ValidatedStructureInput {
        input_hash: parsed.input_hash,
        report: validate_structure_record(&parsed.record),
    })
}

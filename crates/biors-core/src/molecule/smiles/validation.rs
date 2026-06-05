use std::io::BufRead;

use crate::molecule::{MoleculeReadError, MoleculeValidationReport, ValidatedMoleculeInput};

use super::parse::parse_smiles_stream;

/// Validate SMILES records from a buffered reader and discard the raw input hash.
pub fn validate_smiles_reader<R: BufRead>(
    reader: R,
) -> Result<MoleculeValidationReport, MoleculeReadError> {
    Ok(validate_smiles_reader_with_hash(reader)?.report)
}

/// Validate SMILES records from a buffered reader and include a stable raw input hash.
pub fn validate_smiles_reader_with_hash<R: BufRead>(
    reader: R,
) -> Result<ValidatedMoleculeInput, MoleculeReadError> {
    let parsed = parse_smiles_stream(reader)?;
    Ok(ValidatedMoleculeInput {
        input_hash: parsed.input_hash,
        report: crate::molecule::validate_molecule_records(&parsed.records),
    })
}

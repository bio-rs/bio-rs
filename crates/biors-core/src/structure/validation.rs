mod mapping;

use super::types::{
    Chain, ProteinStructureMappingStatus, StructureChainReport, StructureRecord,
    StructureSequenceChain, StructureSequenceOutput, StructureValidationIssue,
    StructureValidationIssueCode, StructureValidationReport,
};
use mapping::map_coordinate_to_seqres;

/// Validate a parsed structure record.
pub fn validate_structure_record(record: &StructureRecord) -> StructureValidationReport {
    let mut warnings = Vec::new();
    let mut errors = Vec::new();
    if record.chains.is_empty() {
        errors.push(StructureValidationIssue::new(
            StructureValidationIssueCode::NoCoordinateChains,
            "structure does not contain coordinate-bearing chains",
        ));
    }

    let chain_reports: Vec<_> = record
        .chains
        .iter()
        .map(|chain| validate_chain(chain, &mut warnings, &mut errors))
        .collect();

    StructureValidationReport {
        format: record.format,
        valid: errors.is_empty(),
        chains: record.chains.len(),
        residues: record.chains.iter().map(|chain| chain.residues.len()).sum(),
        atoms: record.metadata.atom_count,
        hetero_atoms: record.metadata.hetero_atom_count,
        missing_residues: record.metadata.missing_residue_count,
        warning_count: warnings.len(),
        error_count: errors.len(),
        chain_reports,
        warnings,
        errors,
    }
}

/// Summarize a parsed structure record with validation diagnostics.
pub fn summarize_structure_record(record: &StructureRecord) -> StructureValidationReport {
    validate_structure_record(record)
}

/// Extract chain-level protein sequences and SEQRES mappings from a structure.
pub fn extract_structure_sequences(record: &StructureRecord) -> StructureSequenceOutput {
    StructureSequenceOutput {
        format: record.format,
        chains: record
            .chains
            .iter()
            .map(|chain| StructureSequenceChain {
                chain_id: chain.id.clone(),
                coordinate_sequence: chain.coordinate_sequence.clone(),
                seqres_sequence: chain.seqres_sequence.clone(),
                coordinate_residues: chain.coordinate_sequence.chars().count(),
                seqres_residues: chain
                    .seqres_sequence
                    .as_ref()
                    .map(|sequence| sequence.chars().count()),
                missing_residues: chain.missing_residues.clone(),
                mapping: map_coordinate_to_seqres(
                    &chain.coordinate_sequence,
                    &chain.seqres_sequence,
                ),
            })
            .collect(),
    }
}

fn validate_chain(
    chain: &Chain,
    warnings: &mut Vec<StructureValidationIssue>,
    errors: &mut Vec<StructureValidationIssue>,
) -> StructureChainReport {
    for missing in &chain.missing_residues {
        warnings.push(
            StructureValidationIssue::new(
                StructureValidationIssueCode::MissingResidue,
                format!(
                    "SEQRES residue {} {}{} is annotated as missing from coordinates",
                    missing.name, chain.id, missing.sequence_number
                ),
            )
            .with_chain(chain.id.clone())
            .with_residue(missing.sequence_number),
        );
    }

    let mut atom_count = 0usize;
    for residue in &chain.residues {
        if !residue.hetero && residue.one_letter_code == Some('X') {
            warnings.push(
                StructureValidationIssue::new(
                    StructureValidationIssueCode::UnknownResidue,
                    format!(
                        "residue {} {}{} is not a standard protein residue",
                        residue.name, chain.id, residue.sequence_number
                    ),
                )
                .with_chain(chain.id.clone())
                .with_residue(residue.sequence_number),
            );
        }
        for atom in &residue.atoms {
            atom_count += 1;
            validate_atom(chain, residue.sequence_number, atom, warnings, errors);
        }
    }

    let mapping = map_coordinate_to_seqres(&chain.coordinate_sequence, &chain.seqres_sequence);
    if mapping.status == ProteinStructureMappingStatus::Mismatch {
        errors.push(
            StructureValidationIssue::new(
                StructureValidationIssueCode::SequenceMismatch,
                mapping.message.clone(),
            )
            .with_chain(chain.id.clone()),
        );
    }

    StructureChainReport {
        chain_id: chain.id.clone(),
        residues: chain.residues.len(),
        atoms: atom_count,
        coordinate_sequence_length: chain.coordinate_sequence.chars().count(),
        seqres_sequence_length: chain
            .seqres_sequence
            .as_ref()
            .map(|sequence| sequence.chars().count()),
        missing_residues: chain.missing_residues.len(),
        sequence_mapping: mapping,
    }
}

fn validate_atom(
    chain: &Chain,
    residue_number: i32,
    atom: &super::types::Atom,
    warnings: &mut Vec<StructureValidationIssue>,
    errors: &mut Vec<StructureValidationIssue>,
) {
    let coordinate = atom.coordinate;
    if !coordinate.x.is_finite() || !coordinate.y.is_finite() || !coordinate.z.is_finite() {
        errors.push(
            StructureValidationIssue::new(
                StructureValidationIssueCode::InvalidCoordinate,
                format!("atom {} has a non-finite coordinate", atom.serial),
            )
            .with_chain(chain.id.clone())
            .with_residue(residue_number)
            .with_atom(atom.serial),
        );
    }
    if let Some(occupancy) = atom.occupancy {
        if occupancy < 0.0 {
            errors.push(
                StructureValidationIssue::new(
                    StructureValidationIssueCode::InvalidOccupancy,
                    format!("atom {} has negative occupancy {occupancy}", atom.serial),
                )
                .with_chain(chain.id.clone())
                .with_residue(residue_number)
                .with_atom(atom.serial),
            );
        } else if occupancy > 1.0 {
            warnings.push(
                StructureValidationIssue::new(
                    StructureValidationIssueCode::SuspiciousOccupancy,
                    format!(
                        "atom {} has occupancy {occupancy} greater than 1.0",
                        atom.serial
                    ),
                )
                .with_chain(chain.id.clone())
                .with_residue(residue_number)
                .with_atom(atom.serial),
            );
        }
    }
    if atom.element.is_none() {
        warnings.push(
            StructureValidationIssue::new(
                StructureValidationIssueCode::MissingElement,
                format!("atom {} is missing an element symbol", atom.serial),
            )
            .with_chain(chain.id.clone())
            .with_residue(residue_number)
            .with_atom(atom.serial),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::formats::BioFormat;

    #[test]
    fn sequence_output_uses_pdb_format() {
        let output = extract_structure_sequences(&StructureRecord {
            format: BioFormat::Pdb,
            id: None,
            metadata: crate::structure::StructureMetadata {
                title: None,
                line_count: 0,
                model_count: 1,
                atom_count: 0,
                hetero_atom_count: 0,
                seqres_chain_count: 0,
                missing_residue_count: 0,
            },
            chains: Vec::new(),
        });

        assert_eq!(output.format, BioFormat::Pdb);
        assert!(output.chains.is_empty());
    }
}

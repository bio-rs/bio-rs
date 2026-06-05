use crate::formats::BioFormat;
use crate::molecule::derive_molecule_features;

use super::types::{
    BondOrder, MoleculeAtom, MoleculeBond, MoleculeRecord, MoleculeValidationIssue,
    MoleculeValidationIssueCode, MoleculeValidationRecord, MoleculeValidationReport,
};

/// Validate parsed molecule records.
pub fn validate_molecule_records(records: &[MoleculeRecord]) -> MoleculeValidationReport {
    let record_reports: Vec<_> = records.iter().map(validate_molecule_record).collect();
    MoleculeValidationReport {
        format: records
            .first()
            .map(|record| record.format)
            .unwrap_or(BioFormat::Smiles),
        valid: record_reports.iter().all(|record| record.valid),
        records: record_reports.len(),
        valid_records: record_reports.iter().filter(|record| record.valid).count(),
        atom_count: records
            .iter()
            .map(|record| record.metadata.atom_count)
            .sum(),
        bond_count: records
            .iter()
            .map(|record| record.metadata.bond_count)
            .sum(),
        warning_count: record_reports
            .iter()
            .map(|record| record.warnings.len())
            .sum(),
        error_count: record_reports
            .iter()
            .map(|record| record.errors.len())
            .sum(),
        record_reports,
    }
}

/// Summarize parsed molecule records with validation diagnostics.
pub fn summarize_molecule_records(records: &[MoleculeRecord]) -> MoleculeValidationReport {
    validate_molecule_records(records)
}

fn validate_molecule_record(record: &MoleculeRecord) -> MoleculeValidationRecord {
    let mut warnings = Vec::new();
    let mut errors = Vec::new();
    if record.metadata.aromatic_atom_count > 0
        || record
            .graph
            .bonds
            .bonds
            .iter()
            .any(|bond| bond.order == BondOrder::Aromatic)
    {
        warnings.push(MoleculeValidationIssue::new(
            MoleculeValidationIssueCode::AromaticityNotVerified,
            "aromatic atoms and bonds are parsed, but Huckel electron assignment is not verified",
        ));
    }
    validate_valence(record, &mut warnings, &mut errors);

    MoleculeValidationRecord {
        id: record.id.clone(),
        source: record.source.clone(),
        metadata: record.metadata.clone(),
        valid: errors.is_empty(),
        atoms: record.metadata.atom_count,
        bonds: record.metadata.bond_count,
        derived: derive_molecule_features(record),
        warnings,
        errors,
    }
}

fn validate_valence(
    record: &MoleculeRecord,
    warnings: &mut Vec<MoleculeValidationIssue>,
    errors: &mut Vec<MoleculeValidationIssue>,
) {
    for atom in &record.graph.atoms.atoms {
        let Some(max_scaled_valence) = max_scaled_valence(atom) else {
            warnings.push(
                MoleculeValidationIssue::new(
                    MoleculeValidationIssueCode::UnknownValenceModel,
                    format!(
                        "no conservative valence model is configured for atom {} ({})",
                        atom.index, atom.element
                    ),
                )
                .with_atom(atom.index),
            );
            continue;
        };
        let used = scaled_bond_order_sum(atom.index, &record.graph.bonds.bonds)
            + u16::from(atom.explicit_hydrogens) * 2;
        if used > max_scaled_valence {
            errors.push(
                MoleculeValidationIssue::new(
                    MoleculeValidationIssueCode::ValenceExceeded,
                    format!(
                        "atom {} ({}) uses valence {:.1}, exceeding conservative maximum {:.1}",
                        atom.index,
                        atom.element,
                        f32::from(used) / 2.0,
                        f32::from(max_scaled_valence) / 2.0
                    ),
                )
                .with_atom(atom.index),
            );
        }
    }
}

fn scaled_bond_order_sum(atom_index: usize, bonds: &[MoleculeBond]) -> u16 {
    bonds
        .iter()
        .filter(|bond| bond.source_atom == atom_index || bond.target_atom == atom_index)
        .map(|bond| scaled_order(bond.order))
        .sum()
}

fn scaled_order(order: BondOrder) -> u16 {
    match order {
        BondOrder::Single => 2,
        BondOrder::Double => 4,
        BondOrder::Triple => 6,
        BondOrder::Quadruple => 8,
        BondOrder::Aromatic => 3,
    }
}

fn max_scaled_valence(atom: &MoleculeAtom) -> Option<u16> {
    let max = match atom.element.as_str() {
        "*" => return None,
        "H" | "F" | "Cl" | "Br" | "I" => 2,
        "B" => 6,
        "C" | "Si" => 8,
        "N" if atom.charge > 0 => 8,
        "N" => 6,
        "O" => 4,
        "P" | "As" => 10,
        "S" | "Se" => 12,
        _ => return None,
    };
    Some(max)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::molecule::parse_smiles_records;

    #[test]
    fn valence_allows_common_organic_smiles() {
        let records =
            parse_smiles_records("CC(=O)O acetate\n[NH4+] ammonium\n").expect("parse smiles");
        let report = validate_molecule_records(&records);

        assert!(report.valid);
        assert_eq!(report.records, 2);
        assert_eq!(report.error_count, 0);
    }

    #[test]
    fn valence_reports_overfilled_carbon() {
        let records = parse_smiles_records("C(=O)(=O)(=O)\n").expect("parse smiles");
        let report = validate_molecule_records(&records);

        assert!(!report.valid);
        assert_eq!(
            report.record_reports[0].errors[0].code,
            MoleculeValidationIssueCode::ValenceExceeded
        );
    }

    #[test]
    fn aromatic_input_reports_limited_aromaticity_validation() {
        let records = parse_smiles_records("c1ccccc1 benzene\n").expect("parse smiles");
        let report = validate_molecule_records(&records);

        assert!(report.valid);
        assert!(report.record_reports[0].warnings.iter().any(|warning| {
            warning.code == MoleculeValidationIssueCode::AromaticityNotVerified
        }));
        assert!(report.record_reports[0]
            .derived
            .canonical_graph_key
            .starts_with("biors-graph-v0;"));
    }
}

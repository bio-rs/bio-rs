use super::{
    is_ambiguous_residue, is_protein_20_residue, ProteinSequence, ResidueIssue, ValidatedSequence,
    PROTEIN_20,
};

/// Validate one normalized protein sequence against the `protein-20` policy.
pub fn validate_protein_sequence(protein: &ProteinSequence) -> ValidatedSequence {
    let mut warnings = Vec::new();
    let mut errors = Vec::new();

    for (index, residue) in protein.sequence.chars().enumerate() {
        let position = index + 1;
        if is_protein_20_residue(residue) {
            continue;
        }

        if is_ambiguous_residue(residue) {
            warnings.push(ResidueIssue { residue, position });
        } else {
            errors.push(ResidueIssue { residue, position });
        }
    }

    ValidatedSequence {
        id: protein.id.clone(),
        sequence: protein.sequence.clone(),
        alphabet: PROTEIN_20.to_string(),
        valid: warnings.is_empty() && errors.is_empty(),
        warnings,
        errors,
    }
}

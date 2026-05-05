use super::{
    is_ambiguous_residue, is_ambiguous_residue_byte, is_protein_20_residue,
    is_protein_20_residue_byte,
};
use super::{
    AlphabetPolicy, ProteinSequence, ResidueIssue, SequenceRecord, SequenceValidationIssue,
    SymbolClass, ValidatedSequence, ValidatedSequenceRecord, PROTEIN_20,
};

/// Validate one normalized protein sequence against the `protein-20` policy.
pub fn validate_protein_sequence(protein: &ProteinSequence) -> ValidatedSequence {
    validate_protein_sequence_owned(protein.id.clone(), protein.sequence.clone())
}

pub(crate) fn validate_protein_sequence_owned(
    id: String,
    sequence: String,
) -> ValidatedSequence {
    let mut warnings = Vec::new();
    let mut errors = Vec::new();

    if sequence.is_ascii() {
        for (index, byte) in sequence.bytes().enumerate() {
            let position = index + 1;
            if is_protein_20_residue_byte(byte) {
                continue;
            }

            let residue = byte as char;
            if is_ambiguous_residue_byte(byte) {
                warnings.push(ResidueIssue { residue, position });
            } else {
                errors.push(ResidueIssue { residue, position });
            }
        }
    } else {
        for (index, residue) in sequence.chars().enumerate() {
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
    }

    ValidatedSequence {
        id,
        sequence,
        alphabet: PROTEIN_20.to_string(),
        valid: warnings.is_empty() && errors.is_empty(),
        warnings,
        errors,
    }
}

/// Validate one normalized biological sequence against its assigned alphabet policy.
pub fn validate_sequence_record(record: &SequenceRecord) -> ValidatedSequenceRecord {
    let policy = AlphabetPolicy::for_kind(record.kind);
    let mut warnings = Vec::new();
    let mut errors = Vec::new();
    let mut sequence = String::with_capacity(record.sequence.len());
    let mut position = 0;

    if record.sequence.is_ascii() {
        for byte in record.sequence.bytes() {
            if byte.is_ascii_whitespace() {
                continue;
            }
            let symbol = byte.to_ascii_uppercase() as char;
            push_kind_issue(symbol, &mut position, policy, &mut warnings, &mut errors);
            sequence.push(symbol);
        }
    } else {
        for symbol in super::normalized_residues(&record.sequence) {
            push_kind_issue(symbol, &mut position, policy, &mut warnings, &mut errors);
            sequence.push(symbol);
        }
    }

    ValidatedSequenceRecord {
        id: record.id.clone(),
        sequence,
        kind: record.kind,
        alphabet: policy.name().to_string(),
        valid: warnings.is_empty() && errors.is_empty(),
        warnings,
        errors,
    }
}

fn push_kind_issue(
    symbol: char,
    position: &mut usize,
    policy: AlphabetPolicy,
    warnings: &mut Vec<SequenceValidationIssue>,
    errors: &mut Vec<SequenceValidationIssue>,
) {
    *position += 1;
    match policy.classify(symbol) {
        SymbolClass::Standard => {}
        SymbolClass::Ambiguous => {
            warnings.push(SequenceValidationIssue::ambiguous(
                symbol,
                *position,
                policy.kind(),
            ));
        }
        SymbolClass::Invalid => {
            errors.push(SequenceValidationIssue::invalid(
                symbol,
                *position,
                policy.kind(),
            ));
        }
    }
}

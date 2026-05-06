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

pub(crate) fn validate_protein_sequence_owned(id: String, sequence: Vec<u8>) -> ValidatedSequence {
    let mut warnings = Vec::with_capacity(sequence.len());
    let mut errors = Vec::with_capacity(sequence.len());

    if sequence.is_ascii() {
        for (index, byte) in sequence.iter().enumerate() {
            push_protein_byte_issue(*byte, index + 1, &mut warnings, &mut errors);
        }
    } else if let Ok(s) = std::str::from_utf8(&sequence) {
        for (index, residue) in s.chars().enumerate() {
            push_protein_issue(residue, index + 1, &mut warnings, &mut errors);
        }
    } else {
        for (index, byte) in sequence.iter().enumerate() {
            push_protein_byte_issue(*byte, index + 1, &mut warnings, &mut errors);
        }
    }

    let normalized_sequence = String::from_utf8(sequence)
        .unwrap_or_else(|e| String::from_utf8_lossy(e.as_bytes()).into_owned());

    ValidatedSequence {
        id,
        sequence: normalized_sequence,
        alphabet: PROTEIN_20.to_string(),
        valid: warnings.is_empty() && errors.is_empty(),
        warnings,
        errors,
    }
}

fn push_protein_issue(
    residue: char,
    position: usize,
    warnings: &mut Vec<ResidueIssue>,
    errors: &mut Vec<ResidueIssue>,
) {
    if is_protein_20_residue(residue) {
        return;
    }

    if is_ambiguous_residue(residue) {
        warnings.push(ResidueIssue { residue, position });
    } else {
        errors.push(ResidueIssue { residue, position });
    }
}

fn push_protein_byte_issue(
    byte: u8,
    position: usize,
    warnings: &mut Vec<ResidueIssue>,
    errors: &mut Vec<ResidueIssue>,
) {
    if is_protein_20_residue_byte(byte) {
        return;
    }

    let residue = char::from(byte);
    if is_ambiguous_residue_byte(byte) {
        warnings.push(ResidueIssue { residue, position });
    } else {
        errors.push(ResidueIssue { residue, position });
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sequence::ProteinSequence;

    #[test]
    fn validate_protein_sequence_accepts_all_standard_residues() {
        let protein = ProteinSequence {
            id: "std20".into(),
            sequence: b"ACDEFGHIKLMNPQRSTVWY".to_vec(),
        };
        let result = validate_protein_sequence(&protein);
        assert!(result.valid);
        assert!(result.warnings.is_empty());
        assert!(result.errors.is_empty());
        assert_eq!(result.sequence, "ACDEFGHIKLMNPQRSTVWY");
        assert_eq!(result.alphabet, "protein-20");
    }

    #[test]
    fn validate_protein_sequence_warns_for_ambiguous_residues() {
        let protein = ProteinSequence {
            id: "ambig".into(),
            sequence: b"ACXBZJ".to_vec(),
        };
        let result = validate_protein_sequence(&protein);
        assert!(!result.valid);
        assert_eq!(result.warnings.len(), 4);
        assert!(result.errors.is_empty());

        let warning_residues: Vec<char> = result.warnings.iter().map(|w| w.residue).collect();
        assert_eq!(warning_residues, vec!['X', 'B', 'Z', 'J']);

        let positions: Vec<usize> = result.warnings.iter().map(|w| w.position).collect();
        assert_eq!(positions, vec![3, 4, 5, 6]);
    }

    #[test]
    fn validate_protein_sequence_errors_for_invalid_residues() {
        let protein = ProteinSequence {
            id: "bad".into(),
            sequence: b"AC*1D".to_vec(),
        };
        let result = validate_protein_sequence(&protein);
        assert!(!result.valid);
        assert!(result.warnings.is_empty());
        assert_eq!(result.errors.len(), 2);

        assert_eq!(result.errors[0].residue, '*');
        assert_eq!(result.errors[0].position, 3);
        assert_eq!(result.errors[1].residue, '1');
        assert_eq!(result.errors[1].position, 4);
    }

    #[test]
    fn validate_protein_sequence_mixed_warnings_and_errors() {
        let protein = ProteinSequence {
            id: "mixed".into(),
            sequence: b"AX*C".to_vec(),
        };
        let result = validate_protein_sequence(&protein);
        assert!(!result.valid);
        assert_eq!(result.warnings.len(), 1);
        assert_eq!(result.errors.len(), 1);
        assert_eq!(result.warnings[0].residue, 'X');
        assert_eq!(result.warnings[0].position, 2);
        assert_eq!(result.errors[0].residue, '*');
        assert_eq!(result.errors[0].position, 3);
    }

    #[test]
    fn validate_protein_sequence_empty_is_valid() {
        let protein = ProteinSequence {
            id: "empty".into(),
            sequence: b"".to_vec(),
        };
        let result = validate_protein_sequence(&protein);
        assert!(result.valid);
        assert!(result.warnings.is_empty());
        assert!(result.errors.is_empty());
        assert_eq!(result.sequence, "");
    }
}

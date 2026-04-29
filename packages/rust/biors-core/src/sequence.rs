use serde::{Deserialize, Serialize};

pub const PROTEIN_20: &str = "protein-20";
pub const PROTEIN_20_RESIDUES: [char; 20] = [
    'A', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'K', 'L', 'M', 'N', 'P', 'Q', 'R', 'S', 'T', 'V', 'W',
    'Y',
];
pub const AMBIGUOUS_RESIDUES: [char; 6] = ['X', 'B', 'Z', 'J', 'U', 'O'];

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProteinSequence {
    pub id: String,
    pub sequence: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResidueIssue {
    pub residue: char,
    pub position: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValidatedSequence {
    pub id: String,
    pub sequence: String,
    pub alphabet: String,
    pub valid: bool,
    pub warnings: Vec<ResidueIssue>,
    pub errors: Vec<ResidueIssue>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SequenceValidationReport {
    pub records: usize,
    pub valid_records: usize,
    pub warning_count: usize,
    pub error_count: usize,
    pub sequences: Vec<ValidatedSequence>,
}

pub fn normalize_sequence(sequence: &str) -> String {
    let mut normalized = String::with_capacity(sequence.len());
    append_normalized_sequence(sequence, &mut normalized);
    normalized
}

pub(crate) fn append_normalized_sequence(sequence: &str, output: &mut String) {
    output.reserve(sequence.len());
    if sequence.is_ascii() {
        for byte in sequence.bytes() {
            if !byte.is_ascii_whitespace() {
                output.push(byte.to_ascii_uppercase() as char);
            }
        }
        return;
    }

    output.extend(normalized_residues(sequence));
}

pub(crate) fn normalized_residues(sequence: &str) -> impl Iterator<Item = char> + '_ {
    sequence
        .chars()
        .filter(|residue| !residue.is_whitespace())
        .map(|residue| residue.to_ascii_uppercase())
}

pub(crate) fn is_protein_20_residue(residue: char) -> bool {
    if residue.is_ascii() {
        return PROTEIN_20_RESIDUE_LOOKUP[residue as usize];
    }

    matches!(
        residue,
        'A' | 'C'
            | 'D'
            | 'E'
            | 'F'
            | 'G'
            | 'H'
            | 'I'
            | 'K'
            | 'L'
            | 'M'
            | 'N'
            | 'P'
            | 'Q'
            | 'R'
            | 'S'
            | 'T'
            | 'V'
            | 'W'
            | 'Y'
    )
}

pub(crate) fn is_ambiguous_residue(residue: char) -> bool {
    if residue.is_ascii() {
        return AMBIGUOUS_RESIDUE_LOOKUP[residue as usize];
    }

    matches!(residue, 'X' | 'B' | 'Z' | 'J' | 'U' | 'O')
}

const PROTEIN_20_RESIDUE_LOOKUP: [bool; 256] = {
    let mut lookup = [false; 256];
    lookup[b'A' as usize] = true;
    lookup[b'C' as usize] = true;
    lookup[b'D' as usize] = true;
    lookup[b'E' as usize] = true;
    lookup[b'F' as usize] = true;
    lookup[b'G' as usize] = true;
    lookup[b'H' as usize] = true;
    lookup[b'I' as usize] = true;
    lookup[b'K' as usize] = true;
    lookup[b'L' as usize] = true;
    lookup[b'M' as usize] = true;
    lookup[b'N' as usize] = true;
    lookup[b'P' as usize] = true;
    lookup[b'Q' as usize] = true;
    lookup[b'R' as usize] = true;
    lookup[b'S' as usize] = true;
    lookup[b'T' as usize] = true;
    lookup[b'V' as usize] = true;
    lookup[b'W' as usize] = true;
    lookup[b'Y' as usize] = true;
    lookup
};

const AMBIGUOUS_RESIDUE_LOOKUP: [bool; 256] = {
    let mut lookup = [false; 256];
    lookup[b'X' as usize] = true;
    lookup[b'B' as usize] = true;
    lookup[b'Z' as usize] = true;
    lookup[b'J' as usize] = true;
    lookup[b'U' as usize] = true;
    lookup[b'O' as usize] = true;
    lookup
};

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

pub fn summarize_validated_sequences(
    sequences: Vec<ValidatedSequence>,
) -> SequenceValidationReport {
    SequenceValidationReport {
        records: sequences.len(),
        valid_records: sequences.iter().filter(|sequence| sequence.valid).count(),
        warning_count: sequences
            .iter()
            .map(|sequence| sequence.warnings.len())
            .sum(),
        error_count: sequences.iter().map(|sequence| sequence.errors.len()).sum(),
        sequences,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn append_normalized_sequence_extends_existing_buffer() {
        let mut output = String::from("AC");

        append_normalized_sequence(" d e\tf g ", &mut output);

        assert_eq!(output, "ACDEFG");
    }
}

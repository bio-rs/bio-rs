use serde::Serialize;
use std::fmt;

const PROTEIN_20: &str = "protein-20";
const PROTEIN_20_RESIDUES: [char; 20] = [
    'A', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'K', 'L', 'M', 'N', 'P', 'Q', 'R', 'S', 'T', 'V', 'W',
    'Y',
];
const AMBIGUOUS_RESIDUES: [char; 6] = ['X', 'B', 'Z', 'J', 'U', 'O'];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProteinSequence {
    pub id: String,
    pub sequence: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ResidueIssue {
    pub residue: char,
    pub position: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TokenizedProtein {
    pub id: String,
    pub length: usize,
    pub alphabet: &'static str,
    pub valid: bool,
    pub tokens: Vec<u8>,
    pub warnings: Vec<ResidueIssue>,
    pub errors: Vec<ResidueIssue>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BioRsError {
    EmptyInput,
    MissingHeader,
    MissingSequence { id: String },
    MultiFastaUnsupported,
}

impl fmt::Display for BioRsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyInput => write!(f, "FASTA input is empty"),
            Self::MissingHeader => write!(
                f,
                "FASTA input must start with a header line beginning with '>'"
            ),
            Self::MissingSequence { id } => {
                write!(f, "FASTA record '{id}' does not contain a sequence")
            }
            Self::MultiFastaUnsupported => write!(f, "multi-FASTA input is not supported yet"),
        }
    }
}

impl std::error::Error for BioRsError {}

pub fn parse_fasta(input: &str) -> Result<ProteinSequence, BioRsError> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err(BioRsError::EmptyInput);
    }

    let mut lines = trimmed
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty());
    let header = lines.next().ok_or(BioRsError::EmptyInput)?;
    if !header.starts_with('>') {
        return Err(BioRsError::MissingHeader);
    }

    let id = header[1..]
        .split_whitespace()
        .next()
        .unwrap_or_default()
        .to_string();
    let mut sequence = String::new();

    for line in lines {
        if line.starts_with('>') {
            return Err(BioRsError::MultiFastaUnsupported);
        }
        sequence.push_str(line);
    }

    let sequence = sequence.to_ascii_uppercase();
    if sequence.is_empty() {
        return Err(BioRsError::MissingSequence { id });
    }

    Ok(ProteinSequence { id, sequence })
}

pub fn tokenize_fasta(input: &str) -> Result<TokenizedProtein, BioRsError> {
    let protein = parse_fasta(input)?;
    Ok(tokenize_protein(&protein))
}

pub fn tokenize_protein(protein: &ProteinSequence) -> TokenizedProtein {
    let mut tokens = Vec::new();
    let mut warnings = Vec::new();
    let mut errors = Vec::new();

    for (index, residue) in protein.sequence.chars().enumerate() {
        let position = index + 1;
        if let Some(token) = protein_20_token(residue) {
            tokens.push(token);
        } else if AMBIGUOUS_RESIDUES.contains(&residue) {
            warnings.push(ResidueIssue { residue, position });
        } else {
            errors.push(ResidueIssue { residue, position });
        }
    }

    TokenizedProtein {
        id: protein.id.clone(),
        length: protein.sequence.chars().count(),
        alphabet: PROTEIN_20,
        valid: warnings.is_empty() && errors.is_empty(),
        tokens,
        warnings,
        errors,
    }
}

fn protein_20_token(residue: char) -> Option<u8> {
    PROTEIN_20_RESIDUES
        .iter()
        .position(|candidate| *candidate == residue)
        .map(|position| position as u8)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokenizes_valid_protein_20_sequence() {
        let tokenized = tokenize_fasta(">seq1\nACDE").expect("valid FASTA");

        assert_eq!(tokenized.id, "seq1");
        assert_eq!(tokenized.length, 4);
        assert!(tokenized.valid);
        assert_eq!(tokenized.tokens, vec![0, 1, 2, 3]);
        assert!(tokenized.warnings.is_empty());
        assert!(tokenized.errors.is_empty());
    }

    #[test]
    fn normalizes_lowercase_sequence() {
        let tokenized = tokenize_fasta(">seq1\nacde").expect("valid FASTA");

        assert_eq!(tokenized.tokens, vec![0, 1, 2, 3]);
    }

    #[test]
    fn preserves_fasta_id() {
        let protein = parse_fasta(">sp|P12345|KINASE example protein\nACDE").expect("valid FASTA");

        assert_eq!(protein.id, "sp|P12345|KINASE");
    }

    #[test]
    fn reports_ambiguous_residues() {
        let tokenized = tokenize_fasta(">seq1\nAXZ").expect("valid FASTA");

        assert!(!tokenized.valid);
        assert_eq!(tokenized.tokens, vec![0]);
        assert_eq!(
            tokenized.warnings,
            vec![
                ResidueIssue {
                    residue: 'X',
                    position: 2,
                },
                ResidueIssue {
                    residue: 'Z',
                    position: 3,
                },
            ]
        );
        assert!(tokenized.errors.is_empty());
    }

    #[test]
    fn reports_invalid_characters() {
        let tokenized = tokenize_fasta(">seq1\nA*D").expect("valid FASTA");

        assert!(!tokenized.valid);
        assert_eq!(
            tokenized.errors,
            vec![ResidueIssue {
                residue: '*',
                position: 2,
            }]
        );
    }

    #[test]
    fn rejects_empty_sequence() {
        let error = tokenize_fasta(">seq1").expect_err("empty sequence should fail");

        assert_eq!(
            error,
            BioRsError::MissingSequence {
                id: "seq1".to_string()
            }
        );
    }

    #[test]
    fn rejects_multi_fasta() {
        let error = tokenize_fasta(">seq1\nAC\n>seq2\nDE").expect_err("multi-FASTA should fail");

        assert_eq!(error, BioRsError::MultiFastaUnsupported);
    }
}

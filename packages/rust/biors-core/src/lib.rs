use serde::{Deserialize, Serialize};
use std::fmt;

pub mod package;

pub use package::{
    inspect_package_manifest, validate_package_manifest, ModelArtifact, PackageFixture,
    PackageManifest, PackageManifestSummary, PackageValidationReport, PipelineStep, RuntimeTarget,
};

const PROTEIN_20: &str = "protein-20";
const PROTEIN_20_RESIDUES: [char; 20] = [
    'A', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'K', 'L', 'M', 'N', 'P', 'Q', 'R', 'S', 'T', 'V', 'W',
    'Y',
];
const AMBIGUOUS_RESIDUES: [char; 6] = ['X', 'B', 'Z', 'J', 'U', 'O'];

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
pub struct TokenizedProtein {
    pub id: String,
    pub length: usize,
    pub alphabet: String,
    pub valid: bool,
    pub tokens: Vec<u8>,
    pub warnings: Vec<ResidueIssue>,
    pub errors: Vec<ResidueIssue>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProteinBatchSummary {
    pub records: usize,
    pub total_length: usize,
    pub valid_records: usize,
    pub warning_count: usize,
    pub error_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BioRsError {
    EmptyInput,
    MissingHeader,
    MissingSequence { id: String },
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
        }
    }
}

impl std::error::Error for BioRsError {}

pub fn parse_fasta_records(input: &str) -> Result<Vec<ProteinSequence>, BioRsError> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err(BioRsError::EmptyInput);
    }

    let lines = trimmed
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty());

    let mut records = Vec::new();
    let mut current_id = None;
    let mut sequence = String::new();

    for line in lines {
        if let Some(header) = line.strip_prefix('>') {
            if let Some(id) = current_id.replace(fasta_id(header)) {
                push_fasta_record(&mut records, id, &mut sequence)?;
            }
        } else {
            if current_id.is_none() {
                return Err(BioRsError::MissingHeader);
            }
            sequence.push_str(line);
        }
    }

    let id = current_id.ok_or(BioRsError::MissingHeader)?;
    push_fasta_record(&mut records, id, &mut sequence)?;

    Ok(records)
}

pub fn tokenize_fasta_records(input: &str) -> Result<Vec<TokenizedProtein>, BioRsError> {
    let proteins = parse_fasta_records(input)?;
    Ok(proteins.iter().map(tokenize_protein).collect())
}

pub fn summarize_tokenized_proteins(proteins: &[TokenizedProtein]) -> ProteinBatchSummary {
    ProteinBatchSummary {
        records: proteins.len(),
        total_length: proteins.iter().map(|protein| protein.length).sum(),
        valid_records: proteins.iter().filter(|protein| protein.valid).count(),
        warning_count: proteins.iter().map(|protein| protein.warnings.len()).sum(),
        error_count: proteins.iter().map(|protein| protein.errors.len()).sum(),
    }
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
        alphabet: PROTEIN_20.to_string(),
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

fn fasta_id(header: &str) -> String {
    header
        .split_whitespace()
        .next()
        .unwrap_or_default()
        .to_string()
}

fn push_fasta_record(
    records: &mut Vec<ProteinSequence>,
    id: String,
    sequence: &mut String,
) -> Result<(), BioRsError> {
    if sequence.is_empty() {
        return Err(BioRsError::MissingSequence { id });
    }

    records.push(ProteinSequence {
        id,
        sequence: std::mem::take(sequence).to_ascii_uppercase(),
    });
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokenizes_valid_protein_20_sequence() {
        let tokenized = tokenize_fasta_records(">seq1\nACDE").expect("valid FASTA");

        assert_eq!(tokenized.len(), 1);
        assert_eq!(tokenized[0].id, "seq1");
        assert_eq!(tokenized[0].length, 4);
        assert!(tokenized[0].valid);
        assert_eq!(tokenized[0].tokens, vec![0, 1, 2, 3]);
        assert!(tokenized[0].warnings.is_empty());
        assert!(tokenized[0].errors.is_empty());
    }

    #[test]
    fn normalizes_lowercase_sequence() {
        let tokenized = tokenize_fasta_records(">seq1\nacde").expect("valid FASTA");

        assert_eq!(tokenized[0].tokens, vec![0, 1, 2, 3]);
    }

    #[test]
    fn preserves_fasta_id() {
        let proteins =
            parse_fasta_records(">sp|P12345|KINASE example protein\nACDE").expect("valid FASTA");

        assert_eq!(proteins[0].id, "sp|P12345|KINASE");
    }

    #[test]
    fn parses_multi_fasta_records_in_order() {
        let records = parse_fasta_records(">seq1 first\nACDE\n>seq2 second\nmnpq")
            .expect("valid multi-FASTA");

        assert_eq!(
            records,
            vec![
                ProteinSequence {
                    id: "seq1".to_string(),
                    sequence: "ACDE".to_string(),
                },
                ProteinSequence {
                    id: "seq2".to_string(),
                    sequence: "MNPQ".to_string(),
                },
            ]
        );
    }

    #[test]
    fn rejects_empty_record_in_multi_fasta() {
        let error =
            parse_fasta_records(">seq1\nACDE\n>seq2").expect_err("empty record should fail");

        assert_eq!(
            error,
            BioRsError::MissingSequence {
                id: "seq2".to_string()
            }
        );
    }

    #[test]
    fn reports_ambiguous_residues() {
        let tokenized = tokenize_fasta_records(">seq1\nAXZ").expect("valid FASTA");

        assert!(!tokenized[0].valid);
        assert_eq!(tokenized[0].tokens, vec![0]);
        assert_eq!(
            tokenized[0].warnings,
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
        assert!(tokenized[0].errors.is_empty());
    }

    #[test]
    fn reports_invalid_characters() {
        let tokenized = tokenize_fasta_records(">seq1\nA*D").expect("valid FASTA");

        assert!(!tokenized[0].valid);
        assert_eq!(
            tokenized[0].errors,
            vec![ResidueIssue {
                residue: '*',
                position: 2,
            }]
        );
    }

    #[test]
    fn tokenizes_multi_fasta_records_independently() {
        let tokenized =
            tokenize_fasta_records(">seq1\nacx\n>seq2\nBJ*").expect("valid multi-FASTA");

        assert_eq!(tokenized.len(), 2);
        assert_eq!(tokenized[0].id, "seq1");
        assert_eq!(tokenized[0].tokens, vec![0, 1]);
        assert_eq!(
            tokenized[0].warnings,
            vec![ResidueIssue {
                residue: 'X',
                position: 3,
            }]
        );
        assert!(tokenized[0].errors.is_empty());

        assert_eq!(tokenized[1].id, "seq2");
        assert!(tokenized[1].tokens.is_empty());
        assert_eq!(
            tokenized[1].warnings,
            vec![
                ResidueIssue {
                    residue: 'B',
                    position: 1,
                },
                ResidueIssue {
                    residue: 'J',
                    position: 2,
                },
            ]
        );
        assert_eq!(
            tokenized[1].errors,
            vec![ResidueIssue {
                residue: '*',
                position: 3,
            }]
        );
    }

    #[test]
    fn summarizes_tokenized_protein_batches() {
        let tokenized = tokenize_fasta_records(">seq1\nACX\n>seq2\nM*").expect("valid multi-FASTA");
        let summary = summarize_tokenized_proteins(&tokenized);

        assert_eq!(
            summary,
            ProteinBatchSummary {
                records: 2,
                total_length: 5,
                valid_records: 0,
                warning_count: 1,
                error_count: 1,
            }
        );
    }

    #[test]
    fn tokenized_protein_round_trips_through_json() {
        let tokenized = tokenize_fasta_records(">seq1\nACX").expect("valid FASTA");
        let json = serde_json::to_string(&tokenized).expect("serialize tokenized protein");
        let decoded: Vec<TokenizedProtein> =
            serde_json::from_str(&json).expect("deserialize tokenized protein");

        assert_eq!(decoded, tokenized);
    }

    #[test]
    fn rejects_empty_sequence() {
        let error = tokenize_fasta_records(">seq1").expect_err("empty sequence should fail");

        assert_eq!(
            error,
            BioRsError::MissingSequence {
                id: "seq1".to_string()
            }
        );
    }
}

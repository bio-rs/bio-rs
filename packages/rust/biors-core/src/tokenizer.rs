use crate::fasta::parse_fasta_records;
use crate::sequence::{
    ProteinSequence, ResidueIssue, AMBIGUOUS_RESIDUES, PROTEIN_20, PROTEIN_20_RESIDUES,
};
use crate::BioRsError;
use serde::{Deserialize, Serialize};

pub trait Tokenizer {
    fn alphabet(&self) -> &'static str;
    fn vocabulary(&self) -> Vocabulary;
    fn tokenize(&self, protein: &ProteinSequence) -> TokenizedProtein;
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct ProteinTokenizer;

impl Tokenizer for ProteinTokenizer {
    fn alphabet(&self) -> &'static str {
        PROTEIN_20
    }

    fn vocabulary(&self) -> Vocabulary {
        load_protein_20_vocab()
    }

    fn tokenize(&self, protein: &ProteinSequence) -> TokenizedProtein {
        tokenize_protein(protein)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Vocabulary {
    pub name: String,
    pub tokens: Vec<VocabToken>,
    pub unknown_token_policy: UnknownTokenPolicy,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VocabToken {
    pub residue: char,
    pub token_id: u8,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UnknownTokenPolicy {
    WarnOrErrorWithoutToken,
}

pub fn load_protein_20_vocab() -> Vocabulary {
    Vocabulary {
        name: PROTEIN_20.to_string(),
        tokens: PROTEIN_20_RESIDUES
            .iter()
            .enumerate()
            .map(|(token_id, residue)| VocabToken {
                residue: *residue,
                token_id: token_id as u8,
            })
            .collect(),
        unknown_token_policy: protein_20_unknown_token_policy(),
    }
}

pub const fn protein_20_unknown_token_policy() -> UnknownTokenPolicy {
    UnknownTokenPolicy::WarnOrErrorWithoutToken
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

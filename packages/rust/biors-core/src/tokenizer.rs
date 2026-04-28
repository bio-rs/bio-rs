use crate::sequence::{
    normalize_sequence, ProteinSequence, ResidueIssue, ValidatedSequence, AMBIGUOUS_RESIDUES,
    PROTEIN_20, PROTEIN_20_RESIDUES,
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
    pub unknown_token_id: u8,
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
    WarnOrErrorWithUnknownToken,
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
        unknown_token_id: PROTEIN_20_UNKNOWN_TOKEN_ID,
        unknown_token_policy: protein_20_unknown_token_policy(),
    }
}

pub fn load_vocab_json(input: &str) -> Result<Vocabulary, serde_json::Error> {
    serde_json::from_str(input)
}

pub const fn protein_20_unknown_token_policy() -> UnknownTokenPolicy {
    UnknownTokenPolicy::WarnOrErrorWithUnknownToken
}

pub const PROTEIN_20_UNKNOWN_TOKEN_ID: u8 = 20;

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
    let analyzed = analyze_fasta_records(input)?;
    Ok(analyzed
        .into_iter()
        .map(|record| record.tokenized)
        .collect())
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
            tokens.push(PROTEIN_20_UNKNOWN_TOKEN_ID);
            warnings.push(ResidueIssue { residue, position });
        } else {
            tokens.push(PROTEIN_20_UNKNOWN_TOKEN_ID);
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct AnalyzedProtein {
    pub protein: ProteinSequence,
    pub tokenized: TokenizedProtein,
}

pub(crate) fn analyze_fasta_records(input: &str) -> Result<Vec<AnalyzedProtein>, BioRsError> {
    if input.trim().is_empty() {
        return Err(BioRsError::EmptyInput);
    }

    let mut records = Vec::new();
    let mut current_id: Option<String> = None;
    let mut current_sequence = String::new();
    let mut current_tokens = Vec::new();
    let mut current_warnings = Vec::new();
    let mut current_errors = Vec::new();
    let mut current_header_line = 0;
    let mut current_record_index = 0;
    let mut current_length = 0usize;

    for (line_index, raw_line) in input.lines().enumerate() {
        let line_number = line_index + 1;
        let line = raw_line.trim();

        if line.is_empty() {
            continue;
        }

        if let Some(header) = line.strip_prefix('>') {
            let next_id = fasta_id(header).ok_or(BioRsError::MissingIdentifier {
                line: line_number,
                record_index: current_record_index,
            })?;
            if let Some(id) = current_id.replace(next_id) {
                push_analyzed_record(
                    &mut records,
                    id,
                    &mut current_sequence,
                    &mut current_tokens,
                    &mut current_warnings,
                    &mut current_errors,
                    &mut current_length,
                    current_header_line,
                    current_record_index,
                )?;
                current_record_index += 1;
            }
            current_header_line = line_number;
        } else {
            if current_id.is_none() {
                return Err(BioRsError::MissingHeader { line: line_number });
            }

            for residue in normalize_sequence(line).chars() {
                current_length += 1;
                current_sequence.push(residue);
                let position = current_length;
                if let Some(token) = protein_20_token(residue) {
                    current_tokens.push(token);
                } else if AMBIGUOUS_RESIDUES.contains(&residue) {
                    current_tokens.push(PROTEIN_20_UNKNOWN_TOKEN_ID);
                    current_warnings.push(ResidueIssue { residue, position });
                } else {
                    current_tokens.push(PROTEIN_20_UNKNOWN_TOKEN_ID);
                    current_errors.push(ResidueIssue { residue, position });
                }
            }
        }
    }

    let id = current_id.ok_or(BioRsError::MissingHeader { line: 1 })?;
    push_analyzed_record(
        &mut records,
        id,
        &mut current_sequence,
        &mut current_tokens,
        &mut current_warnings,
        &mut current_errors,
        &mut current_length,
        current_header_line,
        current_record_index,
    )?;

    Ok(records)
}

pub(crate) fn validated_sequences_from_analyzed(
    records: &[AnalyzedProtein],
) -> Vec<ValidatedSequence> {
    records
        .iter()
        .map(|record| ValidatedSequence {
            id: record.protein.id.clone(),
            sequence: record.protein.sequence.clone(),
            alphabet: record.tokenized.alphabet.clone(),
            valid: record.tokenized.valid,
            warnings: record.tokenized.warnings.clone(),
            errors: record.tokenized.errors.clone(),
        })
        .collect()
}

fn fasta_id(header: &str) -> Option<String> {
    header.split_whitespace().next().map(str::to_string)
}

#[allow(clippy::too_many_arguments)]
fn push_analyzed_record(
    records: &mut Vec<AnalyzedProtein>,
    id: String,
    sequence: &mut String,
    tokens: &mut Vec<u8>,
    warnings: &mut Vec<ResidueIssue>,
    errors: &mut Vec<ResidueIssue>,
    length: &mut usize,
    line: usize,
    record_index: usize,
) -> Result<(), BioRsError> {
    if sequence.is_empty() {
        return Err(BioRsError::MissingSequence {
            id,
            line,
            record_index,
        });
    }

    let protein = ProteinSequence {
        id: id.clone(),
        sequence: std::mem::take(sequence),
    };
    let tokenized = TokenizedProtein {
        id,
        length: std::mem::take(length),
        alphabet: PROTEIN_20.to_string(),
        valid: warnings.is_empty() && errors.is_empty(),
        tokens: std::mem::take(tokens),
        warnings: std::mem::take(warnings),
        errors: std::mem::take(errors),
    };
    records.push(AnalyzedProtein { protein, tokenized });
    Ok(())
}

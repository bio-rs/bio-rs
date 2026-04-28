use crate::sequence::{
    is_ambiguous_residue, normalized_residues, ProteinSequence, ResidueIssue, ValidatedSequence,
    PROTEIN_20, PROTEIN_20_RESIDUES,
};
use crate::verification::StableInputHasher;
use crate::BioRsError;
use serde::{Deserialize, Serialize};
use std::io::BufRead;

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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TokenizedFastaInput {
    pub input_hash: String,
    pub records: Vec<TokenizedProtein>,
}

pub fn tokenize_fasta_records(input: &str) -> Result<Vec<TokenizedProtein>, BioRsError> {
    let analyzed = analyze_fasta_records(input)?;
    Ok(analyzed
        .into_iter()
        .map(|record| record.tokenized)
        .collect())
}

pub fn tokenize_fasta_records_reader<R: BufRead>(
    reader: R,
) -> Result<TokenizedFastaInput, crate::FastaReadError> {
    let analyzed = analyze_fasta_records_reader(reader)?;
    Ok(TokenizedFastaInput {
        input_hash: analyzed.input_hash,
        records: analyzed
            .records
            .into_iter()
            .map(|record| record.tokenized)
            .collect(),
    })
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
    let mut tokens = Vec::with_capacity(protein.sequence.len());
    let mut warnings = Vec::new();
    let mut errors = Vec::new();

    for (index, residue) in protein.sequence.chars().enumerate() {
        push_tokenized_residue(residue, index + 1, &mut tokens, &mut warnings, &mut errors);
    }

    TokenizedProtein {
        id: protein.id.clone(),
        length: tokens.len(),
        alphabet: PROTEIN_20.to_string(),
        valid: warnings.is_empty() && errors.is_empty(),
        tokens,
        warnings,
        errors,
    }
}

fn push_tokenized_residue(
    residue: char,
    position: usize,
    tokens: &mut Vec<u8>,
    warnings: &mut Vec<ResidueIssue>,
    errors: &mut Vec<ResidueIssue>,
) {
    if let Some(token) = protein_20_token_id(residue) {
        tokens.push(token);
    } else if is_ambiguous_residue(residue) {
        tokens.push(PROTEIN_20_UNKNOWN_TOKEN_ID);
        warnings.push(ResidueIssue { residue, position });
    } else {
        tokens.push(PROTEIN_20_UNKNOWN_TOKEN_ID);
        errors.push(ResidueIssue { residue, position });
    }
}

fn protein_20_token_id(residue: char) -> Option<u8> {
    match residue {
        'A' => Some(0),
        'C' => Some(1),
        'D' => Some(2),
        'E' => Some(3),
        'F' => Some(4),
        'G' => Some(5),
        'H' => Some(6),
        'I' => Some(7),
        'K' => Some(8),
        'L' => Some(9),
        'M' => Some(10),
        'N' => Some(11),
        'P' => Some(12),
        'Q' => Some(13),
        'R' => Some(14),
        'S' => Some(15),
        'T' => Some(16),
        'V' => Some(17),
        'W' => Some(18),
        'Y' => Some(19),
        _ => None,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct AnalyzedProtein {
    pub protein: ProteinSequence,
    pub tokenized: TokenizedProtein,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct AnalyzedFastaInput {
    pub input_hash: String,
    pub records: Vec<AnalyzedProtein>,
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

            for residue in normalized_residues(line) {
                current_length += 1;
                current_sequence.push(residue);
                push_tokenized_residue(
                    residue,
                    current_length,
                    &mut current_tokens,
                    &mut current_warnings,
                    &mut current_errors,
                );
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

pub(crate) fn analyze_fasta_records_reader<R: BufRead>(
    mut reader: R,
) -> Result<AnalyzedFastaInput, crate::FastaReadError> {
    let mut records = Vec::new();
    let mut current_id: Option<String> = None;
    let mut current_sequence = String::new();
    let mut current_tokens = Vec::new();
    let mut current_warnings = Vec::new();
    let mut current_errors = Vec::new();
    let mut current_header_line = 0;
    let mut current_record_index = 0;
    let mut current_length = 0usize;
    let mut line_number = 0usize;
    let mut hasher = StableInputHasher::new();
    let mut raw_line = String::new();

    loop {
        raw_line.clear();
        let bytes = reader.read_line(&mut raw_line)?;
        if bytes == 0 {
            break;
        }
        line_number += 1;
        hasher.update(raw_line.as_bytes());
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
                return Err(BioRsError::MissingHeader { line: line_number }.into());
            }

            for residue in normalized_residues(line) {
                current_length += 1;
                current_sequence.push(residue);
                push_tokenized_residue(
                    residue,
                    current_length,
                    &mut current_tokens,
                    &mut current_warnings,
                    &mut current_errors,
                );
            }
        }
    }

    if line_number == 0 || records.is_empty() && current_id.is_none() {
        return Err(BioRsError::EmptyInput.into());
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

    Ok(AnalyzedFastaInput {
        input_hash: hasher.finalize(),
        records,
    })
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn protein_20_token_id_matches_vocab_order() {
        for (expected, residue) in PROTEIN_20_RESIDUES.iter().enumerate() {
            assert_eq!(protein_20_token_id(*residue), Some(expected as u8));
        }

        assert_eq!(protein_20_token_id('X'), None);
        assert_eq!(protein_20_token_id('*'), None);
    }

    #[test]
    fn ambiguous_residue_lookup_matches_policy_residues() {
        for residue in crate::sequence::AMBIGUOUS_RESIDUES {
            assert!(is_ambiguous_residue(residue));
        }

        assert!(!is_ambiguous_residue('A'));
        assert!(!is_ambiguous_residue('*'));
    }
}

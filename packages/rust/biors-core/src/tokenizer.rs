use crate::fasta_scan::{scan_fasta_reader, scan_fasta_str, FastaRecordSink};
use crate::sequence::{
    is_ambiguous_residue, normalized_residues, ProteinSequence, ResidueIssue, PROTEIN_20,
};
use crate::BioRsError;
use serde::{Deserialize, Serialize};
use std::io::BufRead;

mod vocab;
use vocab::TOKEN_LOOKUP_MISSING;
pub use vocab::{
    load_protein_20_vocab, load_vocab_json, protein_20_unknown_token_policy,
    protein_20_vocab_tokens, protein_20_vocabulary, UnknownTokenPolicy, VocabToken, Vocabulary,
    PROTEIN_20_UNKNOWN_TOKEN_ID,
};

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

impl ProteinTokenizer {
    pub fn vocabulary_ref(&self) -> &'static Vocabulary {
        protein_20_vocabulary()
    }
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

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SummarizedFastaInput {
    pub input_hash: String,
    pub summary: ProteinBatchSummary,
}

pub fn tokenize_fasta_records(input: &str) -> Result<Vec<TokenizedProtein>, BioRsError> {
    let mut sink = TokenizedRecordSink::default();
    scan_fasta_str(input, &mut sink)?;
    Ok(sink.records)
}

pub fn tokenize_fasta_records_reader<R: BufRead>(
    reader: R,
) -> Result<TokenizedFastaInput, crate::FastaReadError> {
    let mut sink = TokenizedRecordSink::default();
    let input_hash = scan_fasta_reader(reader, &mut sink)?;
    Ok(TokenizedFastaInput {
        input_hash,
        records: sink.records,
    })
}

pub fn summarize_fasta_records_reader<R: BufRead>(
    reader: R,
) -> Result<SummarizedFastaInput, crate::FastaReadError> {
    let mut sink = SummaryRecordSink::default();
    let input_hash = scan_fasta_reader(reader, &mut sink)?;

    Ok(SummarizedFastaInput {
        input_hash,
        summary: sink.summary,
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

    if protein.sequence.is_ascii() {
        for (index, byte) in protein.sequence.bytes().enumerate() {
            push_tokenized_residue_byte(byte, index + 1, &mut tokens, &mut warnings, &mut errors);
        }
    } else {
        for (index, residue) in protein.sequence.chars().enumerate() {
            push_tokenized_residue(residue, index + 1, &mut tokens, &mut warnings, &mut errors);
        }
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

fn push_tokenized_residue_byte(
    residue: u8,
    position: usize,
    tokens: &mut Vec<u8>,
    warnings: &mut Vec<ResidueIssue>,
    errors: &mut Vec<ResidueIssue>,
) {
    if let Some(token) = protein_20_token_id_byte(residue) {
        tokens.push(token);
    } else if is_ambiguous_residue_byte(residue) {
        tokens.push(PROTEIN_20_UNKNOWN_TOKEN_ID);
        warnings.push(ResidueIssue {
            residue: residue.to_ascii_uppercase() as char,
            position,
        });
    } else {
        tokens.push(PROTEIN_20_UNKNOWN_TOKEN_ID);
        errors.push(ResidueIssue {
            residue: residue.to_ascii_uppercase() as char,
            position,
        });
    }
}

fn protein_20_token_id(residue: char) -> Option<u8> {
    if residue.is_ascii() {
        return protein_20_token_id_byte(residue as u8);
    }

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

fn protein_20_token_id_byte(residue: u8) -> Option<u8> {
    let token = PROTEIN_20_TOKEN_LOOKUP[residue as usize];
    if token == TOKEN_LOOKUP_MISSING {
        None
    } else {
        Some(token)
    }
}

fn is_ambiguous_residue_byte(residue: u8) -> bool {
    AMBIGUOUS_RESIDUE_LOOKUP[residue as usize]
}

const PROTEIN_20_TOKEN_LOOKUP: [u8; 256] = {
    let mut lookup = [TOKEN_LOOKUP_MISSING; 256];
    lookup[b'A' as usize] = 0;
    lookup[b'C' as usize] = 1;
    lookup[b'D' as usize] = 2;
    lookup[b'E' as usize] = 3;
    lookup[b'F' as usize] = 4;
    lookup[b'G' as usize] = 5;
    lookup[b'H' as usize] = 6;
    lookup[b'I' as usize] = 7;
    lookup[b'K' as usize] = 8;
    lookup[b'L' as usize] = 9;
    lookup[b'M' as usize] = 10;
    lookup[b'N' as usize] = 11;
    lookup[b'P' as usize] = 12;
    lookup[b'Q' as usize] = 13;
    lookup[b'R' as usize] = 14;
    lookup[b'S' as usize] = 15;
    lookup[b'T' as usize] = 16;
    lookup[b'V' as usize] = 17;
    lookup[b'W' as usize] = 18;
    lookup[b'Y' as usize] = 19;
    lookup[b'a' as usize] = 0;
    lookup[b'c' as usize] = 1;
    lookup[b'd' as usize] = 2;
    lookup[b'e' as usize] = 3;
    lookup[b'f' as usize] = 4;
    lookup[b'g' as usize] = 5;
    lookup[b'h' as usize] = 6;
    lookup[b'i' as usize] = 7;
    lookup[b'k' as usize] = 8;
    lookup[b'l' as usize] = 9;
    lookup[b'm' as usize] = 10;
    lookup[b'n' as usize] = 11;
    lookup[b'p' as usize] = 12;
    lookup[b'q' as usize] = 13;
    lookup[b'r' as usize] = 14;
    lookup[b's' as usize] = 15;
    lookup[b't' as usize] = 16;
    lookup[b'v' as usize] = 17;
    lookup[b'w' as usize] = 18;
    lookup[b'y' as usize] = 19;
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
    lookup[b'x' as usize] = true;
    lookup[b'b' as usize] = true;
    lookup[b'z' as usize] = true;
    lookup[b'j' as usize] = true;
    lookup[b'u' as usize] = true;
    lookup[b'o' as usize] = true;
    lookup
};

#[derive(Default)]
struct TokenizedRecordSink {
    records: Vec<TokenizedProtein>,
    current_tokens: Vec<u8>,
    current_warnings: Vec<ResidueIssue>,
    current_errors: Vec<ResidueIssue>,
    current_length: usize,
}

impl FastaRecordSink for TokenizedRecordSink {
    fn push_sequence_line(&mut self, line: &str) {
        if line.is_ascii() {
            self.push_sequence_line_bytes(line.as_bytes());
            return;
        }

        for residue in normalized_residues(line) {
            self.push_residue(residue);
        }
    }

    fn push_sequence_line_bytes(&mut self, line: &[u8]) {
        self.current_tokens.reserve(line.len());
        for &byte in line {
            if byte.is_ascii_whitespace() {
                continue;
            }
            self.push_residue_byte(byte);
        }
    }

    fn finish_record(
        &mut self,
        id: String,
        line: usize,
        record_index: usize,
    ) -> Result<(), BioRsError> {
        if self.current_length == 0 {
            return Err(BioRsError::MissingSequence {
                id,
                line,
                record_index,
            });
        }

        self.records.push(TokenizedProtein {
            id,
            length: std::mem::take(&mut self.current_length),
            alphabet: PROTEIN_20.to_string(),
            valid: self.current_warnings.is_empty() && self.current_errors.is_empty(),
            tokens: std::mem::take(&mut self.current_tokens),
            warnings: std::mem::take(&mut self.current_warnings),
            errors: std::mem::take(&mut self.current_errors),
        });
        Ok(())
    }
}

#[derive(Default)]
struct SummaryRecordSink {
    summary: ProteinBatchSummary,
    current_length: usize,
    current_warning_count: usize,
    current_error_count: usize,
}

impl FastaRecordSink for SummaryRecordSink {
    fn push_sequence_line(&mut self, line: &str) {
        if line.is_ascii() {
            self.push_sequence_line_bytes(line.as_bytes());
            return;
        }

        for residue in normalized_residues(line) {
            self.push_residue(residue);
        }
    }

    fn push_sequence_line_bytes(&mut self, line: &[u8]) {
        for &byte in line {
            if byte.is_ascii_whitespace() {
                continue;
            }
            self.push_residue_byte(byte);
        }
    }

    fn finish_record(
        &mut self,
        id: String,
        line: usize,
        record_index: usize,
    ) -> Result<(), BioRsError> {
        if self.current_length == 0 {
            return Err(BioRsError::MissingSequence {
                id,
                line,
                record_index,
            });
        }

        self.summary.records += 1;
        self.summary.total_length += self.current_length;
        self.summary.warning_count += self.current_warning_count;
        self.summary.error_count += self.current_error_count;
        if self.current_warning_count == 0 && self.current_error_count == 0 {
            self.summary.valid_records += 1;
        }

        self.current_length = 0;
        self.current_warning_count = 0;
        self.current_error_count = 0;
        Ok(())
    }
}

impl SummaryRecordSink {
    fn push_residue(&mut self, residue: char) {
        self.current_length += 1;
        if protein_20_token_id(residue).is_some() {
            return;
        }

        if is_ambiguous_residue(residue) {
            self.current_warning_count += 1;
        } else {
            self.current_error_count += 1;
        }
    }

    fn push_residue_byte(&mut self, residue: u8) {
        self.current_length += 1;
        if protein_20_token_id_byte(residue).is_some() {
            return;
        }

        if is_ambiguous_residue_byte(residue) {
            self.current_warning_count += 1;
        } else {
            self.current_error_count += 1;
        }
    }
}

impl TokenizedRecordSink {
    fn push_residue(&mut self, residue: char) {
        self.current_length += 1;
        push_tokenized_residue(
            residue,
            self.current_length,
            &mut self.current_tokens,
            &mut self.current_warnings,
            &mut self.current_errors,
        );
    }

    fn push_residue_byte(&mut self, residue: u8) {
        let residue = residue.to_ascii_uppercase();
        self.current_length += 1;
        push_tokenized_residue_byte(
            residue,
            self.current_length,
            &mut self.current_tokens,
            &mut self.current_warnings,
            &mut self.current_errors,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn protein_20_token_id_matches_vocab_order() {
        for (expected, residue) in crate::sequence::PROTEIN_20_RESIDUES.iter().enumerate() {
            assert_eq!(protein_20_token_id(*residue), Some(expected as u8));
            assert_eq!(
                protein_20_token_id_byte(*residue as u8),
                Some(expected as u8)
            );
            assert_eq!(
                protein_20_token_id_byte((*residue as u8).to_ascii_lowercase()),
                Some(expected as u8)
            );
        }

        assert_eq!(protein_20_token_id('X'), None);
        assert_eq!(protein_20_token_id_byte(b'X'), None);
        assert_eq!(protein_20_token_id('*'), None);
        assert_eq!(protein_20_token_id_byte(b'*'), None);
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

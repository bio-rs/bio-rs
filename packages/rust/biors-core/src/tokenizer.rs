use crate::fasta_scan::{scan_fasta_reader, scan_fasta_str, FastaRecordSink};
use crate::sequence::{
    is_ambiguous_residue, normalized_residues, ProteinSequence, ResidueIssue, PROTEIN_20,
};
use crate::BioRsError;
use serde::{Deserialize, Serialize};
use std::io::BufRead;

mod lookup;
mod vocab;
use lookup::{
    is_ambiguous_residue_byte, protein_20_token_id, protein_20_token_id_byte,
    push_tokenized_residue, push_tokenized_residue_byte,
};
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

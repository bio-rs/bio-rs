use crate::fasta_scan::{scan_fasta_reader, scan_fasta_str, FastaRecordSink};
use crate::sequence::{
    is_ambiguous_residue, normalized_residues, ProteinSequence, ResidueIssue, ValidatedSequence,
    PROTEIN_20,
};
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
        tokens: protein_20_vocab_tokens().to_vec(),
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
const TOKEN_LOOKUP_MISSING: u8 = u8::MAX;

pub fn protein_20_vocab_tokens() -> &'static [VocabToken; 20] {
    &PROTEIN_20_VOCAB_TOKENS
}

const PROTEIN_20_VOCAB_TOKENS: [VocabToken; 20] = [
    VocabToken {
        residue: 'A',
        token_id: 0,
    },
    VocabToken {
        residue: 'C',
        token_id: 1,
    },
    VocabToken {
        residue: 'D',
        token_id: 2,
    },
    VocabToken {
        residue: 'E',
        token_id: 3,
    },
    VocabToken {
        residue: 'F',
        token_id: 4,
    },
    VocabToken {
        residue: 'G',
        token_id: 5,
    },
    VocabToken {
        residue: 'H',
        token_id: 6,
    },
    VocabToken {
        residue: 'I',
        token_id: 7,
    },
    VocabToken {
        residue: 'K',
        token_id: 8,
    },
    VocabToken {
        residue: 'L',
        token_id: 9,
    },
    VocabToken {
        residue: 'M',
        token_id: 10,
    },
    VocabToken {
        residue: 'N',
        token_id: 11,
    },
    VocabToken {
        residue: 'P',
        token_id: 12,
    },
    VocabToken {
        residue: 'Q',
        token_id: 13,
    },
    VocabToken {
        residue: 'R',
        token_id: 14,
    },
    VocabToken {
        residue: 'S',
        token_id: 15,
    },
    VocabToken {
        residue: 'T',
        token_id: 16,
    },
    VocabToken {
        residue: 'V',
        token_id: 17,
    },
    VocabToken {
        residue: 'W',
        token_id: 18,
    },
    VocabToken {
        residue: 'Y',
        token_id: 19,
    },
];

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
    let residue = residue.to_ascii_uppercase();
    if let Some(token) = protein_20_token_id_byte(residue) {
        tokens.push(token);
    } else if is_ambiguous_residue_byte(residue) {
        tokens.push(PROTEIN_20_UNKNOWN_TOKEN_ID);
        warnings.push(ResidueIssue {
            residue: residue as char,
            position,
        });
    } else {
        tokens.push(PROTEIN_20_UNKNOWN_TOKEN_ID);
        errors.push(ResidueIssue {
            residue: residue as char,
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
    let mut sink = AnalyzedRecordSink::default();
    scan_fasta_str(input, &mut sink)?;
    Ok(sink.records)
}

pub(crate) fn analyze_fasta_records_reader<R: BufRead>(
    reader: R,
) -> Result<AnalyzedFastaInput, crate::FastaReadError> {
    let mut sink = AnalyzedRecordSink::default();
    let input_hash = scan_fasta_reader(reader, &mut sink)?;

    Ok(AnalyzedFastaInput {
        input_hash,
        records: sink.records,
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

#[derive(Default)]
struct AnalyzedRecordSink {
    records: Vec<AnalyzedProtein>,
    current_sequence: String,
    current_tokens: Vec<u8>,
    current_warnings: Vec<ResidueIssue>,
    current_errors: Vec<ResidueIssue>,
    current_length: usize,
}

impl FastaRecordSink for AnalyzedRecordSink {
    fn push_sequence_line(&mut self, line: &str) {
        if line.is_ascii() {
            for byte in line.bytes() {
                if byte.is_ascii_whitespace() {
                    continue;
                }
                self.push_residue_byte(byte);
            }
            return;
        }

        for residue in normalized_residues(line) {
            self.push_residue(residue);
        }
    }

    fn finish_record(
        &mut self,
        id: String,
        line: usize,
        record_index: usize,
    ) -> Result<(), BioRsError> {
        if self.current_sequence.is_empty() {
            return Err(BioRsError::MissingSequence {
                id,
                line,
                record_index,
            });
        }

        let protein = ProteinSequence {
            id: id.clone(),
            sequence: std::mem::take(&mut self.current_sequence),
        };
        let tokenized = TokenizedProtein {
            id,
            length: std::mem::take(&mut self.current_length),
            alphabet: PROTEIN_20.to_string(),
            valid: self.current_warnings.is_empty() && self.current_errors.is_empty(),
            tokens: std::mem::take(&mut self.current_tokens),
            warnings: std::mem::take(&mut self.current_warnings),
            errors: std::mem::take(&mut self.current_errors),
        };
        self.records.push(AnalyzedProtein { protein, tokenized });
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
            for byte in line.bytes() {
                if byte.is_ascii_whitespace() {
                    continue;
                }
                self.push_residue_byte(byte);
            }
            return;
        }

        for residue in normalized_residues(line) {
            self.push_residue(residue);
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
        let residue = residue.to_ascii_uppercase();
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

impl AnalyzedRecordSink {
    fn push_residue(&mut self, residue: char) {
        self.current_length += 1;
        self.current_sequence.push(residue);
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
        self.current_sequence.push(residue as char);
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

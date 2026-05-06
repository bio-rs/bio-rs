use super::config::{profile_vocabulary_name, protein_special_tokens};
use super::lookup::{
    is_ambiguous_residue_byte, protein_20_token_id, protein_20_token_id_byte,
    push_tokenized_residue, push_tokenized_residue_byte,
};
use super::{ProteinBatchSummary, ProteinTokenizerConfig, TokenizedProtein};
use crate::error::BioRsError;
use crate::fasta_scan::FastaRecordSink;
use crate::sequence::{is_ambiguous_residue, normalized_residues, ResidueIssue};

#[derive(Default)]
pub(super) struct TokenizedRecordSink {
    pub(super) records: Vec<TokenizedProtein>,
    config: ProteinTokenizerConfig,
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

        if self.config.add_special_tokens {
            self.current_tokens
                .push(protein_special_tokens().sep.token_id);
        }

        self.records.push(TokenizedProtein {
            id,
            length: self.current_tokens.len(),
            alphabet: profile_vocabulary_name(self.config.profile),
            valid: self.current_warnings.is_empty() && self.current_errors.is_empty(),
            tokens: std::mem::take(&mut self.current_tokens),
            warnings: std::mem::take(&mut self.current_warnings),
            errors: std::mem::take(&mut self.current_errors),
        });
        self.current_length = 0;
        Ok(())
    }
}

impl TokenizedRecordSink {
    pub(super) fn set_config(&mut self, config: ProteinTokenizerConfig) {
        self.config = config;
    }

    fn push_residue(&mut self, residue: char) {
        if self.current_length == 0 && self.config.add_special_tokens {
            self.current_tokens
                .push(protein_special_tokens().cls.token_id);
        }
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
        if self.current_length == 0 && self.config.add_special_tokens {
            self.current_tokens
                .push(protein_special_tokens().cls.token_id);
        }
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

#[derive(Default)]
pub(super) struct SummaryRecordSink {
    pub(super) summary: ProteinBatchSummary,
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

#[cfg(test)]
mod tests;

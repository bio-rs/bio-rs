use super::lookup::{
    is_ambiguous_residue_byte, protein_20_token_id, protein_20_token_id_byte,
    push_tokenized_residue, push_tokenized_residue_byte,
};
use super::{ProteinBatchSummary, TokenizedProtein};
use crate::fasta_scan::FastaRecordSink;
use crate::sequence::{is_ambiguous_residue, normalized_residues, ResidueIssue, PROTEIN_20};
use crate::BioRsError;

#[derive(Default)]
pub(super) struct TokenizedRecordSink {
    pub(super) records: Vec<TokenizedProtein>,
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
mod tests {
    use super::*;
    use crate::fasta_scan::{scan_fasta_reader, scan_fasta_str};
    use crate::tokenizer::PROTEIN_20_UNKNOWN_TOKEN_ID;
    use std::io::Cursor;

    #[test]
    fn tokenized_record_sink_tokenizes_ascii_sequence_via_bytes() {
        let mut sink = TokenizedRecordSink::default();
        scan_fasta_reader(Cursor::new(b">seq1\nACDE\n"), &mut sink, |_line| {})
            .expect("valid FASTA");

        assert_eq!(sink.records.len(), 1);
        assert_eq!(sink.records[0].id, "seq1");
        assert_eq!(sink.records[0].length, 4);
        assert_eq!(sink.records[0].tokens, vec![0, 1, 2, 3]);
        assert!(sink.records[0].valid);
        assert!(sink.records[0].warnings.is_empty());
        assert!(sink.records[0].errors.is_empty());
    }

    #[test]
    fn tokenized_record_sink_tokenizes_non_ascii_sequence_via_str() {
        let mut sink = TokenizedRecordSink::default();
        scan_fasta_str(">seq1\nacde\n", &mut sink).expect("valid FASTA");

        assert_eq!(sink.records.len(), 1);
        assert_eq!(sink.records[0].length, 4);
        assert_eq!(sink.records[0].tokens, vec![0, 1, 2, 3]);
        assert!(sink.records[0].valid);
    }

    #[test]
    fn tokenized_record_sink_warns_for_ambiguous_residue() {
        let mut sink = TokenizedRecordSink::default();
        scan_fasta_reader(Cursor::new(b">seq1\nACXDE\n"), &mut sink, |_line| {})
            .expect("valid FASTA");

        assert_eq!(
            sink.records[0].tokens,
            vec![0, 1, PROTEIN_20_UNKNOWN_TOKEN_ID, 2, 3]
        );
        assert!(!sink.records[0].valid);
        assert_eq!(sink.records[0].warnings.len(), 1);
        assert_eq!(sink.records[0].warnings[0].residue, 'X');
        assert_eq!(sink.records[0].warnings[0].position, 3);
        assert!(sink.records[0].errors.is_empty());
    }

    #[test]
    fn tokenized_record_sink_errors_for_invalid_residue() {
        let mut sink = TokenizedRecordSink::default();
        scan_fasta_reader(Cursor::new(b">seq1\nAC*DE\n"), &mut sink, |_line| {})
            .expect("valid FASTA");

        assert_eq!(
            sink.records[0].tokens,
            vec![0, 1, PROTEIN_20_UNKNOWN_TOKEN_ID, 2, 3]
        );
        assert!(!sink.records[0].valid);
        assert!(sink.records[0].warnings.is_empty());
        assert_eq!(sink.records[0].errors.len(), 1);
        assert_eq!(sink.records[0].errors[0].residue, '*');
        assert_eq!(sink.records[0].errors[0].position, 3);
    }

    #[test]
    fn tokenized_record_sink_rejects_empty_sequence() {
        let mut sink = TokenizedRecordSink::default();
        let err = scan_fasta_reader(Cursor::new(b">seq1\n>seq2\nACDE\n"), &mut sink, |_line| {})
            .expect_err("empty record should error");

        assert!(matches!(
            err,
            crate::FastaReadError::Parse(crate::BioRsError::MissingSequence { id, line: 1, record_index: 0 })
            if id == "seq1"
        ));
    }

    #[test]
    fn summary_record_sink_summarizes_ascii_sequence() {
        let mut sink = SummaryRecordSink::default();
        scan_fasta_reader(
            Cursor::new(b">seq1\nACDE\nFGHI\n>seq2\nKLMN\n"),
            &mut sink,
            |_line| {},
        )
        .expect("valid FASTA");

        assert_eq!(sink.summary.records, 2);
        assert_eq!(sink.summary.total_length, 12);
        assert_eq!(sink.summary.valid_records, 2);
        assert_eq!(sink.summary.warning_count, 0);
        assert_eq!(sink.summary.error_count, 0);
    }

    #[test]
    fn summary_record_sink_counts_warnings_and_errors() {
        let mut sink = SummaryRecordSink::default();
        scan_fasta_reader(
            Cursor::new(b">seq1\nACXDE\n>seq2\nAC*DE\n>seq3\nACDE\n"),
            &mut sink,
            |_line| {},
        )
        .expect("valid FASTA");

        assert_eq!(sink.summary.records, 3);
        assert_eq!(sink.summary.total_length, 14);
        assert_eq!(sink.summary.valid_records, 1);
        assert_eq!(sink.summary.warning_count, 1);
        assert_eq!(sink.summary.error_count, 1);
    }

    #[test]
    fn summary_record_sink_rejects_empty_sequence() {
        let mut sink = SummaryRecordSink::default();
        let err = scan_fasta_reader(Cursor::new(b">seq1\n>seq2\nACDE\n"), &mut sink, |_line| {})
            .expect_err("empty record should error");

        assert!(matches!(
            err,
            crate::FastaReadError::Parse(crate::BioRsError::MissingSequence { id, line: 1, record_index: 0 })
            if id == "seq1"
        ));
    }
}

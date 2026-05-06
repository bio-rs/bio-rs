use crate::error::BioRsError;
use crate::error::FastaReadError;
use crate::fasta_scan::{scan_fasta_reader, scan_fasta_str, FastaRecordSink};
use crate::sequence::{
    append_normalized_sequence_bytes_to_vec, append_normalized_sequence_to_vec,
    validate_protein_sequence_owned,
};
use crate::sequence::{ProteinSequence, SequenceValidationReport};
use crate::verification::StableInputHasher;
use serde::{Deserialize, Serialize};
use std::io::BufRead;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ParsedFastaInput {
    pub input_hash: String,
    pub records: Vec<ProteinSequence>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValidatedFastaInput {
    pub input_hash: String,
    pub report: SequenceValidationReport,
}

/// Parse FASTA text into normalized protein sequence records.
pub fn parse_fasta_records(input: &str) -> Result<Vec<ProteinSequence>, BioRsError> {
    let mut sink = ParsedRecordSink::default();
    scan_fasta_str(input, &mut sink)?;
    Ok(sink.records)
}

/// Parse FASTA records from a buffered reader without preloading the full input.
pub fn parse_fasta_records_reader<R: BufRead>(
    reader: R,
) -> Result<ParsedFastaInput, FastaReadError> {
    let mut sink = ParsedRecordSink::default();
    let mut hasher = StableInputHasher::new();
    scan_fasta_reader(reader, &mut sink, |line| hasher.update(line))?;

    Ok(ParsedFastaInput {
        input_hash: hasher.finalize(),
        records: sink.records,
    })
}

/// Validate FASTA text and return aggregate sequence validation details.
pub fn validate_fasta_input(input: &str) -> Result<SequenceValidationReport, BioRsError> {
    let mut sink = ValidatedRecordSink::default();
    scan_fasta_str(input, &mut sink)?;
    Ok(sink.report)
}

/// Validate FASTA from a buffered reader and discard the raw input hash.
pub fn validate_fasta_reader<R: BufRead>(
    reader: R,
) -> Result<SequenceValidationReport, FastaReadError> {
    Ok(validate_fasta_reader_with_hash(reader)?.report)
}

/// Validate FASTA from a buffered reader and include a stable raw input hash.
pub fn validate_fasta_reader_with_hash<R: BufRead>(
    reader: R,
) -> Result<ValidatedFastaInput, FastaReadError> {
    let mut sink = ValidatedRecordSink::default();
    let mut hasher = StableInputHasher::new();
    scan_fasta_reader(reader, &mut sink, |line| hasher.update(line))?;
    Ok(ValidatedFastaInput {
        input_hash: hasher.finalize(),
        report: sink.report,
    })
}

#[derive(Default)]
struct ParsedRecordSink {
    records: Vec<ProteinSequence>,
    sequence: Vec<u8>,
}

impl FastaRecordSink for ParsedRecordSink {
    fn push_sequence_line(&mut self, line: &str) {
        append_normalized_sequence_to_vec(line, &mut self.sequence);
    }

    fn push_sequence_line_bytes(&mut self, line: &[u8]) {
        append_normalized_sequence_bytes_to_vec(line, &mut self.sequence);
    }

    fn finish_record(
        &mut self,
        id: String,
        line: usize,
        record_index: usize,
    ) -> Result<(), BioRsError> {
        if self.sequence.is_empty() {
            return Err(BioRsError::MissingSequence {
                id,
                line,
                record_index,
            });
        }

        self.records.push(ProteinSequence {
            id,
            sequence: std::mem::take(&mut self.sequence),
        });
        Ok(())
    }
}

#[derive(Default)]
struct ValidatedRecordSink {
    report: SequenceValidationReport,
    current_sequence: Vec<u8>,
}

impl FastaRecordSink for ValidatedRecordSink {
    fn push_sequence_line(&mut self, line: &str) {
        append_normalized_sequence_to_vec(line, &mut self.current_sequence);
    }

    fn push_sequence_line_bytes(&mut self, line: &[u8]) {
        append_normalized_sequence_bytes_to_vec(line, &mut self.current_sequence);
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

        let validated =
            validate_protein_sequence_owned(id, std::mem::take(&mut self.current_sequence));
        if validated.valid {
            self.report.valid_records += 1;
        }
        self.report.records += 1;
        self.report.warning_count += validated.warnings.len();
        self.report.error_count += validated.errors.len();
        self.report.sequences.push(validated);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fasta_scan::scan_fasta_str;

    #[test]
    fn parsed_record_sink_collects_normalized_records() {
        let mut sink = ParsedRecordSink::default();
        scan_fasta_str(">seq1\nACDE\nFGHI\n>seq2\nKLMN\n", &mut sink).expect("valid FASTA");

        assert_eq!(sink.records.len(), 2);
        assert_eq!(sink.records[0].id, "seq1");
        assert_eq!(sink.records[0].sequence, b"ACDEFGHI");
        assert_eq!(sink.records[1].id, "seq2");
        assert_eq!(sink.records[1].sequence, b"KLMN");
    }

    #[test]
    fn parsed_record_sink_rejects_empty_sequence() {
        let mut sink = ParsedRecordSink::default();
        let err = scan_fasta_str(">seq1\n>seq2\nACDE\n", &mut sink)
            .expect_err("empty record should error");

        assert!(matches!(
            err,
            BioRsError::MissingSequence { id, line: 1, record_index: 0 }
            if id == "seq1"
        ));
    }

    #[test]
    fn validated_record_sink_reports_valid_sequence() {
        let mut sink = ValidatedRecordSink::default();
        scan_fasta_str(">seq1\nACDE\n", &mut sink).expect("valid FASTA");

        assert_eq!(sink.report.records, 1);
        assert_eq!(sink.report.valid_records, 1);
        assert_eq!(sink.report.warning_count, 0);
        assert_eq!(sink.report.error_count, 0);
        assert_eq!(sink.report.sequences.len(), 1);
        assert!(sink.report.sequences[0].valid);
        assert_eq!(sink.report.sequences[0].sequence, "ACDE");
    }

    #[test]
    fn validated_record_sink_reports_warning_for_ambiguous_residue() {
        let mut sink = ValidatedRecordSink::default();
        scan_fasta_str(">seq1\nACXDE\n", &mut sink).expect("valid FASTA");

        assert_eq!(sink.report.records, 1);
        assert_eq!(sink.report.valid_records, 0);
        assert_eq!(sink.report.warning_count, 1);
        assert_eq!(sink.report.error_count, 0);
        assert!(!sink.report.sequences[0].valid);
        assert_eq!(sink.report.sequences[0].warnings[0].residue, 'X');
        assert_eq!(sink.report.sequences[0].warnings[0].position, 3);
    }

    #[test]
    fn validated_record_sink_reports_error_for_invalid_residue() {
        let mut sink = ValidatedRecordSink::default();
        scan_fasta_str(">seq1\nAC*DE\n", &mut sink).expect("valid FASTA");

        assert_eq!(sink.report.records, 1);
        assert_eq!(sink.report.valid_records, 0);
        assert_eq!(sink.report.warning_count, 0);
        assert_eq!(sink.report.error_count, 1);
        assert!(!sink.report.sequences[0].valid);
        assert_eq!(sink.report.sequences[0].errors[0].residue, '*');
        assert_eq!(sink.report.sequences[0].errors[0].position, 3);
    }

    #[test]
    fn validated_record_sink_rejects_empty_sequence() {
        let mut sink = ValidatedRecordSink::default();
        let err = scan_fasta_str(">seq1\n>seq2\nACDE\n", &mut sink)
            .expect_err("empty record should error");

        assert!(matches!(
            err,
            BioRsError::MissingSequence { id, line: 1, record_index: 0 }
            if id == "seq1"
        ));
    }
}

use crate::error::BioRsError;
use crate::fasta_scan::{scan_fasta_reader, scan_fasta_str, FastaRecordSink};
use crate::sequence::{
    append_normalized_sequence, append_normalized_sequence_bytes, is_ambiguous_residue,
    is_protein_20_residue, ResidueIssue, ValidatedSequence, PROTEIN_20,
};
use crate::{FastaReadError, ProteinSequence, SequenceValidationReport};
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

pub fn parse_fasta_records(input: &str) -> Result<Vec<ProteinSequence>, BioRsError> {
    let mut sink = ParsedRecordSink::default();
    scan_fasta_str(input, &mut sink)?;
    Ok(sink.records)
}

pub fn parse_fasta_records_reader<R: BufRead>(
    reader: R,
) -> Result<ParsedFastaInput, FastaReadError> {
    let mut sink = ParsedRecordSink::default();
    let input_hash = scan_fasta_reader(reader, &mut sink)?;

    Ok(ParsedFastaInput {
        input_hash,
        records: sink.records,
    })
}

pub fn validate_fasta_input(input: &str) -> Result<SequenceValidationReport, BioRsError> {
    let mut sink = ValidatedRecordSink::default();
    scan_fasta_str(input, &mut sink)?;
    Ok(sink.report)
}

pub fn validate_fasta_reader<R: BufRead>(
    reader: R,
) -> Result<SequenceValidationReport, FastaReadError> {
    Ok(validate_fasta_reader_with_hash(reader)?.report)
}

pub fn validate_fasta_reader_with_hash<R: BufRead>(
    reader: R,
) -> Result<ValidatedFastaInput, FastaReadError> {
    let mut sink = ValidatedRecordSink::default();
    let input_hash = scan_fasta_reader(reader, &mut sink)?;
    Ok(ValidatedFastaInput {
        input_hash,
        report: sink.report,
    })
}

#[derive(Default)]
struct ParsedRecordSink {
    records: Vec<ProteinSequence>,
    sequence: String,
}

impl FastaRecordSink for ParsedRecordSink {
    fn push_sequence_line(&mut self, line: &str) {
        append_normalized_sequence(line, &mut self.sequence);
    }

    fn push_sequence_line_bytes(&mut self, line: &[u8]) {
        append_normalized_sequence_bytes(line, &mut self.sequence);
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
    current_sequence: String,
    current_warnings: Vec<ResidueIssue>,
    current_errors: Vec<ResidueIssue>,
    current_length: usize,
}

impl FastaRecordSink for ValidatedRecordSink {
    fn push_sequence_line(&mut self, line: &str) {
        if line.is_ascii() {
            self.push_sequence_line_bytes(line.as_bytes());
            return;
        }

        for residue in line
            .chars()
            .filter(|residue| !residue.is_whitespace())
            .map(|residue| residue.to_ascii_uppercase())
        {
            self.push_residue(residue);
        }
    }

    fn push_sequence_line_bytes(&mut self, line: &[u8]) {
        self.current_sequence.reserve(line.len());
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

        let warnings = std::mem::take(&mut self.current_warnings);
        let errors = std::mem::take(&mut self.current_errors);
        let valid = warnings.is_empty() && errors.is_empty();
        if valid {
            self.report.valid_records += 1;
        }
        self.report.records += 1;
        self.report.warning_count += warnings.len();
        self.report.error_count += errors.len();
        self.report.sequences.push(ValidatedSequence {
            id,
            sequence: std::mem::take(&mut self.current_sequence),
            alphabet: PROTEIN_20.to_string(),
            valid,
            warnings,
            errors,
        });
        self.current_length = 0;
        Ok(())
    }
}

impl ValidatedRecordSink {
    fn push_residue(&mut self, residue: char) {
        self.current_length += 1;
        self.current_sequence.push(residue);
        if is_protein_20_residue(residue) {
            return;
        }

        let issue = ResidueIssue {
            residue,
            position: self.current_length,
        };
        if is_ambiguous_residue(residue) {
            self.current_warnings.push(issue);
        } else {
            self.current_errors.push(issue);
        }
    }

    fn push_residue_byte(&mut self, residue: u8) {
        self.push_residue(residue.to_ascii_uppercase() as char);
    }
}

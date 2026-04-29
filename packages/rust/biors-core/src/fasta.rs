use crate::error::BioRsError;
use crate::fasta_scan::{scan_fasta_reader, scan_fasta_str, FastaRecordSink};
use crate::sequence::{
    append_normalized_sequence, append_normalized_sequence_bytes, summarize_validated_sequences,
};
use crate::tokenizer::{analyze_fasta_records, validated_sequences_from_analyzed};
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
    let analyzed = analyze_fasta_records(input)?;
    let validated = validated_sequences_from_analyzed(&analyzed);
    Ok(summarize_validated_sequences(validated))
}

pub fn validate_fasta_reader<R: BufRead>(
    reader: R,
) -> Result<SequenceValidationReport, FastaReadError> {
    Ok(validate_fasta_reader_with_hash(reader)?.report)
}

pub fn validate_fasta_reader_with_hash<R: BufRead>(
    reader: R,
) -> Result<ValidatedFastaInput, FastaReadError> {
    let analyzed = crate::tokenizer::analyze_fasta_records_reader(reader)?;
    let validated = validated_sequences_from_analyzed(&analyzed.records);
    Ok(ValidatedFastaInput {
        input_hash: analyzed.input_hash,
        report: summarize_validated_sequences(validated),
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

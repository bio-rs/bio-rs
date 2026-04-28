use crate::error::BioRsError;
use crate::sequence::{append_normalized_sequence, summarize_validated_sequences};
use crate::tokenizer::{analyze_fasta_records, validated_sequences_from_analyzed};
use crate::verification::StableInputHasher;
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
    if input.trim().is_empty() {
        return Err(BioRsError::EmptyInput);
    }

    let mut records = Vec::new();
    let mut current_id: Option<String> = None;
    let mut sequence = String::new();
    let mut current_header_line = 0;
    let mut current_record_index = 0;

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
                push_fasta_record(
                    &mut records,
                    id,
                    &mut sequence,
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
            append_normalized_sequence(line, &mut sequence);
        }
    }

    let id = current_id.ok_or(BioRsError::MissingHeader { line: 1 })?;
    push_fasta_record(
        &mut records,
        id,
        &mut sequence,
        current_header_line,
        current_record_index,
    )?;

    Ok(records)
}

pub fn parse_fasta_records_reader<R: BufRead>(
    mut reader: R,
) -> Result<ParsedFastaInput, FastaReadError> {
    let mut records = Vec::new();
    let mut current_id: Option<String> = None;
    let mut sequence = String::new();
    let mut current_header_line = 0;
    let mut current_record_index = 0;
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
                push_fasta_record(
                    &mut records,
                    id,
                    &mut sequence,
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
            append_normalized_sequence(line, &mut sequence);
        }
    }

    if line_number == 0 || records.is_empty() && current_id.is_none() {
        return Err(BioRsError::EmptyInput.into());
    }

    let id = current_id.ok_or(BioRsError::MissingHeader { line: 1 })?;
    push_fasta_record(
        &mut records,
        id,
        &mut sequence,
        current_header_line,
        current_record_index,
    )?;

    Ok(ParsedFastaInput {
        input_hash: hasher.finalize(),
        records,
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

fn fasta_id(header: &str) -> Option<String> {
    header.split_whitespace().next().map(str::to_string)
}

fn push_fasta_record(
    records: &mut Vec<ProteinSequence>,
    id: String,
    sequence: &mut String,
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

    records.push(ProteinSequence {
        id,
        sequence: std::mem::take(sequence),
    });
    Ok(())
}

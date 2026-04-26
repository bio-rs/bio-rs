use crate::error::BioRsError;
use crate::sequence::{
    normalize_sequence, summarize_validated_sequences, validate_protein_sequence,
};
use crate::{ProteinSequence, SequenceValidationReport};

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
            if let Some(id) = current_id.replace(fasta_id(header)) {
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
            sequence.push_str(&normalize_sequence(line));
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

pub fn validate_fasta_input(input: &str) -> Result<SequenceValidationReport, BioRsError> {
    let records = parse_fasta_records(input)?;
    let validated = records.iter().map(validate_protein_sequence).collect();
    Ok(summarize_validated_sequences(validated))
}

fn fasta_id(header: &str) -> String {
    header
        .split_whitespace()
        .next()
        .unwrap_or_default()
        .to_string()
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

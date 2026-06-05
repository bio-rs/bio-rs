use std::io::BufRead;

use crate::formats::FormatMetadata;
use crate::verification::StableInputHasher;

use super::error::{FastqParseError, FormatReadError};
use super::types::{FastqRecord, ParsedFastqInput};

/// Parse FASTQ text into read records.
pub fn parse_fastq_records(input: &str) -> Result<Vec<FastqRecord>, FastqParseError> {
    parse_fastq_records_reader(input.as_bytes())
        .map(|parsed| parsed.records)
        .map_err(|error| match error {
            FormatReadError::FastqParse(error) => error,
            FormatReadError::Io(_) => unreachable!("in-memory FASTQ parsing cannot fail with I/O"),
        })
}

/// Parse FASTQ records from a buffered reader without preloading the full input.
pub fn parse_fastq_records_reader<R: BufRead>(
    reader: R,
) -> Result<ParsedFastqInput, FormatReadError> {
    let parsed = parse_fastq_stream(reader)?;
    Ok(ParsedFastqInput {
        input_hash: parsed.input_hash,
        records: parsed.records,
    })
}

pub(super) struct ParsedFastqStream {
    pub input_hash: String,
    pub records: Vec<FastqRecord>,
}

pub(super) fn parse_fastq_stream<R: BufRead>(
    mut reader: R,
) -> Result<ParsedFastqStream, FormatReadError> {
    let mut hasher = StableInputHasher::new();
    let mut cursor = LineCursor::default();
    let mut records = Vec::new();

    loop {
        let Some((line_start, header)) = cursor.read_line(&mut reader, &mut hasher)? else {
            if records.is_empty() {
                return Err(FastqParseError::EmptyInput.into());
            }
            break;
        };

        let record_index = records.len();
        if !header.starts_with('@') {
            return Err(FastqParseError::MissingHeader {
                line: line_start,
                record_index,
            }
            .into());
        }

        let HeaderParts { id, description } =
            parse_header_payload(&header[1..], line_start, record_index)?;
        let (sequence, separator_line, separator_id) = read_sequence_body(
            &mut reader,
            &mut cursor,
            &mut hasher,
            &id,
            line_start,
            record_index,
        )?;
        reject_separator_identifier_mismatch(&id, separator_id, separator_line, record_index)?;
        let (quality, line_end) = read_quality_body(
            &mut reader,
            &mut cursor,
            &mut hasher,
            &id,
            sequence.chars().count(),
            record_index,
        )?;
        records.push(FastqRecord {
            id,
            description,
            sequence: sequence.to_ascii_uppercase(),
            quality,
            metadata: FormatMetadata::new(record_index, line_start, line_end),
        });
    }

    Ok(ParsedFastqStream {
        input_hash: hasher.finalize(),
        records,
    })
}

fn reject_separator_identifier_mismatch(
    id: &str,
    separator_id: Option<String>,
    line: usize,
    record_index: usize,
) -> Result<(), FormatReadError> {
    if let Some(separator_id) = separator_id {
        if separator_id != id {
            return Err(FastqParseError::SeparatorIdentifierMismatch {
                id: id.to_string(),
                separator_id,
                line,
                record_index,
            }
            .into());
        }
    }
    Ok(())
}

fn read_sequence_body<R: BufRead>(
    reader: &mut R,
    cursor: &mut LineCursor,
    hasher: &mut StableInputHasher,
    id: &str,
    header_line: usize,
    record_index: usize,
) -> Result<(String, usize, Option<String>), FormatReadError> {
    let mut sequence = String::new();
    loop {
        let Some((line, value)) = cursor.read_line(reader, hasher)? else {
            return Err(FastqParseError::MissingSeparator {
                id: id.to_string(),
                line: header_line,
                record_index,
            }
            .into());
        };
        if let Some(separator_id) = value.strip_prefix('+') {
            if sequence.is_empty() {
                return Err(FastqParseError::MissingSequence {
                    id: id.to_string(),
                    line: header_line,
                    record_index,
                }
                .into());
            }
            let separator_id = first_token(separator_id.trim()).map(str::to_string);
            return Ok((sequence, line, separator_id));
        }
        sequence.push_str(value.trim());
    }
}

fn read_quality_body<R: BufRead>(
    reader: &mut R,
    cursor: &mut LineCursor,
    hasher: &mut StableInputHasher,
    id: &str,
    expected_len: usize,
    record_index: usize,
) -> Result<(String, usize), FormatReadError> {
    let mut quality = String::new();
    let mut last_line = cursor.line_number;
    while quality.len() < expected_len {
        let Some((line, value)) = cursor.read_line(reader, hasher)? else {
            return Err(FastqParseError::MissingQuality {
                id: id.to_string(),
                expected: expected_len,
                observed: quality.len(),
                record_index,
            }
            .into());
        };
        last_line = line;
        quality.push_str(&value);
    }

    if quality.len() != expected_len {
        return Err(FastqParseError::QualityLengthMismatch {
            id: id.to_string(),
            expected: expected_len,
            observed: quality.len(),
            line: last_line,
            record_index,
        }
        .into());
    }

    Ok((quality, last_line))
}

#[derive(Default)]
struct LineCursor {
    line_number: usize,
}

impl LineCursor {
    fn read_line<R: BufRead>(
        &mut self,
        reader: &mut R,
        hasher: &mut StableInputHasher,
    ) -> Result<Option<(usize, String)>, FormatReadError> {
        let mut raw = String::new();
        let bytes = reader.read_line(&mut raw)?;
        if bytes == 0 {
            return Ok(None);
        }
        self.line_number += 1;
        hasher.update(raw.as_bytes());
        Ok(Some((self.line_number, trim_line_end(&raw).to_string())))
    }
}

struct HeaderParts {
    id: String,
    description: Option<String>,
}

fn parse_header_payload(
    payload: &str,
    line: usize,
    record_index: usize,
) -> Result<HeaderParts, FastqParseError> {
    let payload = payload.trim();
    let Some(id) = first_token(payload) else {
        return Err(FastqParseError::MissingIdentifier { line, record_index });
    };
    let description = payload
        .strip_prefix(id)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string);
    Ok(HeaderParts {
        id: id.to_string(),
        description,
    })
}

fn first_token(value: &str) -> Option<&str> {
    value
        .split_whitespace()
        .next()
        .filter(|token| !token.is_empty())
}

fn trim_line_end(value: &str) -> &str {
    value.trim_end_matches(['\n', '\r'])
}

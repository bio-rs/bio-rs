use crate::formats::{BioFormat, FormatField, FormatMetadata};

use super::error::{FastqParseError, FormatReadError};
use super::parse::parse_fastq_records;
use super::validation::validate_fastq_reader;

#[test]
fn parse_fastq_accepts_multiline_sequence_and_quality() {
    let parsed = parse_fastq_records("@r1 sample\nacg\nTN\n+\n!!!!!\n").expect("valid FASTQ");

    assert_eq!(parsed.len(), 1);
    assert_eq!(parsed[0].id, "r1");
    assert_eq!(parsed[0].description.as_deref(), Some("sample"));
    assert_eq!(parsed[0].sequence, "ACGTN");
    assert_eq!(parsed[0].quality, "!!!!!");
    assert_eq!(parsed[0].metadata, FormatMetadata::new(0, 1, 5));

    let shared = parsed[0].to_format_record();
    assert_eq!(shared.format, BioFormat::Fastq);
    assert_eq!(shared.fields[0], FormatField::new("sequence", "ACGTN"));
}

#[test]
fn parse_fastq_rejects_quality_length_mismatch() {
    let error = validate_fastq_reader("@r1\nACG\n+\n!!!!\n".as_bytes()).expect_err("mismatch");

    assert!(matches!(
        error,
        FormatReadError::FastqParse(FastqParseError::QualityLengthMismatch {
            expected: 3,
            observed: 4,
            record_index: 0,
            ..
        })
    ));
    assert_eq!(error.code(), "fastq.quality_length_mismatch");
}

#[test]
fn parse_fastq_rejects_separator_identifier_mismatch() {
    let error = validate_fastq_reader("@r1\nACG\n+r2\n!!!\n".as_bytes()).expect_err("mismatch");

    assert!(matches!(
        error,
        FormatReadError::FastqParse(FastqParseError::SeparatorIdentifierMismatch {
            id,
            separator_id,
            ..
        }) if id == "r1" && separator_id == "r2"
    ));
}

#[test]
fn parse_fastq_rejects_empty_input() {
    let error = validate_fastq_reader("".as_bytes()).expect_err("empty");

    assert!(matches!(
        error,
        FormatReadError::FastqParse(FastqParseError::EmptyInput)
    ));
}

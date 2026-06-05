use std::io::BufRead;

use crate::formats::BioFormat;
use crate::sequence::{AlphabetPolicy, SequenceKind, SymbolClass};

use super::error::FormatReadError;
use super::parse::parse_fastq_stream;
use super::types::{
    FastqRecord, FastqRecordValidation, FastqValidationIssue, FastqValidationReport,
    ValidatedFastqInput,
};

/// Validate FASTQ records from a buffered reader and discard the raw input hash.
pub fn validate_fastq_reader<R: BufRead>(
    reader: R,
) -> Result<FastqValidationReport, FormatReadError> {
    Ok(validate_fastq_reader_with_hash(reader)?.report)
}

/// Validate FASTQ records from a buffered reader and include a stable raw input hash.
pub fn validate_fastq_reader_with_hash<R: BufRead>(
    reader: R,
) -> Result<ValidatedFastqInput, FormatReadError> {
    let parsed = parse_fastq_stream(reader)?;
    Ok(ValidatedFastqInput {
        input_hash: parsed.input_hash,
        report: summarize_fastq_records(&parsed.records),
    })
}

fn summarize_fastq_records(records: &[FastqRecord]) -> FastqValidationReport {
    let record_reports: Vec<_> = records.iter().map(validate_fastq_record).collect();
    FastqValidationReport {
        format: BioFormat::Fastq,
        sequence_kind: SequenceKind::Dna,
        records: record_reports.len(),
        valid_records: record_reports.iter().filter(|record| record.valid).count(),
        warning_count: record_reports
            .iter()
            .map(|record| record.warnings.len())
            .sum(),
        error_count: record_reports
            .iter()
            .map(|record| record.errors.len())
            .sum(),
        record_reports,
    }
}

fn validate_fastq_record(record: &FastqRecord) -> FastqRecordValidation {
    let mut warnings = Vec::new();
    let mut errors = Vec::new();
    validate_fastq_sequence(&record.sequence, &mut warnings, &mut errors);
    validate_fastq_quality(&record.quality, &mut errors);
    FastqRecordValidation {
        id: record.id.clone(),
        description: record.description.clone(),
        sequence_length: record.sequence.chars().count(),
        quality_length: fastq_quality_symbol_count(&record.quality),
        metadata: record.metadata.clone(),
        valid: warnings.is_empty() && errors.is_empty(),
        warnings,
        errors,
    }
}

fn validate_fastq_sequence(
    sequence: &str,
    warnings: &mut Vec<FastqValidationIssue>,
    errors: &mut Vec<FastqValidationIssue>,
) {
    let policy = AlphabetPolicy::for_kind(SequenceKind::Dna);
    for (position, symbol) in sequence.chars().enumerate() {
        match policy.classify(symbol) {
            SymbolClass::Standard => {}
            SymbolClass::Ambiguous => {
                warnings.push(FastqValidationIssue::ambiguous(symbol, position + 1));
            }
            SymbolClass::Invalid => {
                errors.push(FastqValidationIssue::invalid_symbol(symbol, position + 1));
            }
        }
    }
}

pub(crate) fn validate_fastq_quality(quality: &str, errors: &mut Vec<FastqValidationIssue>) {
    for (position, symbol) in quality.chars().enumerate() {
        if !matches!(symbol as u32, 33..=126) {
            errors.push(FastqValidationIssue::invalid_quality_character(
                symbol,
                position + 1,
            ));
        }
    }
}

pub(crate) fn fastq_quality_symbol_count(quality: &str) -> usize {
    quality.chars().count()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::formats::FastqValidationIssueCode;

    #[test]
    fn validate_fastq_reports_ambiguous_symbols_without_repeating_quality_payload() {
        let output = validate_fastq_reader("@r1\nACGN\n+\n!!!!\n".as_bytes()).expect("valid FASTQ");

        assert_eq!(output.records, 1);
        assert_eq!(output.valid_records, 0);
        assert_eq!(output.warning_count, 1);
        assert_eq!(output.error_count, 0);
        assert_eq!(
            output.record_reports[0].warnings[0].code.as_str(),
            "ambiguous_symbol"
        );
        assert_eq!(output.record_reports[0].sequence_length, 4);
        assert_eq!(output.record_reports[0].quality_length, 4);
    }

    #[test]
    fn validate_fastq_reports_invalid_quality_characters() {
        let output = validate_fastq_reader("@r1\nACG\n+\n!! \n".as_bytes()).expect("valid FASTQ");

        assert_eq!(output.records, 1);
        assert_eq!(output.valid_records, 0);
        assert_eq!(output.error_count, 1);
        assert_eq!(
            output.record_reports[0].errors[0].code,
            FastqValidationIssueCode::InvalidQualityCharacter
        );
    }

    #[test]
    fn validate_fastq_reports_non_ascii_quality_as_invalid_and_counts_symbols() {
        let output = validate_fastq_reader("@r1\nAC\n+\né!\n".as_bytes()).expect("valid FASTQ");

        assert_eq!(output.records, 1);
        assert_eq!(output.valid_records, 0);
        assert_eq!(output.error_count, 1);
        assert_eq!(output.record_reports[0].sequence_length, 2);
        assert_eq!(output.record_reports[0].quality_length, 2);
        assert_eq!(
            output.record_reports[0].errors[0].code,
            FastqValidationIssueCode::InvalidQualityCharacter
        );
    }
}

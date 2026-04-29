use super::{SequenceValidationReport, ValidatedSequence};

/// Summarize per-record validation results into a batch validation report.
pub fn summarize_validated_sequences(
    sequences: Vec<ValidatedSequence>,
) -> SequenceValidationReport {
    SequenceValidationReport {
        records: sequences.len(),
        valid_records: sequences.iter().filter(|sequence| sequence.valid).count(),
        warning_count: sequences
            .iter()
            .map(|sequence| sequence.warnings.len())
            .sum(),
        error_count: sequences.iter().map(|sequence| sequence.errors.len()).sum(),
        sequences,
    }
}

use super::{
    KindAwareSequenceValidationReport, SequenceValidationReport, ValidatedSequence,
    ValidatedSequenceRecord,
};

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

/// Summarize kind-aware per-record validation results into a batch report.
pub fn summarize_validated_sequence_records(
    sequences: Vec<ValidatedSequenceRecord>,
) -> KindAwareSequenceValidationReport {
    let mut report = KindAwareSequenceValidationReport {
        records: sequences.len(),
        valid_records: sequences.iter().filter(|sequence| sequence.valid).count(),
        warning_count: sequences
            .iter()
            .map(|sequence| sequence.warnings.len())
            .sum(),
        error_count: sequences.iter().map(|sequence| sequence.errors.len()).sum(),
        sequences,
        ..KindAwareSequenceValidationReport::default()
    };

    for sequence in &report.sequences {
        report.kind_counts.increment(sequence.kind);
    }

    report
}

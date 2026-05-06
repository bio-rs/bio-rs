use crate::error::{BioRsError, FastaReadError};
use crate::fasta_scan::{scan_fasta_reader, scan_fasta_str, FastaRecordSink};
use crate::sequence::{
    append_normalized_sequence, append_normalized_sequence_bytes, detect_sequence_kind,
    summarize_validated_sequence_records, KindAwareSequenceValidationReport,
    KindAwareSequenceValidationSummary, SequenceKindSelection, SequenceRecord,
    ValidatedSequenceRecord,
};
use crate::verification::StableInputHasher;
use serde::{Deserialize, Serialize};
use std::io::BufRead;

/// Kind-aware FASTA validation report plus a stable hash of the raw reader input.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValidatedKindAwareFastaInput {
    /// Stable hash of the exact bytes read from the input stream.
    pub input_hash: String,
    /// Aggregate kind-aware sequence validation report.
    pub report: KindAwareSequenceValidationReport,
}

/// Kind-aware FASTA validation summary plus a stable hash of the raw reader input.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValidatedKindAwareFastaSummaryInput {
    /// Stable hash of the exact bytes read from the input stream.
    pub input_hash: String,
    /// Aggregate kind-aware sequence validation summary without per-record payloads.
    pub summary: KindAwareSequenceValidationSummary,
}

/// Validate FASTA text with explicit or per-record detected sequence kinds.
pub fn validate_fasta_input_with_kind(
    input: &str,
    selection: SequenceKindSelection,
) -> Result<KindAwareSequenceValidationReport, BioRsError> {
    let mut sink = KindAwareValidatedRecordSink::new(selection);
    scan_fasta_str(input, &mut sink)?;
    Ok(sink.finish())
}

/// Validate FASTA from a buffered reader with explicit or detected sequence kinds.
pub fn validate_fasta_reader_with_kind<R: BufRead>(
    reader: R,
    selection: SequenceKindSelection,
) -> Result<KindAwareSequenceValidationReport, FastaReadError> {
    Ok(validate_fasta_reader_with_kind_and_hash(reader, selection)?.report)
}

/// Validate FASTA from a buffered reader with kind-aware output and raw input hash.
pub fn validate_fasta_reader_with_kind_and_hash<R: BufRead>(
    reader: R,
    selection: SequenceKindSelection,
) -> Result<ValidatedKindAwareFastaInput, FastaReadError> {
    let mut sink = KindAwareValidatedRecordSink::new(selection);
    let mut hasher = StableInputHasher::new();
    scan_fasta_reader(reader, &mut sink, |line| hasher.update(line))?;
    Ok(ValidatedKindAwareFastaInput {
        input_hash: hasher.finalize(),
        report: sink.finish(),
    })
}

/// Summarize kind-aware FASTA validation from a buffered reader without retaining per-record payloads.
pub fn validate_fasta_reader_summary_with_kind_and_hash<R: BufRead>(
    reader: R,
    selection: SequenceKindSelection,
) -> Result<ValidatedKindAwareFastaSummaryInput, FastaReadError> {
    let mut sink = KindAwareValidationSummarySink::new(selection);
    let mut hasher = StableInputHasher::new();
    scan_fasta_reader(reader, &mut sink, |line| hasher.update(line))?;
    Ok(ValidatedKindAwareFastaSummaryInput {
        input_hash: hasher.finalize(),
        summary: sink.finish(),
    })
}

struct KindAwareValidatedRecordSink {
    selection: SequenceKindSelection,
    sequences: Vec<ValidatedSequenceRecord>,
    current_sequence: String,
}

impl KindAwareValidatedRecordSink {
    fn new(selection: SequenceKindSelection) -> Self {
        Self {
            selection,
            sequences: Vec::new(),
            current_sequence: String::new(),
        }
    }

    fn finish(self) -> KindAwareSequenceValidationReport {
        summarize_validated_sequence_records(self.sequences)
    }
}

impl FastaRecordSink for KindAwareValidatedRecordSink {
    fn push_sequence_line(&mut self, line: &str) {
        append_normalized_sequence(line, &mut self.current_sequence);
    }

    fn push_sequence_line_bytes(&mut self, line: &[u8]) {
        append_normalized_sequence_bytes(line, &mut self.current_sequence);
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

        let sequence = std::mem::take(&mut self.current_sequence);
        let kind = self
            .selection
            .explicit_kind()
            .unwrap_or_else(|| detect_sequence_kind(&sequence));
        let record = SequenceRecord { id, sequence, kind };
        self.sequences
            .push(crate::sequence::validate_sequence_record(&record));
        Ok(())
    }
}

struct KindAwareValidationSummarySink {
    selection: SequenceKindSelection,
    summary: KindAwareSequenceValidationSummary,
    current_sequence: String,
}

impl KindAwareValidationSummarySink {
    fn new(selection: SequenceKindSelection) -> Self {
        Self {
            selection,
            summary: KindAwareSequenceValidationSummary::default(),
            current_sequence: String::new(),
        }
    }

    fn finish(self) -> KindAwareSequenceValidationSummary {
        self.summary
    }
}

impl FastaRecordSink for KindAwareValidationSummarySink {
    fn push_sequence_line(&mut self, line: &str) {
        append_normalized_sequence(line, &mut self.current_sequence);
    }

    fn push_sequence_line_bytes(&mut self, line: &[u8]) {
        append_normalized_sequence_bytes(line, &mut self.current_sequence);
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

        let sequence = std::mem::take(&mut self.current_sequence);
        let kind = self
            .selection
            .explicit_kind()
            .unwrap_or_else(|| detect_sequence_kind(&sequence));
        let record = SequenceRecord { id, sequence, kind };
        let validated = crate::sequence::validate_sequence_record(&record);
        self.summary.add_record(&validated);
        Ok(())
    }
}

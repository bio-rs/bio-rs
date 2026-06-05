use crate::error::{BioRsError, FastaReadError};
use crate::fasta_scan::{scan_fasta_reader, scan_fasta_str, FastaRecordSink};
use crate::sequence::{
    append_normalized_sequence, append_normalized_sequence_bytes,
    detect_sequence_kind_with_metadata, normalized_residues, summarize_validated_sequence_records,
    AlphabetPolicy, KindAwareSequenceValidationReport, KindAwareSequenceValidationSummary,
    SequenceKind, SequenceKindSelection, SequenceRecord, SymbolClass, ValidatedSequenceRecord,
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
    let mut hasher = StableInputHasher::new();
    let summary = if let Some(kind) = selection.explicit_kind() {
        let mut sink = ExplicitKindValidationSummarySink::new(kind);
        scan_fasta_reader(reader, &mut sink, |line| hasher.update(line))?;
        sink.finish()
    } else {
        let mut sink = KindAwareValidationSummarySink::new(selection);
        scan_fasta_reader(reader, &mut sink, |line| hasher.update(line))?;
        sink.finish()
    };
    Ok(ValidatedKindAwareFastaSummaryInput {
        input_hash: hasher.finalize(),
        summary,
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
        let sequence =
            take_non_empty_sequence(&mut self.current_sequence, &id, line, record_index)?;
        self.sequences
            .push(validate_record_for_selection(id, sequence, self.selection));
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
        let sequence =
            take_non_empty_sequence(&mut self.current_sequence, &id, line, record_index)?;
        let validated = validate_record_for_selection(id, sequence, self.selection);
        self.summary.add_record(&validated);
        Ok(())
    }
}

fn take_non_empty_sequence(
    current_sequence: &mut String,
    id: &str,
    line: usize,
    record_index: usize,
) -> Result<String, BioRsError> {
    if current_sequence.is_empty() {
        return Err(BioRsError::MissingSequence {
            id: id.to_string(),
            line,
            record_index,
        });
    }

    Ok(std::mem::take(current_sequence))
}

fn validate_record_for_selection(
    id: String,
    sequence: String,
    selection: SequenceKindSelection,
) -> ValidatedSequenceRecord {
    let (kind, auto_detection) = selection
        .explicit_kind()
        .map(|kind| (kind, None))
        .unwrap_or_else(|| {
            let detection = detect_sequence_kind_with_metadata(&sequence);
            (detection.selected_kind, Some(detection))
        });
    let record = SequenceRecord { id, sequence, kind };
    let mut validated = crate::sequence::validate_sequence_record(&record);
    validated.auto_detection = auto_detection;
    validated
}

struct ExplicitKindValidationSummarySink {
    kind: SequenceKind,
    policy: AlphabetPolicy,
    summary: KindAwareSequenceValidationSummary,
    current_length: usize,
    current_warning_count: usize,
    current_error_count: usize,
}

impl ExplicitKindValidationSummarySink {
    fn new(kind: SequenceKind) -> Self {
        Self {
            kind,
            policy: AlphabetPolicy::for_kind(kind),
            summary: KindAwareSequenceValidationSummary::default(),
            current_length: 0,
            current_warning_count: 0,
            current_error_count: 0,
        }
    }

    fn finish(self) -> KindAwareSequenceValidationSummary {
        self.summary
    }

    fn push_symbol(&mut self, symbol: char) {
        self.current_length += 1;
        match self.policy.classify(symbol) {
            SymbolClass::Standard => {}
            SymbolClass::Ambiguous => self.current_warning_count += 1,
            SymbolClass::Invalid => self.current_error_count += 1,
        }
    }

    fn push_symbol_byte(&mut self, symbol: u8) {
        if symbol.is_ascii_whitespace() {
            return;
        }

        self.current_length += 1;
        match self.policy.classify_byte(symbol) {
            SymbolClass::Standard => {}
            SymbolClass::Ambiguous => self.current_warning_count += 1,
            SymbolClass::Invalid => self.current_error_count += 1,
        }
    }
}

impl FastaRecordSink for ExplicitKindValidationSummarySink {
    fn push_sequence_line(&mut self, line: &str) {
        if line.is_ascii() {
            self.push_sequence_line_bytes(line.as_bytes());
            return;
        }

        for symbol in normalized_residues(line) {
            self.push_symbol(symbol);
        }
    }

    fn push_sequence_line_bytes(&mut self, line: &[u8]) {
        for &symbol in line {
            self.push_symbol_byte(symbol);
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

        self.summary.records += 1;
        if self.current_warning_count == 0 && self.current_error_count == 0 {
            self.summary.valid_records += 1;
        }
        self.summary.warning_count += self.current_warning_count;
        self.summary.error_count += self.current_error_count;
        self.summary.kind_counts.increment(self.kind);

        self.current_length = 0;
        self.current_warning_count = 0;
        self.current_error_count = 0;
        Ok(())
    }
}

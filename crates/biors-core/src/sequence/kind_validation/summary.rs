use crate::error::BioRsError;
use crate::fasta_scan::FastaRecordSink;
use crate::sequence::{
    normalized_residues, AlphabetPolicy, KindAwareSequenceValidationSummary, SequenceKind,
    SymbolClass,
};

pub(super) struct ExplicitKindValidationSummarySink {
    kind: SequenceKind,
    policy: AlphabetPolicy,
    summary: KindAwareSequenceValidationSummary,
    current_length: usize,
    current_warning_count: usize,
    current_error_count: usize,
}

impl ExplicitKindValidationSummarySink {
    pub(super) fn new(kind: SequenceKind) -> Self {
        Self {
            kind,
            policy: AlphabetPolicy::for_kind(kind),
            summary: KindAwareSequenceValidationSummary::default(),
            current_length: 0,
            current_warning_count: 0,
            current_error_count: 0,
        }
    }

    pub(super) fn finish(self) -> KindAwareSequenceValidationSummary {
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

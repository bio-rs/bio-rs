use super::{AlphabetPolicy, SequenceKind, SymbolClass};

/// Detect the most likely sequence kind from normalized biological symbols.
pub fn detect_sequence_kind(sequence: &str) -> SequenceKind {
    let mut evidence = KindEvidence::default();

    if sequence.is_ascii() {
        for byte in sequence.bytes() {
            if byte.is_ascii_whitespace() {
                continue;
            }
            evidence.observe_byte(byte.to_ascii_uppercase());
        }
    } else {
        for symbol in sequence
            .chars()
            .filter(|symbol| !symbol.is_whitespace())
            .map(|symbol| symbol.to_ascii_uppercase())
        {
            evidence.observe_char(symbol);
        }
    }

    evidence.detect()
}

#[derive(Default)]
struct KindEvidence {
    protein_errors: usize,
    dna_errors: usize,
    rna_errors: usize,
    has_u: bool,
    has_t: bool,
    has_protein_only_symbol: bool,
}

impl KindEvidence {
    fn observe_byte(&mut self, symbol: u8) {
        self.observe_classifications(
            symbol as char,
            AlphabetPolicy::for_kind(SequenceKind::Protein).classify_byte(symbol),
            AlphabetPolicy::for_kind(SequenceKind::Dna).classify_byte(symbol),
            AlphabetPolicy::for_kind(SequenceKind::Rna).classify_byte(symbol),
        );
    }

    fn observe_char(&mut self, symbol: char) {
        self.observe_classifications(
            symbol,
            AlphabetPolicy::for_kind(SequenceKind::Protein).classify(symbol),
            AlphabetPolicy::for_kind(SequenceKind::Dna).classify(symbol),
            AlphabetPolicy::for_kind(SequenceKind::Rna).classify(symbol),
        );
    }

    fn observe_classifications(
        &mut self,
        symbol: char,
        protein: SymbolClass,
        dna: SymbolClass,
        rna: SymbolClass,
    ) {
        if protein == SymbolClass::Invalid {
            self.protein_errors += 1;
        }
        if dna == SymbolClass::Invalid {
            self.dna_errors += 1;
        }
        if rna == SymbolClass::Invalid {
            self.rna_errors += 1;
        }

        self.has_u |= symbol == 'U';
        self.has_t |= symbol == 'T';
        self.has_protein_only_symbol |= protein != SymbolClass::Invalid
            && dna == SymbolClass::Invalid
            && rna == SymbolClass::Invalid;
    }

    fn detect(self) -> SequenceKind {
        let min_errors = self
            .protein_errors
            .min(self.dna_errors)
            .min(self.rna_errors);

        if self.has_protein_only_symbol && self.protein_errors == min_errors {
            return SequenceKind::Protein;
        }
        if self.has_u && !self.has_t && self.rna_errors == min_errors {
            return SequenceKind::Rna;
        }
        if self.dna_errors == min_errors {
            return SequenceKind::Dna;
        }
        if self.rna_errors == min_errors {
            return SequenceKind::Rna;
        }
        SequenceKind::Protein
    }
}

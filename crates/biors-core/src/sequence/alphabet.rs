use super::kind::SequenceKind;
use super::residue::{is_ambiguous_residue, is_protein_20_residue};

/// Classification of one normalized sequence symbol under an alphabet policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SymbolClass {
    /// Supported canonical residue or base.
    Standard,
    /// Supported ambiguous IUPAC code that should be surfaced as a warning.
    Ambiguous,
    /// Unsupported symbol for the selected sequence kind.
    Invalid,
}

/// Alphabet policy for one biological sequence kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AlphabetPolicy {
    kind: SequenceKind,
}

impl AlphabetPolicy {
    pub const fn for_kind(kind: SequenceKind) -> Self {
        Self { kind }
    }

    pub const fn kind(self) -> SequenceKind {
        self.kind
    }

    /// Return the stable policy name used in JSON reports.
    pub const fn name(self) -> &'static str {
        self.kind.alphabet_name()
    }

    /// Classify a symbol after ASCII uppercasing.
    pub fn classify(self, symbol: char) -> SymbolClass {
        if symbol.is_ascii() {
            return self.classify_byte(symbol as u8);
        }

        SymbolClass::Invalid
    }

    /// Classify an ASCII byte without allocating.
    pub fn classify_byte(self, symbol: u8) -> SymbolClass {
        let symbol = symbol.to_ascii_uppercase();
        match self.kind {
            SequenceKind::Protein => classify_protein_byte(symbol),
            SequenceKind::Dna => classify_nucleotide_byte(symbol, b'T'),
            SequenceKind::Rna => classify_nucleotide_byte(symbol, b'U'),
        }
    }
}

fn classify_protein_byte(symbol: u8) -> SymbolClass {
    let symbol = symbol as char;
    if is_protein_20_residue(symbol) {
        SymbolClass::Standard
    } else if is_ambiguous_residue(symbol) {
        SymbolClass::Ambiguous
    } else {
        SymbolClass::Invalid
    }
}

fn classify_nucleotide_byte(symbol: u8, thymine_or_uracil: u8) -> SymbolClass {
    match symbol {
        b'A' | b'C' | b'G' => SymbolClass::Standard,
        value if value == thymine_or_uracil => SymbolClass::Standard,
        b'R' | b'Y' | b'S' | b'W' | b'K' | b'M' | b'B' | b'D' | b'H' | b'V' | b'N' => {
            SymbolClass::Ambiguous
        }
        _ => SymbolClass::Invalid,
    }
}

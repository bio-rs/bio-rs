use serde::{Deserialize, Serialize};

use super::kind::SequenceKind;
use crate::Diagnostic;

mod serde_bytes_as_str {
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S: Serializer>(bytes: &[u8], serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(std::str::from_utf8(bytes).map_err(serde::ser::Error::custom)?)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Vec<u8>, D::Error> {
        let s = String::deserialize(deserializer)?;
        Ok(s.into_bytes())
    }
}

/// A named protein sequence parsed from FASTA or supplied by callers.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProteinSequence {
    /// FASTA identifier without the leading `>` and without the description suffix.
    pub id: String,
    /// Normalized sequence residues with whitespace removed and ASCII letters uppercased.
    #[serde(with = "serde_bytes_as_str")]
    pub sequence: Vec<u8>,
}

/// A residue-level validation warning or error with a one-based sequence position.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResidueIssue {
    /// The normalized residue that caused the warning or error.
    pub residue: char,
    /// One-based position in the normalized sequence.
    pub position: usize,
}

/// Validation result for one protein sequence.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValidatedSequence {
    /// Sequence identifier.
    pub id: String,
    /// Normalized sequence that was validated.
    pub sequence: String,
    /// Alphabet policy used for validation.
    pub alphabet: String,
    /// True when the sequence has no warnings and no errors.
    pub valid: bool,
    /// Ambiguous but recognized residues such as `X`, `B`, or `Z`.
    pub warnings: Vec<ResidueIssue>,
    /// Residues outside the supported protein alphabet and ambiguity policy.
    pub errors: Vec<ResidueIssue>,
}

/// Aggregate validation report for a batch of protein sequences.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SequenceValidationReport {
    /// Number of input records.
    pub records: usize,
    /// Number of records with no warnings and no errors.
    pub valid_records: usize,
    /// Total number of ambiguous-residue warnings.
    pub warning_count: usize,
    /// Total number of invalid-residue errors.
    pub error_count: usize,
    /// Per-record validation details.
    pub sequences: Vec<ValidatedSequence>,
}

/// A normalized biological sequence with an assigned sequence kind.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SequenceRecord {
    /// Sequence identifier.
    pub id: String,
    /// Normalized sequence symbols with whitespace removed and ASCII letters uppercased.
    pub sequence: String,
    /// Biological sequence kind used for validation.
    pub kind: SequenceKind,
}

impl SequenceRecord {
    /// Build a normalized sequence record for kind-aware validation.
    pub fn new(id: impl Into<String>, sequence: impl AsRef<str>, kind: SequenceKind) -> Self {
        Self {
            id: id.into(),
            sequence: super::normalize_sequence(sequence.as_ref()),
            kind,
        }
    }
}

/// Stable diagnostic code for kind-aware sequence validation issues.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SequenceValidationIssueCode {
    /// Supported ambiguous IUPAC symbol.
    AmbiguousSymbol,
    /// Unsupported symbol for the selected kind.
    InvalidSymbol,
}

impl SequenceValidationIssueCode {
    /// Return the stable diagnostic code string.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::AmbiguousSymbol => "sequence.ambiguous_symbol",
            Self::InvalidSymbol => "sequence.invalid_symbol",
        }
    }
}

/// Kind-aware sequence validation warning or error.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SequenceValidationIssue {
    /// Normalized symbol that caused the issue.
    pub symbol: char,
    /// One-based position in the normalized sequence.
    pub position: usize,
    /// Sequence kind used when classifying this symbol.
    pub kind: SequenceKind,
    /// Stable machine-readable issue code.
    pub code: SequenceValidationIssueCode,
    /// Human-readable issue message.
    pub message: String,
}

impl SequenceValidationIssue {
    /// Create an ambiguous-symbol warning.
    pub fn ambiguous(symbol: char, position: usize, kind: SequenceKind) -> Self {
        Self {
            symbol,
            position,
            kind,
            code: SequenceValidationIssueCode::AmbiguousSymbol,
            message: format!(
                "{} symbol '{symbol}' at position {position} is ambiguous IUPAC code",
                kind.display_name()
            ),
        }
    }

    /// Create an invalid-symbol error.
    pub fn invalid(symbol: char, position: usize, kind: SequenceKind) -> Self {
        Self {
            symbol,
            position,
            kind,
            code: SequenceValidationIssueCode::InvalidSymbol,
            message: format!(
                "{} symbol '{symbol}' at position {position} is not supported by {}",
                kind.display_name(),
                kind.alphabet_name()
            ),
        }
    }
}

impl Diagnostic for SequenceValidationIssue {
    fn code(&self) -> &'static str {
        self.code.as_str()
    }

    fn message(&self) -> String {
        self.message.clone()
    }
}

/// Validation result for one kind-aware biological sequence.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValidatedSequenceRecord {
    /// Sequence identifier.
    pub id: String,
    /// Normalized sequence that was validated.
    pub sequence: String,
    /// Biological sequence kind used for validation.
    pub kind: SequenceKind,
    /// Alphabet policy used for validation.
    pub alphabet: String,
    /// True when the sequence has no warnings and no errors.
    pub valid: bool,
    /// Ambiguous but recognized symbols for the selected kind.
    pub warnings: Vec<SequenceValidationIssue>,
    /// Symbols outside the selected kind's alphabet and ambiguity policy.
    pub errors: Vec<SequenceValidationIssue>,
}

/// Per-kind record counts for a mixed biological sequence validation batch.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SequenceKindCounts {
    /// Number of protein records.
    pub protein: usize,
    /// Number of DNA records.
    pub dna: usize,
    /// Number of RNA records.
    pub rna: usize,
}

impl SequenceKindCounts {
    /// Add one record to the count for `kind`.
    pub fn increment(&mut self, kind: SequenceKind) {
        match kind {
            SequenceKind::Protein => self.protein += 1,
            SequenceKind::Dna => self.dna += 1,
            SequenceKind::Rna => self.rna += 1,
        }
    }
}

/// Aggregate validation report for mixed biological sequence batches.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KindAwareSequenceValidationReport {
    /// Number of input records.
    pub records: usize,
    /// Number of records with no warnings and no errors.
    pub valid_records: usize,
    /// Total number of ambiguous-symbol warnings.
    pub warning_count: usize,
    /// Total number of invalid-symbol errors.
    pub error_count: usize,
    /// Per-kind record counts.
    pub kind_counts: SequenceKindCounts,
    /// Per-record validation details.
    pub sequences: Vec<ValidatedSequenceRecord>,
}

/// Aggregate validation summary for mixed biological sequence batches without per-record payloads.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KindAwareSequenceValidationSummary {
    /// Number of input records.
    pub records: usize,
    /// Number of records with no warnings and no errors.
    pub valid_records: usize,
    /// Total number of ambiguous-symbol warnings.
    pub warning_count: usize,
    /// Total number of invalid-symbol errors.
    pub error_count: usize,
    /// Per-kind record counts.
    pub kind_counts: SequenceKindCounts,
}

impl KindAwareSequenceValidationSummary {
    /// Add one validated record to the summary.
    pub fn add_record(&mut self, record: &ValidatedSequenceRecord) {
        self.records += 1;
        if record.valid {
            self.valid_records += 1;
        }
        self.warning_count += record.warnings.len();
        self.error_count += record.errors.len();
        self.kind_counts.increment(record.kind);
    }
}

use serde::{Deserialize, Serialize};

/// A named protein sequence parsed from FASTA or supplied by callers.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProteinSequence {
    /// FASTA identifier without the leading `>` and without the description suffix.
    pub id: String,
    /// Normalized sequence residues with whitespace removed and ASCII letters uppercased.
    pub sequence: String,
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

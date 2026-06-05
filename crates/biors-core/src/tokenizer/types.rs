use crate::sequence::ResidueIssue;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Tokenized representation of one protein sequence.
pub struct TokenizedProtein {
    /// Sequence identifier.
    pub id: String,
    /// Number of emitted token IDs.
    pub length: usize,
    /// Alphabet policy used for tokenization.
    pub alphabet: String,
    /// True when no warnings or errors were emitted.
    pub valid: bool,
    /// Token IDs.
    pub tokens: Vec<u8>,
    /// Ambiguous-residue warnings.
    pub warnings: Vec<ResidueIssue>,
    /// Invalid-residue errors.
    pub errors: Vec<ResidueIssue>,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Aggregate summary for a tokenized protein batch.
pub struct ProteinBatchSummary {
    /// Number of records.
    pub records: usize,
    /// Sum of tokenized sequence lengths.
    pub total_length: usize,
    /// Number of records without warnings or errors.
    pub valid_records: usize,
    /// Total warning count.
    pub warning_count: usize,
    /// Total error count.
    pub error_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Tokenized FASTA reader output with a stable raw input hash.
pub struct TokenizedFastaInput {
    /// Stable hash of the raw input bytes.
    pub input_hash: String,
    /// Tokenized records.
    pub records: Vec<TokenizedProtein>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// FASTA reader summary output with a stable raw input hash.
pub struct SummarizedFastaInput {
    /// Stable hash of the raw input bytes.
    pub input_hash: String,
    /// Aggregate tokenization summary.
    pub summary: ProteinBatchSummary,
}

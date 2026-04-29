//! Protein sequence types, normalization, residue policy, and validation reports.

mod normalization;
mod report;
mod residue;
mod types;
mod validation;

pub use normalization::normalize_sequence;
pub(crate) use normalization::{
    append_normalized_sequence, append_normalized_sequence_bytes, normalized_residues,
};
pub use report::summarize_validated_sequences;
pub(crate) use residue::{is_ambiguous_residue, is_protein_20_residue};
pub use residue::{AMBIGUOUS_RESIDUES, PROTEIN_20, PROTEIN_20_RESIDUES};
pub use types::{ProteinSequence, ResidueIssue, SequenceValidationReport, ValidatedSequence};
pub use validation::validate_protein_sequence;

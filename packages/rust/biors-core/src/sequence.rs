//! Biological sequence types, normalization, alphabet policies, and validation reports.

mod alphabet;
mod detection;
mod kind;
mod normalization;
mod report;
mod residue;
mod types;
mod validation;

pub use alphabet::{AlphabetPolicy, SymbolClass};
pub use detection::detect_sequence_kind;
pub use kind::{SequenceKind, SequenceKindSelection};
pub use normalization::normalize_sequence;
pub(crate) use normalization::{
    append_normalized_sequence, append_normalized_sequence_bytes,
    append_normalized_sequence_bytes_to_vec, append_normalized_sequence_to_vec,
    normalized_residues,
};
pub use report::{summarize_validated_sequence_records, summarize_validated_sequences};
pub(crate) use residue::{
    is_ambiguous_residue, is_ambiguous_residue_byte, is_protein_20_residue,
    is_protein_20_residue_byte,
};
pub use residue::{AMBIGUOUS_RESIDUES, PROTEIN_20, PROTEIN_20_RESIDUES};
pub use types::{
    KindAwareSequenceValidationReport, ProteinSequence, ResidueIssue, SequenceKindCounts,
    SequenceRecord, SequenceValidationIssue, SequenceValidationIssueCode, SequenceValidationReport,
    ValidatedSequence, ValidatedSequenceRecord,
};
pub(crate) use validation::validate_protein_sequence_owned;
pub use validation::{validate_protein_sequence, validate_sequence_record};

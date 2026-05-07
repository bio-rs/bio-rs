//! Core Rust APIs for bio-rs biological sequence validation, protein tokenization, model input
//! construction, package manifests, and package fixture verification.
//!
//! FASTA reader paths prefer an ASCII byte-level scanner for common biological
//! FASTA input. Non-ASCII sequence lines fall back to UTF-8 validation so public
//! Unicode behavior remains explicit and test-covered.

pub mod error;
pub mod fasta;
mod fasta_scan;
pub mod model_input;
pub mod package;
pub mod sequence;
pub mod tokenizer;
pub mod verification;
pub mod versioning;
pub mod workflow;

pub use error::{BioRsError, FastaReadError};
pub use fasta::{parse_fasta_records, parse_fasta_records_reader, validate_fasta_reader};
pub use model_input::ModelInput;
pub use sequence::{ProteinSequence, SequenceKind};
pub use tokenizer::{
    tokenize_fasta_records_reader, tokenize_protein, ProteinTokenizer, ProteinTokenizerConfig,
};
pub use workflow::{prepare_protein_model_input_workflow, SequenceWorkflowOutput};

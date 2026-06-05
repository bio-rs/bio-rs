//! Core Rust APIs for bio-rs biological sequence validation, profile-aware
//! tokenization, model-input construction, package manifests, service
//! contracts, runtime planning, and package fixture verification.
//!
//! FASTA reader paths prefer an ASCII byte-level scanner for common biological
//! FASTA input. Non-ASCII sequence lines fall back to UTF-8 validation so public
//! Unicode behavior remains explicit and test-covered.

pub mod conversion;
pub mod error;
pub mod fasta;
mod fasta_scan;
pub mod formats;
pub mod hash;
pub mod model_input;
pub mod molecule;
pub mod package;
pub mod runtime;
pub mod sequence;
pub mod service;
pub mod structure;
pub mod templates;
pub mod tokenizer;
pub mod verification;
pub mod versioning;
pub mod workflow;

pub use error::{BioRsError, FastaReadError};
pub use fasta::{parse_fasta_records, parse_fasta_records_reader, validate_fasta_reader};
pub use formats::{parse_fastq_records, parse_fastq_records_reader, validate_fastq_reader};
pub use model_input::ModelInput;
pub use molecule::{
    parse_smiles_records, parse_smiles_records_reader, validate_smiles_reader, MoleculeRecord,
};
pub use sequence::{BiologicalSequence, FastaSequence, ProteinSequence, SequenceKind};
pub use structure::{
    parse_pdb_record, parse_pdb_record_reader, validate_pdb_reader, StructureRecord,
};
pub use templates::{find_task_template, task_template_ids, task_templates, TaskTemplate};
pub use tokenizer::{
    tokenize_fasta_records_reader, tokenize_protein, ProteinTokenizer, ProteinTokenizerConfig,
    TokenizedSequence, TokenizerConfig, TokenizerProfile,
};
pub use workflow::{prepare_protein_model_input_workflow, SequenceWorkflowOutput};

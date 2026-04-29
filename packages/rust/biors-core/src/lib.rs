//! Core Rust APIs for bio-rs protein FASTA validation, tokenization, model input
//! construction, package manifests, and package fixture verification.
//!
//! FASTA reader paths prefer an ASCII byte-level scanner for the common protein
//! FASTA case. Non-ASCII sequence lines fall back to UTF-8 validation so public
//! Unicode behavior remains explicit and test-covered.

/// Structured error types used by parsing and reader APIs.
pub mod error;
/// FASTA parsing and validation entry points.
pub mod fasta;
mod fasta_scan;
/// Model-ready tensor input builders.
pub mod model_input;
/// Portable package manifest contracts and validation helpers.
pub mod package;
/// Protein sequence normalization and validation helpers.
pub mod sequence;
/// Protein tokenization and vocabulary helpers.
pub mod tokenizer;
/// Package fixture verification and stable hashing helpers.
pub mod verification;

pub use error::{BioRsError, ErrorLocation, FastaReadError};
pub use fasta::{
    parse_fasta_records, parse_fasta_records_reader, validate_fasta_input, validate_fasta_reader,
    validate_fasta_reader_with_hash, ParsedFastaInput, ValidatedFastaInput,
};
#[allow(deprecated)]
pub use model_input::build_model_inputs;
pub use model_input::{
    build_model_inputs_checked, build_model_inputs_unchecked, validate_model_input_policy,
    ModelInput, ModelInputBuildError, ModelInputPolicy, ModelInputRecord, PaddingPolicy,
};
pub use package::{
    inspect_package_manifest, is_sha256_checksum, plan_runtime_bridge, read_package_file,
    resolve_package_asset_path, resolve_package_path, sha256_digest, validate_package_manifest,
    validate_package_manifest_artifacts, validate_package_relative_path, DataShape, DataType,
    ModelArtifact, ModelFormat, PackageFixture, PackageManifest, PackageManifestSummary,
    PackageValidationIssue, PackageValidationIssueCode, PackageValidationReport, PipelineStep,
    RuntimeBackend, RuntimeBridgeReport, RuntimeTarget, RuntimeTargetPlatform, SchemaVersion,
    TokenAsset,
};
pub use sequence::{
    normalize_sequence, validate_protein_sequence, ProteinSequence, ResidueIssue,
    SequenceValidationReport, ValidatedSequence,
};
pub use tokenizer::{
    load_protein_20_vocab, load_vocab_json, protein_20_unknown_token_policy,
    protein_20_vocab_tokens, protein_20_vocabulary, summarize_fasta_records_reader,
    summarize_tokenized_proteins, tokenize_fasta_records, tokenize_fasta_records_reader,
    tokenize_protein, ProteinBatchSummary, ProteinTokenizer, SummarizedFastaInput,
    TokenizedFastaInput, TokenizedProtein, Tokenizer, UnknownTokenPolicy, VocabToken, Vocabulary,
    PROTEIN_20_UNKNOWN_TOKEN_ID,
};
pub use verification::{
    stable_input_hash, verify_package_outputs, verify_package_outputs_with_observation_base,
    ContentMismatchDiff, FirstDifference, FixtureObservation, FixtureVerificationResult,
    PackageVerificationReport, StableInputHasher, VerificationIssueCode, VerificationStatus,
};

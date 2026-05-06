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

pub use error::{BioRsError, Diagnostic, ErrorLocation, FastaReadError};
pub use fasta::{
    parse_fasta_records, parse_fasta_records_reader, validate_fasta_input,
    validate_fasta_input_with_kind, validate_fasta_reader, validate_fasta_reader_with_hash,
    validate_fasta_reader_with_kind, validate_fasta_reader_with_kind_and_hash, ParsedFastaInput,
    ValidatedFastaInput, ValidatedKindAwareFastaInput,
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
    ModelArtifact, ModelFormat, PackageArtifactError, PackageFixture, PackageManifest,
    PackageManifestSummary, PackageValidationIssue, PackageValidationIssueCode,
    PackageValidationReport, PipelineStep, RuntimeBackend, RuntimeBridgeReport, RuntimeTarget,
    RuntimeTargetPlatform, SchemaVersion, TokenAsset,
};
pub use sequence::{
    detect_sequence_kind, normalize_sequence, validate_protein_sequence, validate_sequence_record,
    AlphabetPolicy, KindAwareSequenceValidationReport, ProteinSequence, ResidueIssue, SequenceKind,
    SequenceKindCounts, SequenceKindSelection, SequenceRecord, SequenceValidationIssue,
    SequenceValidationIssueCode, SequenceValidationReport, SymbolClass, ValidatedSequence,
    ValidatedSequenceRecord,
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

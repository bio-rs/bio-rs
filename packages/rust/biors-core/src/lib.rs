pub mod error;
pub mod fasta;
pub mod model_input;
pub mod package;
pub mod sequence;
pub mod tokenizer;
pub mod verification;

pub use error::{BioRsError, ErrorLocation};
pub use fasta::{parse_fasta_records, validate_fasta_input};
pub use model_input::{
    build_model_inputs, build_model_inputs_checked, validate_model_input_policy, ModelInput,
    ModelInputBuildError, ModelInputPolicy, ModelInputRecord, PaddingPolicy,
};
pub use package::{
    inspect_package_manifest, is_sha256_checksum, plan_runtime_bridge, read_package_file,
    resolve_package_asset_path, resolve_package_path, sha256_digest, validate_package_manifest,
    validate_package_manifest_artifacts, validate_package_relative_path, DataShape, DataType,
    ModelArtifact, ModelFormat, PackageFixture, PackageManifest, PackageManifestSummary,
    PackageValidationReport, PipelineStep, RuntimeBackend, RuntimeBridgeReport, RuntimeTarget,
    RuntimeTargetPlatform, SchemaVersion, TokenAsset,
};
pub use sequence::{
    normalize_sequence, validate_protein_sequence, ProteinSequence, ResidueIssue,
    SequenceValidationReport, ValidatedSequence,
};
pub use tokenizer::{
    load_protein_20_vocab, load_vocab_json, protein_20_unknown_token_policy,
    summarize_tokenized_proteins, tokenize_fasta_records, tokenize_protein, ProteinBatchSummary,
    ProteinTokenizer, TokenizedProtein, Tokenizer, UnknownTokenPolicy, VocabToken, Vocabulary,
    PROTEIN_20_UNKNOWN_TOKEN_ID,
};
pub use verification::{
    stable_input_hash, verify_package_outputs, verify_package_outputs_with_observation_base,
    FixtureObservation, FixtureVerificationResult, PackageVerificationReport, VerificationStatus,
};

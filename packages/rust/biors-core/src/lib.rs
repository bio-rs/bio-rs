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
    build_model_inputs, ModelInput, ModelInputPolicy, ModelInputRecord, PaddingPolicy,
};
pub use package::{
    inspect_package_manifest, plan_runtime_bridge, validate_package_manifest, DataShape,
    ModelArtifact, PackageFixture, PackageManifest, PackageManifestSummary,
    PackageValidationReport, PipelineStep, RuntimeBridgeReport, RuntimeTarget, TokenAsset,
};
pub use sequence::{
    normalize_sequence, validate_protein_sequence, ProteinSequence, ResidueIssue,
    SequenceValidationReport, ValidatedSequence,
};
pub use tokenizer::{
    load_protein_20_vocab, protein_20_unknown_token_policy, summarize_tokenized_proteins,
    tokenize_fasta_records, tokenize_protein, ProteinBatchSummary, ProteinTokenizer,
    TokenizedProtein, Tokenizer, UnknownTokenPolicy, VocabToken, Vocabulary,
};
pub use verification::{
    stable_input_hash, verify_package_outputs, FixtureObservation, FixtureVerificationResult,
    PackageVerificationReport, VerificationStatus,
};

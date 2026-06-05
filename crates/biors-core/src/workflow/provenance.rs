use super::{
    SequenceWorkflowHashes, SequenceWorkflowInvocation, SequenceWorkflowProvenance,
    WorkflowTokenizerMetadata, NORMALIZATION_POLICY, PROTEIN_WORKFLOW_NAME, SEQUENCE_WORKFLOW_NAME,
};
use crate::model_input::ModelInputPolicy;
use crate::sequence::SequenceKind;
use crate::tokenizer::{
    inspect_protein_tokenizer_config, protein_tokenizer_config_for_profile, ProteinTokenizerProfile,
};

pub(super) fn workflow_provenance(
    input_hash: String,
    policy: ModelInputPolicy,
    invocation: SequenceWorkflowInvocation,
    hashes: SequenceWorkflowHashes,
    profile: ProteinTokenizerProfile,
) -> SequenceWorkflowProvenance {
    let vocab =
        inspect_protein_tokenizer_config(&protein_tokenizer_config_for_profile(profile)).vocabulary;
    SequenceWorkflowProvenance {
        biors_core_version: env!("CARGO_PKG_VERSION").to_string(),
        invocation,
        input_hash,
        normalization: NORMALIZATION_POLICY.to_string(),
        validation_alphabet: vocab.name.clone(),
        tokenizer: WorkflowTokenizerMetadata {
            name: vocab.name.clone(),
            vocab_size: vocab.tokens.len(),
            unknown_token_id: vocab.unknown_token_id,
            unknown_token_policy: vocab.unknown_token_policy.clone(),
        },
        model_input_policy: policy,
        hashes,
    }
}

pub(super) fn workflow_name(profile: ProteinTokenizerProfile) -> &'static str {
    match profile.sequence_kind() {
        SequenceKind::Protein => PROTEIN_WORKFLOW_NAME,
        SequenceKind::Dna | SequenceKind::Rna => SEQUENCE_WORKFLOW_NAME,
    }
}

use super::{
    SequenceWorkflowReadinessIssue, TokenizationWorkflowOutput, WorkflowTokenizerMetadata,
    NORMALIZATION_POLICY,
};
use crate::model_input::{ModelInput, ModelInputPolicy};
use crate::sequence::SequenceValidationReport;
use crate::tokenizer::{
    inspect_protein_tokenizer_config, protein_tokenizer_config_for_profile, ProteinTokenizerProfile,
};
use serde::{Deserialize, Serialize};

pub(super) const CORE_WORKFLOW_COMMAND: &str = "biors-core prepare_protein_model_input_workflow";

/// Command or API invocation captured in workflow provenance.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SequenceWorkflowInvocation {
    pub command: String,
    pub arguments: Vec<String>,
}

/// Reproducibility hashes included in workflow provenance.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SequenceWorkflowHashes {
    pub vocabulary_sha256: String,
    pub output_data_sha256: String,
}

pub(super) struct WorkflowHashInput<'a> {
    pub(super) workflow: &'static str,
    pub(super) profile: ProteinTokenizerProfile,
    pub(super) input_hash: &'a str,
    pub(super) policy: &'a ModelInputPolicy,
    pub(super) invocation: &'a SequenceWorkflowInvocation,
    pub(super) validation: &'a SequenceValidationReport,
    pub(super) tokenization: &'a TokenizationWorkflowOutput,
    pub(super) model_input: &'a Option<ModelInput>,
    pub(super) readiness_issues: &'a [SequenceWorkflowReadinessIssue],
}

pub(super) fn workflow_hashes(input: WorkflowHashInput<'_>) -> SequenceWorkflowHashes {
    let vocab =
        inspect_protein_tokenizer_config(&protein_tokenizer_config_for_profile(input.profile))
            .vocabulary;
    SequenceWorkflowHashes {
        vocabulary_sha256: json_sha256(&vocab),
        output_data_sha256: json_sha256(&WorkflowHashPayload {
            workflow: input.workflow,
            model_ready: input.readiness_issues.is_empty(),
            biors_core_version: env!("CARGO_PKG_VERSION"),
            invocation: input.invocation,
            input_hash: input.input_hash,
            normalization: NORMALIZATION_POLICY,
            validation_alphabet: vocab.name.as_str(),
            tokenizer: WorkflowTokenizerMetadata {
                name: vocab.name.clone(),
                vocab_size: vocab.tokens.len(),
                unknown_token_id: vocab.unknown_token_id,
                unknown_token_policy: vocab.unknown_token_policy.clone(),
            },
            model_input_policy: input.policy,
            validation: input.validation,
            tokenization: input.tokenization,
            model_input: input.model_input,
            readiness_issues: input.readiness_issues,
        }),
    }
}

#[derive(Serialize)]
struct WorkflowHashPayload<'a> {
    workflow: &'static str,
    model_ready: bool,
    biors_core_version: &'static str,
    invocation: &'a SequenceWorkflowInvocation,
    input_hash: &'a str,
    normalization: &'static str,
    validation_alphabet: &'a str,
    tokenizer: WorkflowTokenizerMetadata,
    model_input_policy: &'a ModelInputPolicy,
    validation: &'a SequenceValidationReport,
    tokenization: &'a TokenizationWorkflowOutput,
    model_input: &'a Option<ModelInput>,
    readiness_issues: &'a [SequenceWorkflowReadinessIssue],
}

fn json_sha256<T: Serialize>(value: &T) -> String {
    match serde_json::to_vec(value) {
        Ok(bytes) => crate::hash::sha256_canonical_json_digest(&bytes),
        Err(error) => crate::hash::sha256_bytes_digest(error.to_string().as_bytes()),
    }
}

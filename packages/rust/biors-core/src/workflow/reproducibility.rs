use super::{
    SequenceWorkflowReadinessIssue, TokenizationWorkflowOutput, WorkflowTokenizerMetadata,
    NORMALIZATION_POLICY, WORKFLOW_NAME,
};
use crate::{load_protein_20_vocab, ModelInput, ModelInputPolicy, SequenceValidationReport};
use serde::{Deserialize, Serialize};

pub(super) const CORE_WORKFLOW_COMMAND: &str = "biors-core prepare_protein_model_input_workflow";

/// Command or API invocation captured in workflow provenance.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SequenceWorkflowInvocation {
    /// Stable command/API name.
    pub command: String,
    /// Resolved arguments that affect workflow output.
    pub arguments: Vec<String>,
}

/// Reproducibility hashes included in workflow provenance.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SequenceWorkflowHashes {
    /// SHA-256 digest of the tokenizer vocabulary contract.
    pub vocabulary_sha256: String,
    /// SHA-256 digest of workflow output content excluding hash values.
    pub output_data_sha256: String,
}

pub(super) fn workflow_hashes(
    input_hash: &str,
    policy: &ModelInputPolicy,
    invocation: &SequenceWorkflowInvocation,
    validation: &SequenceValidationReport,
    tokenization: &TokenizationWorkflowOutput,
    model_input: &Option<ModelInput>,
    readiness_issues: &[SequenceWorkflowReadinessIssue],
) -> SequenceWorkflowHashes {
    let vocab = load_protein_20_vocab();
    SequenceWorkflowHashes {
        vocabulary_sha256: json_sha256(&vocab),
        output_data_sha256: json_sha256(&WorkflowHashPayload {
            workflow: WORKFLOW_NAME,
            model_ready: readiness_issues.is_empty(),
            biors_core_version: env!("CARGO_PKG_VERSION"),
            invocation,
            input_hash,
            normalization: NORMALIZATION_POLICY,
            validation_alphabet: vocab.name.as_str(),
            tokenizer: WorkflowTokenizerMetadata {
                name: vocab.name.clone(),
                vocab_size: vocab.tokens.len(),
                unknown_token_id: vocab.unknown_token_id,
                unknown_token_policy: vocab.unknown_token_policy.clone(),
            },
            model_input_policy: policy,
            validation,
            tokenization,
            model_input,
            readiness_issues,
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
        Ok(bytes) => crate::sha256_digest(&bytes),
        Err(error) => crate::sha256_digest(error.to_string().as_bytes()),
    }
}

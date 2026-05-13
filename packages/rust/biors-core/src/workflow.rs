use crate::model_input::{
    build_model_inputs_checked, validate_model_input_policy, ModelInput, ModelInputBuildError,
    ModelInputPolicy,
};
use crate::sequence::{validate_protein_sequence, ProteinSequence, SequenceValidationReport};
use crate::tokenizer::{
    load_protein_20_vocab, summarize_tokenized_proteins, ProteinBatchSummary, TokenizedProtein,
    UnknownTokenPolicy,
};
use serde::{Deserialize, Serialize};

mod reproducibility;
use reproducibility::{workflow_hashes, CORE_WORKFLOW_COMMAND};
pub use reproducibility::{SequenceWorkflowHashes, SequenceWorkflowInvocation};

const WORKFLOW_NAME: &str = "protein_model_input.v0";
const NORMALIZATION_POLICY: &str = "strip_ascii_whitespace_uppercase";
const READINESS_ISSUE_CODE: &str = "sequence.not_model_ready";

/// End-to-end protein sequence preparation output for model-input workflows.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SequenceWorkflowOutput {
    pub workflow: String,
    pub model_ready: bool,
    pub provenance: SequenceWorkflowProvenance,
    pub validation: SequenceValidationReport,
    pub tokenization: TokenizationWorkflowOutput,
    pub model_input: Option<ModelInput>,
    pub readiness_issues: Vec<SequenceWorkflowReadinessIssue>,
}

/// Reproducibility metadata for the workflow output.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SequenceWorkflowProvenance {
    pub biors_core_version: String,
    pub invocation: SequenceWorkflowInvocation,
    pub input_hash: String,
    pub normalization: String,
    pub validation_alphabet: String,
    pub tokenizer: WorkflowTokenizerMetadata,
    pub model_input_policy: ModelInputPolicy,
    pub hashes: SequenceWorkflowHashes,
}

/// Tokenizer metadata included in workflow provenance.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkflowTokenizerMetadata {
    pub name: String,
    pub vocab_size: usize,
    pub unknown_token_id: u8,
    pub unknown_token_policy: UnknownTokenPolicy,
}

/// Tokenization section of a workflow output.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TokenizationWorkflowOutput {
    pub summary: ProteinBatchSummary,
    pub records: Vec<TokenizedProtein>,
}

/// Reason a record could not be converted into model-ready input.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SequenceWorkflowReadinessIssue {
    pub code: String,
    pub id: String,
    pub warning_count: usize,
    pub error_count: usize,
    pub message: String,
}

/// Build the stable protein validation -> tokenization -> model-input workflow.
pub fn prepare_protein_model_input_workflow(
    input_hash: String,
    records: &[ProteinSequence],
    policy: ModelInputPolicy,
) -> Result<SequenceWorkflowOutput, ModelInputBuildError> {
    prepare_protein_model_input_workflow_with_invocation(
        input_hash,
        records,
        policy,
        SequenceWorkflowInvocation {
            command: CORE_WORKFLOW_COMMAND.to_string(),
            arguments: Vec::new(),
        },
    )
}

/// Build the stable workflow and capture a command/API invocation in provenance.
pub fn prepare_protein_model_input_workflow_with_invocation(
    input_hash: String,
    records: &[ProteinSequence],
    policy: ModelInputPolicy,
    invocation: SequenceWorkflowInvocation,
) -> Result<SequenceWorkflowOutput, ModelInputBuildError> {
    validate_model_input_policy(&policy)?;

    let validation = crate::sequence::summarize_validated_sequences(
        records.iter().map(validate_protein_sequence).collect(),
    );
    let tokenized: Vec<_> = records
        .iter()
        .map(crate::tokenizer::tokenize_protein)
        .collect();
    let readiness_issues = readiness_issues(&tokenized);
    let model_input = if readiness_issues.is_empty() {
        Some(build_model_inputs_checked(&tokenized, policy.clone())?)
    } else {
        None
    };
    let tokenization = TokenizationWorkflowOutput {
        summary: summarize_tokenized_proteins(&tokenized),
        records: tokenized,
    };
    let model_ready = readiness_issues.is_empty();
    let hashes = workflow_hashes(
        input_hash.as_str(),
        &policy,
        &invocation,
        &validation,
        &tokenization,
        &model_input,
        &readiness_issues,
    );

    Ok(SequenceWorkflowOutput {
        workflow: WORKFLOW_NAME.to_string(),
        model_ready,
        provenance: provenance(input_hash, policy, invocation, hashes),
        validation,
        tokenization,
        model_input,
        readiness_issues,
    })
}

fn provenance(
    input_hash: String,
    policy: ModelInputPolicy,
    invocation: SequenceWorkflowInvocation,
    hashes: SequenceWorkflowHashes,
) -> SequenceWorkflowProvenance {
    let vocab = load_protein_20_vocab();
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

fn readiness_issues(tokenized: &[TokenizedProtein]) -> Vec<SequenceWorkflowReadinessIssue> {
    tokenized
        .iter()
        .filter(|record| !record.warnings.is_empty() || !record.errors.is_empty())
        .map(|record| {
            let warning_count = record.warnings.len();
            let error_count = record.errors.len();
            SequenceWorkflowReadinessIssue {
                code: READINESS_ISSUE_CODE.to_string(),
                id: record.id.clone(),
                warning_count,
                error_count,
                message: format!(
                    "sequence '{}' is not model-ready: {warning_count} warnings and {error_count} errors must be resolved before model-input generation",
                    record.id
                ),
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model_input::PaddingPolicy;

    #[test]
    fn workflow_preserves_validation_tokenization_and_model_input() {
        let output = prepare_protein_model_input_workflow(
            "fnv1a64:0000000000000000".to_string(),
            &[ProteinSequence {
                id: "seq1".to_string(),
                sequence: b"ACDE".to_vec(),
            }],
            ModelInputPolicy {
                max_length: 6,
                pad_token_id: 0,
                padding: PaddingPolicy::FixedLength,
            },
        )
        .expect("workflow output");

        assert!(output.model_ready);
        assert_eq!(output.validation.records, 1);
        assert_eq!(output.validation.sequences[0].sequence, "ACDE");
        assert_eq!(output.tokenization.records[0].tokens, vec![0, 1, 2, 3]);
        assert_eq!(
            output.model_input.expect("model input").records[0].input_ids,
            vec![0, 1, 2, 3, 0, 0]
        );
        assert_eq!(output.provenance.invocation.command, CORE_WORKFLOW_COMMAND);
        assert!(output
            .provenance
            .hashes
            .vocabulary_sha256
            .starts_with("sha256:"));
        assert!(output
            .provenance
            .hashes
            .output_data_sha256
            .starts_with("sha256:"));
        assert!(output.readiness_issues.is_empty());
    }

    #[test]
    fn workflow_keeps_reports_when_model_input_is_not_ready() {
        let output = prepare_protein_model_input_workflow(
            "fnv1a64:0000000000000000".to_string(),
            &[ProteinSequence {
                id: "seq1".to_string(),
                sequence: b"AX*".to_vec(),
            }],
            ModelInputPolicy {
                max_length: 6,
                pad_token_id: 0,
                padding: PaddingPolicy::FixedLength,
            },
        )
        .expect("workflow output");

        assert!(!output.model_ready);
        assert!(output.model_input.is_none());
        assert_eq!(output.validation.warning_count, 1);
        assert_eq!(output.validation.error_count, 1);
        assert_eq!(output.readiness_issues[0].code, READINESS_ISSUE_CODE);
    }
}

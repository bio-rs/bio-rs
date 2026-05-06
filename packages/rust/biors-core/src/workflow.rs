use crate::{
    build_model_inputs_checked, load_protein_20_vocab, summarize_tokenized_proteins,
    validate_model_input_policy, validate_protein_sequence, ModelInput, ModelInputBuildError,
    ModelInputPolicy, ProteinBatchSummary, ProteinSequence, SequenceValidationReport,
    TokenizedProtein, UnknownTokenPolicy,
};
use serde::{Deserialize, Serialize};

const WORKFLOW_NAME: &str = "protein_model_input.v0";
const NORMALIZATION_POLICY: &str = "strip_ascii_whitespace_uppercase";
const READINESS_ISSUE_CODE: &str = "sequence.not_model_ready";

/// End-to-end protein sequence preparation output for model-input workflows.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SequenceWorkflowOutput {
    /// Stable workflow contract name.
    pub workflow: String,
    /// True when all records were validated, tokenized, and converted to model input.
    pub model_ready: bool,
    /// Deterministic metadata needed to reproduce preprocessing.
    pub provenance: SequenceWorkflowProvenance,
    /// Protein validation report over normalized records.
    pub validation: SequenceValidationReport,
    /// Deterministic tokenization output and aggregate summary.
    pub tokenization: TokenizationWorkflowOutput,
    /// Model-ready tensors when every record is valid.
    pub model_input: Option<ModelInput>,
    /// Per-record reasons that prevented model-input generation.
    pub readiness_issues: Vec<SequenceWorkflowReadinessIssue>,
}

/// Reproducibility metadata for the workflow output.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SequenceWorkflowProvenance {
    /// bio-rs core version used for preprocessing.
    pub biors_core_version: String,
    /// Stable hash of the exact input bytes.
    pub input_hash: String,
    /// Normalization policy applied before validation and tokenization.
    pub normalization: String,
    /// Validation alphabet used by this workflow.
    pub validation_alphabet: String,
    /// Tokenizer metadata used for deterministic token IDs.
    pub tokenizer: WorkflowTokenizerMetadata,
    /// Model-input policy used to build arrays.
    pub model_input_policy: ModelInputPolicy,
}

/// Tokenizer metadata included in workflow provenance.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkflowTokenizerMetadata {
    /// Vocabulary/profile name.
    pub name: String,
    /// Number of ordinary vocabulary tokens.
    pub vocab_size: usize,
    /// Token ID emitted for unresolved residues.
    pub unknown_token_id: u8,
    /// Policy used when ambiguous or unsupported residues are encountered.
    pub unknown_token_policy: UnknownTokenPolicy,
}

/// Tokenization section of a workflow output.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TokenizationWorkflowOutput {
    /// Aggregate tokenization summary.
    pub summary: ProteinBatchSummary,
    /// Per-record tokenization details.
    pub records: Vec<TokenizedProtein>,
}

/// Reason a record could not be converted into model-ready input.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SequenceWorkflowReadinessIssue {
    /// Stable machine-readable readiness issue code.
    pub code: String,
    /// Sequence identifier.
    pub id: String,
    /// Number of validation/tokenization warnings.
    pub warning_count: usize,
    /// Number of validation/tokenization errors.
    pub error_count: usize,
    /// Human-readable readiness message.
    pub message: String,
}

/// Build the stable protein validation -> tokenization -> model-input workflow.
pub fn prepare_protein_model_input_workflow(
    input_hash: String,
    records: &[ProteinSequence],
    policy: ModelInputPolicy,
) -> Result<SequenceWorkflowOutput, ModelInputBuildError> {
    validate_model_input_policy(&policy)?;

    let validation = crate::sequence::summarize_validated_sequences(
        records.iter().map(validate_protein_sequence).collect(),
    );
    let tokenized: Vec<_> = records.iter().map(crate::tokenize_protein).collect();
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

    Ok(SequenceWorkflowOutput {
        workflow: WORKFLOW_NAME.to_string(),
        model_ready: readiness_issues.is_empty(),
        provenance: provenance(input_hash, policy),
        validation,
        tokenization,
        model_input,
        readiness_issues,
    })
}

fn provenance(input_hash: String, policy: ModelInputPolicy) -> SequenceWorkflowProvenance {
    let vocab = load_protein_20_vocab();
    SequenceWorkflowProvenance {
        biors_core_version: env!("CARGO_PKG_VERSION").to_string(),
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
    use crate::PaddingPolicy;

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

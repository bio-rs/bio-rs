use crate::model_input::{
    build_model_inputs_checked, validate_model_input_policy, ModelInput, ModelInputBuildError,
    ModelInputPolicy,
};
use crate::sequence::{
    validate_sequence_record, ProteinSequence, ResidueIssue, SequenceKind, SequenceRecord,
    SequenceValidationIssue, SequenceValidationReport, ValidatedSequence,
};
use crate::tokenizer::{
    protein_tokenizer_config_for_profile, summarize_tokenized_proteins,
    tokenize_protein_with_config, ProteinBatchSummary, ProteinTokenizerConfig,
    ProteinTokenizerProfile, TokenizedProtein, UnknownTokenPolicy,
};
use serde::{Deserialize, Serialize};

mod reproducibility;
use reproducibility::{workflow_hashes, WorkflowHashInput, CORE_WORKFLOW_COMMAND};
pub use reproducibility::{SequenceWorkflowHashes, SequenceWorkflowInvocation};

const PROTEIN_WORKFLOW_NAME: &str = "protein_model_input.v0";
const SEQUENCE_WORKFLOW_NAME: &str = "sequence_model_input.v0";
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
            arguments: vec![format!("records={}", records.len())],
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
    prepare_model_input_workflow_with_config(
        input_hash,
        records,
        policy,
        protein_tokenizer_config_for_profile(ProteinTokenizerProfile::Protein20),
        invocation,
    )
}

/// Build a profile-aware validation -> tokenization -> model-input workflow.
pub fn prepare_model_input_workflow_with_config(
    input_hash: String,
    records: &[ProteinSequence],
    policy: ModelInputPolicy,
    config: ProteinTokenizerConfig,
    invocation: SequenceWorkflowInvocation,
) -> Result<SequenceWorkflowOutput, ModelInputBuildError> {
    validate_workflow_input_hash(&input_hash)?;
    validate_model_input_policy(&policy)?;

    let validation = validate_records_for_profile(records, config.profile);
    let tokenized: Vec<_> = records
        .iter()
        .map(|record| tokenize_protein_with_config(record, &config))
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
    let workflow = workflow_name(config.profile);
    let hashes = workflow_hashes(WorkflowHashInput {
        workflow,
        profile: config.profile,
        input_hash: input_hash.as_str(),
        policy: &policy,
        invocation: &invocation,
        validation: &validation,
        tokenization: &tokenization,
        model_input: &model_input,
        readiness_issues: &readiness_issues,
    });

    Ok(SequenceWorkflowOutput {
        workflow: workflow.to_string(),
        model_ready,
        provenance: provenance(input_hash, policy, invocation, hashes, config.profile),
        validation,
        tokenization,
        model_input,
        readiness_issues,
    })
}

fn validate_records_for_profile(
    records: &[ProteinSequence],
    profile: ProteinTokenizerProfile,
) -> SequenceValidationReport {
    let kind = profile.sequence_kind();
    let sequences = records
        .iter()
        .map(|record| {
            let validation = validate_sequence_record(&SequenceRecord {
                id: record.id.clone(),
                sequence: String::from_utf8_lossy(&record.sequence).into_owned(),
                kind,
            });
            ValidatedSequence {
                id: validation.id,
                sequence: validation.sequence,
                alphabet: validation.alphabet,
                valid: validation.valid,
                warnings: validation
                    .warnings
                    .into_iter()
                    .map(issue_to_residue_issue)
                    .collect(),
                errors: validation
                    .errors
                    .into_iter()
                    .map(issue_to_residue_issue)
                    .collect(),
            }
        })
        .collect();
    crate::sequence::summarize_validated_sequences(sequences)
}

fn issue_to_residue_issue(issue: SequenceValidationIssue) -> ResidueIssue {
    ResidueIssue {
        residue: issue.symbol,
        position: issue.position,
    }
}

fn validate_workflow_input_hash(input_hash: &str) -> Result<(), ModelInputBuildError> {
    if crate::verification::is_stable_input_hash(input_hash) {
        Ok(())
    } else {
        Err(ModelInputBuildError::InvalidInputHash {
            input_hash: input_hash.to_string(),
        })
    }
}

fn provenance(
    input_hash: String,
    policy: ModelInputPolicy,
    invocation: SequenceWorkflowInvocation,
    hashes: SequenceWorkflowHashes,
    profile: ProteinTokenizerProfile,
) -> SequenceWorkflowProvenance {
    let vocab = crate::tokenizer::inspect_protein_tokenizer_config(
        &protein_tokenizer_config_for_profile(profile),
    )
    .vocabulary;
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

fn workflow_name(profile: ProteinTokenizerProfile) -> &'static str {
    match profile.sequence_kind() {
        SequenceKind::Protein => PROTEIN_WORKFLOW_NAME,
        SequenceKind::Dna | SequenceKind::Rna => SEQUENCE_WORKFLOW_NAME,
    }
}

fn readiness_issues(tokenized: &[TokenizedProtein]) -> Vec<SequenceWorkflowReadinessIssue> {
    tokenized
        .iter()
        .filter(|record| {
            record.tokens.is_empty() || !record.warnings.is_empty() || !record.errors.is_empty()
        })
        .map(|record| {
            let warning_count = record.warnings.len();
            let error_count = record.errors.len();
            let message = if record.tokens.is_empty() {
                format!(
                    "sequence '{}' is not model-ready: empty sequences cannot be converted into model input",
                    record.id
                )
            } else {
                format!(
                    "sequence '{}' is not model-ready: {warning_count} warnings and {error_count} errors must be resolved before model-input generation",
                    record.id
                )
            };
            SequenceWorkflowReadinessIssue {
                code: READINESS_ISSUE_CODE.to_string(),
                id: record.id.clone(),
                warning_count,
                error_count,
                message,
            }
        })
        .collect()
}

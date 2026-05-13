use crate::tokenizer::TokenizedProtein;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Policy for converting tokenized proteins into model-ready arrays.
pub struct ModelInputPolicy {
    /// Maximum token length per record.
    pub max_length: usize,
    /// Token ID used when fixed-length padding is requested.
    pub pad_token_id: u8,
    pub padding: PaddingPolicy,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
/// Padding strategy for model input records.
pub enum PaddingPolicy {
    /// Pad every record to `max_length`.
    FixedLength,
    /// Preserve each record's truncated length without padding.
    NoPadding,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Batch of model-ready input records.
pub struct ModelInput {
    pub policy: ModelInputPolicy,
    pub records: Vec<ModelInputRecord>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Model-ready representation of one tokenized protein.
pub struct ModelInputRecord {
    pub id: String,
    /// Token IDs after truncation and optional padding.
    pub input_ids: Vec<u8>,
    /// Attention mask with `1` for real tokens and `0` for padding.
    pub attention_mask: Vec<u8>,
    /// True when input tokens were truncated to `max_length`.
    pub truncated: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Errors returned by checked model-input builders.
pub enum ModelInputBuildError {
    /// The model input policy is internally invalid.
    InvalidPolicy { message: String },
    /// A tokenized sequence still contains unresolved warnings or errors.
    InvalidTokenizedSequence {
        id: String,
        warning_count: usize,
        error_count: usize,
    },
}

/// Build model input without rejecting unresolved tokenization warnings or errors.
pub fn build_model_inputs_unchecked(
    tokenized: &[TokenizedProtein],
    policy: ModelInputPolicy,
) -> ModelInput {
    let records = tokenized
        .iter()
        .map(|record| model_input_from_tokenized(record, &policy))
        .collect();

    ModelInput { policy, records }
}

/// Build model input after rejecting invalid policies and unresolved residue issues.
pub fn build_model_inputs_checked(
    tokenized: &[TokenizedProtein],
    policy: ModelInputPolicy,
) -> Result<ModelInput, ModelInputBuildError> {
    validate_model_input_policy(&policy)?;

    for record in tokenized {
        if !record.warnings.is_empty() || !record.errors.is_empty() {
            return Err(ModelInputBuildError::InvalidTokenizedSequence {
                id: record.id.clone(),
                warning_count: record.warnings.len(),
                error_count: record.errors.len(),
            });
        }
    }

    Ok(build_model_inputs_unchecked(tokenized, policy))
}

/// Validate a model-input policy without building records.
pub fn validate_model_input_policy(policy: &ModelInputPolicy) -> Result<(), ModelInputBuildError> {
    if policy.max_length == 0 {
        return Err(ModelInputBuildError::InvalidPolicy {
            message: "max_length must be greater than zero".to_string(),
        });
    }

    Ok(())
}

fn model_input_from_tokenized(
    tokenized: &TokenizedProtein,
    policy: &ModelInputPolicy,
) -> ModelInputRecord {
    let end = tokenized.tokens.len().min(policy.max_length);
    let mut input_ids = tokenized.tokens[..end].to_vec();
    let mut attention_mask = vec![1; input_ids.len()];
    let truncated = tokenized.tokens.len() > policy.max_length;

    if policy.padding == PaddingPolicy::FixedLength {
        input_ids.resize(policy.max_length, policy.pad_token_id);
        attention_mask.resize(policy.max_length, 0);
    }

    ModelInputRecord {
        id: tokenized.id.clone(),
        input_ids,
        attention_mask,
        truncated,
    }
}

impl fmt::Display for ModelInputBuildError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidPolicy { message } => write!(f, "invalid model input policy: {message}"),
            Self::InvalidTokenizedSequence {
                id,
                warning_count,
                error_count,
            } => write!(
                f,
                "sequence '{id}' is not model-ready: {warning_count} warnings and {error_count} errors must be resolved before building model input"
            ),
        }
    }
}

impl std::error::Error for ModelInputBuildError {}

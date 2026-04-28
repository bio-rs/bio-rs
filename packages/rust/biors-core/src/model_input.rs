use crate::TokenizedProtein;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModelInputPolicy {
    pub max_length: usize,
    pub pad_token_id: u8,
    pub padding: PaddingPolicy,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PaddingPolicy {
    FixedLength,
    NoPadding,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModelInput {
    pub policy: ModelInputPolicy,
    pub records: Vec<ModelInputRecord>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModelInputRecord {
    pub id: String,
    pub input_ids: Vec<u8>,
    pub attention_mask: Vec<u8>,
    pub truncated: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModelInputBuildError {
    InvalidPolicy {
        message: String,
    },
    InvalidTokenizedSequence {
        id: String,
        warning_count: usize,
        error_count: usize,
    },
}

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

#[deprecated(
    since = "0.9.7",
    note = "use build_model_inputs_checked for safe output or build_model_inputs_unchecked when unresolved residues are intentional"
)]
pub fn build_model_inputs(tokenized: &[TokenizedProtein], policy: ModelInputPolicy) -> ModelInput {
    build_model_inputs_unchecked(tokenized, policy)
}

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
    let mut input_ids: Vec<u8> = tokenized
        .tokens
        .iter()
        .copied()
        .take(policy.max_length)
        .collect();
    let mut attention_mask = vec![1; input_ids.len()];
    let truncated = tokenized.tokens.len() > policy.max_length;

    if policy.padding == PaddingPolicy::FixedLength {
        while input_ids.len() < policy.max_length {
            input_ids.push(policy.pad_token_id);
            attention_mask.push(0);
        }
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

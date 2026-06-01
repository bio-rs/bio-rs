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
    /// Workflow provenance received an invalid input hash.
    InvalidInputHash { input_hash: String },
    /// A tokenized sequence has no model input tokens.
    EmptyTokenizedSequence { id: String },
    /// A tokenized sequence still contains unresolved warnings or errors.
    InvalidTokenizedSequence {
        id: String,
        warning_count: usize,
        error_count: usize,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Errors returned when validating already-built model-input payloads.
pub enum ModelInputPayloadError {
    /// One record has different input ID and attention-mask lengths.
    LengthMismatch {
        id: String,
        input_ids: usize,
        attention_mask: usize,
    },
    /// Fixed-length payload records must match the policy max length.
    FixedLengthMismatch {
        id: String,
        expected: usize,
        actual: usize,
    },
    /// No-padding payload records cannot exceed the policy max length.
    NoPaddingLengthExceeded {
        id: String,
        max_length: usize,
        actual: usize,
    },
    /// Attention masks must contain only `0` and `1`.
    NonBinaryAttentionMask { id: String, index: usize, value: u8 },
    /// A record has no tokens selected by its attention mask.
    EmptyUnmaskedTokens { id: String },
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
        if record.tokens.is_empty() {
            return Err(ModelInputBuildError::EmptyTokenizedSequence {
                id: record.id.clone(),
            });
        }
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

/// Validate an already-built model-input payload received at an integration boundary.
pub fn validate_model_input_payload(input: &ModelInput) -> Result<(), ModelInputPayloadError> {
    for record in &input.records {
        if record.input_ids.len() != record.attention_mask.len() {
            return Err(ModelInputPayloadError::LengthMismatch {
                id: record.id.clone(),
                input_ids: record.input_ids.len(),
                attention_mask: record.attention_mask.len(),
            });
        }
        match input.policy.padding {
            PaddingPolicy::FixedLength if record.input_ids.len() != input.policy.max_length => {
                return Err(ModelInputPayloadError::FixedLengthMismatch {
                    id: record.id.clone(),
                    expected: input.policy.max_length,
                    actual: record.input_ids.len(),
                });
            }
            PaddingPolicy::NoPadding if record.input_ids.len() > input.policy.max_length => {
                return Err(ModelInputPayloadError::NoPaddingLengthExceeded {
                    id: record.id.clone(),
                    max_length: input.policy.max_length,
                    actual: record.input_ids.len(),
                });
            }
            _ => {}
        }

        for (index, value) in record.attention_mask.iter().copied().enumerate() {
            if value != 0 && value != 1 {
                return Err(ModelInputPayloadError::NonBinaryAttentionMask {
                    id: record.id.clone(),
                    index,
                    value,
                });
            }
        }

        if !record.attention_mask.contains(&1) {
            return Err(ModelInputPayloadError::EmptyUnmaskedTokens {
                id: record.id.clone(),
            });
        }
    }

    Ok(())
}

fn model_input_from_tokenized(
    tokenized: &TokenizedProtein,
    policy: &ModelInputPolicy,
) -> ModelInputRecord {
    let end = tokenized.tokens.len().min(policy.max_length);
    let truncated = tokenized.tokens.len() > policy.max_length;

    let output_len = match policy.padding {
        PaddingPolicy::FixedLength => policy.max_length,
        PaddingPolicy::NoPadding => end,
    };

    let mut input_ids = Vec::with_capacity(output_len);
    input_ids.extend_from_slice(&tokenized.tokens[..end]);

    let mut attention_mask = Vec::with_capacity(output_len);
    attention_mask.resize(end, 1);

    if matches!(policy.padding, PaddingPolicy::FixedLength) {
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
            Self::InvalidInputHash { input_hash } => write!(
                f,
                "invalid workflow input hash '{input_hash}': expected fnv1a64:<16 lowercase hex>"
            ),
            Self::EmptyTokenizedSequence { id } => write!(
                f,
                "sequence '{id}' is empty and cannot be converted into model input"
            ),
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

impl ModelInputPayloadError {
    pub const fn code(&self) -> &'static str {
        match self {
            Self::LengthMismatch { .. } => "model_input.length_mismatch",
            Self::FixedLengthMismatch { .. } => "model_input.fixed_length_mismatch",
            Self::NoPaddingLengthExceeded { .. } => "model_input.no_padding_length_exceeded",
            Self::NonBinaryAttentionMask { .. } => "model_input.non_binary_attention_mask",
            Self::EmptyUnmaskedTokens { .. } => "model_input.empty_attention_mask",
        }
    }
}

impl fmt::Display for ModelInputPayloadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::LengthMismatch {
                id,
                input_ids,
                attention_mask,
            } => write!(
                f,
                "record '{id}' has {input_ids} input ids but {attention_mask} attention-mask values"
            ),
            Self::FixedLengthMismatch {
                id,
                expected,
                actual,
            } => write!(
                f,
                "record '{id}' has {actual} input ids, expected fixed length {expected}"
            ),
            Self::NoPaddingLengthExceeded {
                id,
                max_length,
                actual,
            } => write!(
                f,
                "record '{id}' has {actual} input ids, exceeding max_length {max_length}"
            ),
            Self::NonBinaryAttentionMask { id, index, value } => write!(
                f,
                "record '{id}' attention_mask[{index}] is {value}, expected 0 or 1"
            ),
            Self::EmptyUnmaskedTokens { id } => {
                write!(f, "record '{id}' has no unmasked tokens")
            }
        }
    }
}

impl std::error::Error for ModelInputPayloadError {}

#[cfg(test)]
mod tests;

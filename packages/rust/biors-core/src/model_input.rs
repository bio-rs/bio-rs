use crate::tokenizer::TokenizedProtein;
use serde::{Deserialize, Serialize};

mod errors;

pub use errors::{ModelInputBuildError, ModelInputPayloadError};

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

    let mut records = Vec::with_capacity(tokenized.len());
    for record in tokenized {
        validate_tokenized_record_is_model_ready(record)?;
        records.push(model_input_from_tokenized(record, &policy));
    }

    Ok(ModelInput { policy, records })
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
        validate_model_input_record_payload(record, &input.policy)?;
    }

    Ok(())
}

fn validate_tokenized_record_is_model_ready(
    record: &TokenizedProtein,
) -> Result<(), ModelInputBuildError> {
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

    Ok(())
}

fn validate_model_input_record_payload(
    record: &ModelInputRecord,
    policy: &ModelInputPolicy,
) -> Result<(), ModelInputPayloadError> {
    validate_record_lengths(record, policy)?;
    validate_attention_mask_values(record)?;
    validate_attention_mask_selects_tokens(record)?;
    Ok(())
}

fn validate_record_lengths(
    record: &ModelInputRecord,
    policy: &ModelInputPolicy,
) -> Result<(), ModelInputPayloadError> {
    if record.input_ids.len() != record.attention_mask.len() {
        return Err(ModelInputPayloadError::LengthMismatch {
            id: record.id.clone(),
            input_ids: record.input_ids.len(),
            attention_mask: record.attention_mask.len(),
        });
    }

    match &policy.padding {
        PaddingPolicy::FixedLength if record.input_ids.len() != policy.max_length => {
            Err(ModelInputPayloadError::FixedLengthMismatch {
                id: record.id.clone(),
                expected: policy.max_length,
                actual: record.input_ids.len(),
            })
        }
        PaddingPolicy::NoPadding if record.input_ids.len() > policy.max_length => {
            Err(ModelInputPayloadError::NoPaddingLengthExceeded {
                id: record.id.clone(),
                max_length: policy.max_length,
                actual: record.input_ids.len(),
            })
        }
        _ => Ok(()),
    }
}

fn validate_attention_mask_values(record: &ModelInputRecord) -> Result<(), ModelInputPayloadError> {
    for (index, value) in record.attention_mask.iter().copied().enumerate() {
        if value != 0 && value != 1 {
            return Err(ModelInputPayloadError::NonBinaryAttentionMask {
                id: record.id.clone(),
                index,
                value,
            });
        }
    }

    Ok(())
}

fn validate_attention_mask_selects_tokens(
    record: &ModelInputRecord,
) -> Result<(), ModelInputPayloadError> {
    if !record.attention_mask.contains(&1) {
        return Err(ModelInputPayloadError::EmptyUnmaskedTokens {
            id: record.id.clone(),
        });
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

    let (input_ids, attention_mask) = match &policy.padding {
        PaddingPolicy::FixedLength => {
            let mut input_ids = vec![policy.pad_token_id; output_len];
            let mut attention_mask = vec![0; output_len];
            input_ids[..end].copy_from_slice(&tokenized.tokens[..end]);
            attention_mask[..end].fill(1);
            (input_ids, attention_mask)
        }
        PaddingPolicy::NoPadding => {
            let input_ids = tokenized.tokens[..end].to_vec();
            let attention_mask = vec![1; end];
            (input_ids, attention_mask)
        }
    };

    ModelInputRecord {
        id: tokenized.id.clone(),
        input_ids,
        attention_mask,
        truncated,
    }
}

#[cfg(test)]
mod tests;

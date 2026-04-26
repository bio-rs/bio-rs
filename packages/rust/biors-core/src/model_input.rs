use crate::TokenizedProtein;
use serde::{Deserialize, Serialize};

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

pub fn build_model_inputs(tokenized: &[TokenizedProtein], policy: ModelInputPolicy) -> ModelInput {
    let records = tokenized
        .iter()
        .map(|record| model_input_from_tokenized(record, &policy))
        .collect();

    ModelInput { policy, records }
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

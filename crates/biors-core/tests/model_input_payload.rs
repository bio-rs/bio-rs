use biors_core::model_input::{
    validate_model_input_payload, ModelInput, ModelInputPayloadError, ModelInputPolicy,
    ModelInputRecord, PaddingPolicy,
};

#[test]
fn validate_model_input_payload_rejects_length_mismatch() {
    let input = model_input_payload("seq1", vec![1, 2], vec![1]);
    let result = validate_model_input_payload(&input);

    assert!(
        matches!(result, Err(ModelInputPayloadError::LengthMismatch { id, input_ids, attention_mask })
            if id == "seq1" && input_ids == 2 && attention_mask == 1
        )
    );
}

#[test]
fn validate_model_input_payload_rejects_non_binary_attention_mask() {
    let input = model_input_payload("seq2", vec![1, 2, 0], vec![1, 2, 0]);
    let result = validate_model_input_payload(&input);

    assert!(
        matches!(result, Err(ModelInputPayloadError::NonBinaryAttentionMask { id, index, value })
            if id == "seq2" && index == 1 && value == 2
        )
    );
}

#[test]
fn validate_model_input_payload_rejects_short_fixed_length_record() {
    let input = model_input_payload_with_policy(
        make_policy(4, PaddingPolicy::FixedLength),
        "seq-fixed",
        vec![1, 2],
        vec![1, 1],
    );
    let result = validate_model_input_payload(&input);

    assert!(
        matches!(result, Err(ModelInputPayloadError::FixedLengthMismatch { id, expected, actual })
            if id == "seq-fixed" && expected == 4 && actual == 2
        )
    );
}

#[test]
fn validate_model_input_payload_rejects_long_no_padding_record() {
    let input = model_input_payload_with_policy(
        make_policy(2, PaddingPolicy::NoPadding),
        "seq-long",
        vec![1, 2, 3],
        vec![1, 1, 1],
    );
    let result = validate_model_input_payload(&input);

    assert!(
        matches!(result, Err(ModelInputPayloadError::NoPaddingLengthExceeded { id, max_length, actual })
            if id == "seq-long" && max_length == 2 && actual == 3
        )
    );
}

#[test]
fn validate_model_input_payload_rejects_empty_unmasked_tokens() {
    let input = model_input_payload("seq3", vec![0, 0], vec![0, 0]);
    let result = validate_model_input_payload(&input);

    assert!(
        matches!(result, Err(ModelInputPayloadError::EmptyUnmaskedTokens { id }) if id == "seq3")
    );
}

#[test]
fn validate_model_input_payload_accepts_binary_masked_records() {
    let input = model_input_payload("seq4", vec![1, 2, 0], vec![1, 1, 0]);
    assert!(validate_model_input_payload(&input).is_ok());
}

fn make_policy(max_length: usize, padding: PaddingPolicy) -> ModelInputPolicy {
    ModelInputPolicy {
        max_length,
        pad_token_id: 0,
        padding,
    }
}

fn model_input_payload(id: &str, input_ids: Vec<u8>, attention_mask: Vec<u8>) -> ModelInput {
    model_input_payload_with_policy(
        make_policy(input_ids.len().max(1), PaddingPolicy::FixedLength),
        id,
        input_ids,
        attention_mask,
    )
}

fn model_input_payload_with_policy(
    policy: ModelInputPolicy,
    id: &str,
    input_ids: Vec<u8>,
    attention_mask: Vec<u8>,
) -> ModelInput {
    ModelInput {
        policy,
        records: vec![ModelInputRecord {
            id: id.to_string(),
            input_ids,
            attention_mask,
            truncated: false,
        }],
    }
}

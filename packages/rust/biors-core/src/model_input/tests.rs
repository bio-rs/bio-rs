use crate::model_input::{
    build_model_inputs_checked, build_model_inputs_unchecked, validate_model_input_payload,
    validate_model_input_policy, ModelInput, ModelInputBuildError, ModelInputPayloadError,
    ModelInputPolicy, ModelInputRecord, PaddingPolicy, TokenizedProtein,
};
use crate::sequence::ResidueIssue;

fn make_protein(
    id: &str,
    tokens: Vec<u8>,
    warnings: Vec<ResidueIssue>,
    errors: Vec<ResidueIssue>,
) -> TokenizedProtein {
    let length = tokens.len();
    let valid = warnings.is_empty() && errors.is_empty();
    TokenizedProtein {
        id: id.to_string(),
        length,
        alphabet: "protein-20".to_string(),
        valid,
        tokens,
        warnings,
        errors,
    }
}

fn make_policy(max_length: usize, padding: PaddingPolicy) -> ModelInputPolicy {
    ModelInputPolicy {
        max_length,
        pad_token_id: 0,
        padding,
    }
}

#[test]
fn validate_model_input_policy_rejects_zero_max_length() {
    let policy = make_policy(0, PaddingPolicy::NoPadding);
    let result = validate_model_input_policy(&policy);
    assert!(
        matches!(result, Err(ModelInputBuildError::InvalidPolicy { message }) if message == "max_length must be greater than zero")
    );
}

#[test]
fn validate_model_input_policy_accepts_non_zero_max_length() {
    let policy = make_policy(1, PaddingPolicy::NoPadding);
    assert!(validate_model_input_policy(&policy).is_ok());
}

#[test]
fn build_model_inputs_checked_rejects_warning() {
    let protein = make_protein(
        "seq1",
        vec![1, 2, 3],
        vec![ResidueIssue {
            residue: 'X',
            position: 1,
        }],
        vec![],
    );
    let policy = make_policy(5, PaddingPolicy::NoPadding);
    let result = build_model_inputs_checked(&[protein], policy);
    assert!(
        matches!(result, Err(ModelInputBuildError::InvalidTokenizedSequence { id, warning_count, error_count })
            if id == "seq1" && warning_count == 1 && error_count == 0
        )
    );
}

#[test]
fn build_model_inputs_checked_rejects_error() {
    let protein = make_protein(
        "seq2",
        vec![1, 2, 3],
        vec![],
        vec![ResidueIssue {
            residue: 'Z',
            position: 2,
        }],
    );
    let policy = make_policy(5, PaddingPolicy::NoPadding);
    let result = build_model_inputs_checked(&[protein], policy);
    assert!(
        matches!(result, Err(ModelInputBuildError::InvalidTokenizedSequence { id, warning_count, error_count })
            if id == "seq2" && warning_count == 0 && error_count == 1
        )
    );
}

#[test]
fn build_model_inputs_checked_rejects_warning_and_error() {
    let protein = make_protein(
        "seq3",
        vec![1, 2, 3],
        vec![ResidueIssue {
            residue: 'X',
            position: 1,
        }],
        vec![ResidueIssue {
            residue: 'Z',
            position: 2,
        }],
    );
    let policy = make_policy(5, PaddingPolicy::NoPadding);
    let result = build_model_inputs_checked(&[protein], policy);
    assert!(
        matches!(result, Err(ModelInputBuildError::InvalidTokenizedSequence { id, warning_count, error_count })
            if id == "seq3" && warning_count == 1 && error_count == 1
        )
    );
}

#[test]
fn build_model_inputs_checked_accepts_valid() {
    let protein = make_protein("seq4", vec![1, 2, 3], vec![], vec![]);
    let policy = make_policy(5, PaddingPolicy::NoPadding);
    let result = build_model_inputs_checked(std::slice::from_ref(&protein), policy.clone());
    assert!(result.is_ok());
    let model_input = result.unwrap();
    assert_eq!(model_input.policy, policy);
    assert_eq!(model_input.records.len(), 1);
    assert_eq!(model_input.records[0].id, "seq4");
    assert_eq!(model_input.records[0].input_ids, vec![1, 2, 3]);
    assert_eq!(model_input.records[0].attention_mask, vec![1, 1, 1]);
    assert!(!model_input.records[0].truncated);
}

#[test]
fn build_model_inputs_checked_rejects_invalid_policy() {
    let protein = make_protein("seq5", vec![1, 2, 3], vec![], vec![]);
    let policy = make_policy(0, PaddingPolicy::NoPadding);
    let result = build_model_inputs_checked(&[protein], policy);
    assert!(
        matches!(result, Err(ModelInputBuildError::InvalidPolicy { message }) if message == "max_length must be greater than zero")
    );
}

#[test]
fn build_model_inputs_unchecked_builds_with_warnings() {
    let protein = make_protein(
        "seq6",
        vec![1, 2, 3],
        vec![ResidueIssue {
            residue: 'X',
            position: 1,
        }],
        vec![],
    );
    let policy = make_policy(5, PaddingPolicy::NoPadding);
    let model_input = build_model_inputs_unchecked(std::slice::from_ref(&protein), policy.clone());
    assert_eq!(model_input.policy, policy);
    assert_eq!(model_input.records.len(), 1);
    assert_eq!(model_input.records[0].id, "seq6");
    assert_eq!(model_input.records[0].input_ids, vec![1, 2, 3]);
    assert_eq!(model_input.records[0].attention_mask, vec![1, 1, 1]);
    assert!(!model_input.records[0].truncated);
}

#[test]
fn build_model_inputs_unchecked_builds_with_errors() {
    let protein = make_protein(
        "seq7",
        vec![1, 2, 3],
        vec![],
        vec![ResidueIssue {
            residue: 'Z',
            position: 2,
        }],
    );
    let policy = make_policy(5, PaddingPolicy::NoPadding);
    let model_input = build_model_inputs_unchecked(std::slice::from_ref(&protein), policy.clone());
    assert_eq!(model_input.policy, policy);
    assert_eq!(model_input.records.len(), 1);
    assert_eq!(model_input.records[0].id, "seq7");
    assert_eq!(model_input.records[0].input_ids, vec![1, 2, 3]);
    assert_eq!(model_input.records[0].attention_mask, vec![1, 1, 1]);
    assert!(!model_input.records[0].truncated);
}

#[test]
fn truncation_behavior_no_padding() {
    let protein = make_protein("seq8", vec![1, 2, 3, 4, 5, 6, 7], vec![], vec![]);
    let policy = make_policy(4, PaddingPolicy::NoPadding);
    let model_input = build_model_inputs_unchecked(&[protein], policy);
    assert_eq!(model_input.records[0].input_ids, vec![1, 2, 3, 4]);
    assert_eq!(model_input.records[0].attention_mask, vec![1, 1, 1, 1]);
    assert!(model_input.records[0].truncated);
}

#[test]
fn truncation_behavior_fixed_length_padding() {
    let protein = make_protein("seq9", vec![1, 2, 3, 4, 5, 6, 7], vec![], vec![]);
    let policy = make_policy(4, PaddingPolicy::FixedLength);
    let model_input = build_model_inputs_unchecked(&[protein], policy);
    assert_eq!(model_input.records[0].input_ids, vec![1, 2, 3, 4]);
    assert_eq!(model_input.records[0].attention_mask, vec![1, 1, 1, 1]);
    assert!(model_input.records[0].truncated);
}

#[test]
fn no_truncation_when_tokens_fit() {
    let protein = make_protein("seq10", vec![1, 2, 3], vec![], vec![]);
    let policy = make_policy(5, PaddingPolicy::NoPadding);
    let model_input = build_model_inputs_unchecked(&[protein], policy);
    assert_eq!(model_input.records[0].input_ids, vec![1, 2, 3]);
    assert_eq!(model_input.records[0].attention_mask, vec![1, 1, 1]);
    assert!(!model_input.records[0].truncated);
}

#[test]
fn padding_behavior_fixed_length() {
    let protein = make_protein("seq11", vec![1, 2, 3], vec![], vec![]);
    let policy = make_policy(6, PaddingPolicy::FixedLength);
    let model_input = build_model_inputs_unchecked(&[protein], policy);
    assert_eq!(model_input.records[0].input_ids, vec![1, 2, 3, 0, 0, 0]);
    assert_eq!(
        model_input.records[0].attention_mask,
        vec![1, 1, 1, 0, 0, 0]
    );
    assert!(!model_input.records[0].truncated);
}

#[test]
fn padding_behavior_no_padding() {
    let protein = make_protein("seq12", vec![1, 2, 3], vec![], vec![]);
    let policy = make_policy(6, PaddingPolicy::NoPadding);
    let model_input = build_model_inputs_unchecked(&[protein], policy);
    assert_eq!(model_input.records[0].input_ids, vec![1, 2, 3]);
    assert_eq!(model_input.records[0].attention_mask, vec![1, 1, 1]);
    assert!(!model_input.records[0].truncated);
}

#[test]
fn truncation_and_padding_combined() {
    let protein = make_protein("seq13", vec![1, 2, 3, 4, 5, 6, 7], vec![], vec![]);
    let policy = make_policy(5, PaddingPolicy::FixedLength);
    let model_input = build_model_inputs_unchecked(&[protein], policy);
    assert_eq!(model_input.records[0].input_ids, vec![1, 2, 3, 4, 5]);
    assert_eq!(model_input.records[0].attention_mask, vec![1, 1, 1, 1, 1]);
    assert!(model_input.records[0].truncated);
}

#[test]
fn multiple_records_mixed_issues_unchecked() {
    let protein_a = make_protein("a", vec![1, 2], vec![], vec![]);
    let protein_b = make_protein(
        "b",
        vec![3, 4, 5],
        vec![ResidueIssue {
            residue: 'X',
            position: 1,
        }],
        vec![],
    );
    let policy = make_policy(4, PaddingPolicy::FixedLength);
    let model_input = build_model_inputs_unchecked(&[protein_a, protein_b], policy);
    assert_eq!(model_input.records.len(), 2);
    assert_eq!(model_input.records[0].id, "a");
    assert_eq!(model_input.records[0].input_ids, vec![1, 2, 0, 0]);
    assert_eq!(model_input.records[0].attention_mask, vec![1, 1, 0, 0]);
    assert!(!model_input.records[0].truncated);

    assert_eq!(model_input.records[1].id, "b");
    assert_eq!(model_input.records[1].input_ids, vec![3, 4, 5, 0]);
    assert_eq!(model_input.records[1].attention_mask, vec![1, 1, 1, 0]);
    assert!(!model_input.records[1].truncated);
}

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

fn model_input_payload(id: &str, input_ids: Vec<u8>, attention_mask: Vec<u8>) -> ModelInput {
    ModelInput {
        policy: make_policy(input_ids.len().max(1), PaddingPolicy::FixedLength),
        records: vec![ModelInputRecord {
            id: id.to_string(),
            input_ids,
            attention_mask,
            truncated: false,
        }],
    }
}

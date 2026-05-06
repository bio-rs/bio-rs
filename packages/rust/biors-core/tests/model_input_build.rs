use biors_core::model_input::{
    build_model_inputs_checked, build_model_inputs_unchecked, ModelInputBuildError,
    ModelInputPolicy, PaddingPolicy,
};
use biors_core::tokenizer::tokenize_fasta_records;

#[test]
fn builds_deterministic_model_input_with_attention_mask() {
    let tokenized = tokenize_fasta_records(">seq1\nACDEFG\n").expect("valid FASTA");
    let model_input = build_model_inputs_unchecked(
        &tokenized,
        ModelInputPolicy {
            max_length: 4,
            pad_token_id: 0,
            padding: PaddingPolicy::FixedLength,
        },
    );

    assert_eq!(model_input.records[0].input_ids, vec![0, 1, 2, 3]);
    assert_eq!(model_input.records[0].attention_mask, vec![1, 1, 1, 1]);
    assert!(model_input.records[0].truncated);
}

#[test]
fn pads_model_input_to_fixed_length() {
    let tokenized = tokenize_fasta_records(">seq1\nAC\n").expect("valid FASTA");
    let model_input = build_model_inputs_unchecked(
        &tokenized,
        ModelInputPolicy {
            max_length: 4,
            pad_token_id: 255,
            padding: PaddingPolicy::FixedLength,
        },
    );

    assert_eq!(model_input.records[0].input_ids, vec![0, 1, 255, 255]);
    assert_eq!(model_input.records[0].attention_mask, vec![1, 1, 0, 0]);
    assert!(!model_input.records[0].truncated);
}

#[test]
fn preserves_unpadded_model_input_when_no_padding_is_requested() {
    let tokenized = tokenize_fasta_records(">seq1\nAC\n").expect("valid FASTA");
    let model_input = build_model_inputs_unchecked(
        &tokenized,
        ModelInputPolicy {
            max_length: 4,
            pad_token_id: 255,
            padding: PaddingPolicy::NoPadding,
        },
    );

    assert_eq!(model_input.records[0].input_ids, vec![0, 1]);
    assert_eq!(model_input.records[0].attention_mask, vec![1, 1]);
    assert!(!model_input.records[0].truncated);
}

#[test]
fn rejects_model_input_for_sequences_with_ambiguous_or_invalid_residues() {
    let tokenized = tokenize_fasta_records(">seq1\nAX*\n").expect("valid FASTA envelope");
    let error = build_model_inputs_checked(
        &tokenized,
        ModelInputPolicy {
            max_length: 8,
            pad_token_id: 0,
            padding: PaddingPolicy::FixedLength,
        },
    )
    .expect_err("invalid tokenized sequence should not become model-ready input");

    assert_eq!(
        error,
        ModelInputBuildError::InvalidTokenizedSequence {
            id: "seq1".to_string(),
            warning_count: 1,
            error_count: 1,
        }
    );
}

#[test]
fn rejects_zero_length_model_input_policy() {
    let tokenized = tokenize_fasta_records(">seq1\nACDE\n").expect("valid FASTA");
    let error = build_model_inputs_checked(
        &tokenized,
        ModelInputPolicy {
            max_length: 0,
            pad_token_id: 0,
            padding: PaddingPolicy::FixedLength,
        },
    )
    .expect_err("zero max_length should fail");

    assert_eq!(
        error,
        ModelInputBuildError::InvalidPolicy {
            message: "max_length must be greater than zero".to_string(),
        }
    );
}

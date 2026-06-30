use biors_core::model_input::{ModelInputBuildError, ModelInputPolicy, PaddingPolicy};
use biors_core::sequence::ProteinSequence;
use biors_core::tokenizer::UnknownTokenPolicy;
use biors_core::workflow::prepare_protein_model_input_workflow;

const CORE_WORKFLOW_COMMAND: &str = "biors-core prepare_protein_model_input_workflow";
const READINESS_ISSUE_CODE: &str = "sequence.not_model_ready";

#[test]
fn workflow_preserves_validation_tokenization_and_model_input() {
    let output = prepare_protein_model_input_workflow(
        "fnv1a64:0000000000000000".to_string(),
        &[ProteinSequence::new_normalized("seq1", "ACDE")],
        fixed_length_policy(),
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
    assert_eq!(output.provenance.invocation.command, CORE_WORKFLOW_COMMAND);
    assert_eq!(output.provenance.invocation.arguments, vec!["records=1"]);
    assert!(output
        .provenance
        .hashes
        .vocabulary_sha256
        .starts_with("sha256:"));
    assert!(output
        .provenance
        .hashes
        .output_data_sha256
        .starts_with("sha256:"));
    assert_eq!(
        output.provenance.tokenizer.unknown_token_policy,
        UnknownTokenPolicy::WarnOrErrorWithUnknownToken
    );
    assert!(output.readiness_issues.is_empty());
}

#[test]
fn workflow_rejects_schema_invalid_input_hashes() {
    let result = prepare_protein_model_input_workflow(
        "not-a-fnv-hash".to_string(),
        &[ProteinSequence::new_normalized("seq1", "ACDE")],
        fixed_length_policy(),
    );

    assert!(matches!(
        result,
        Err(ModelInputBuildError::InvalidInputHash { input_hash }) if input_hash == "not-a-fnv-hash"
    ));
}

#[test]
fn workflow_normalizes_direct_lowercase_sequences_before_model_input() {
    let output = prepare_protein_model_input_workflow(
        "fnv1a64:0000000000000000".to_string(),
        &[ProteinSequence {
            id: "seq1".to_string(),
            sequence: b"ac de".to_vec(),
        }],
        fixed_length_policy(),
    )
    .expect("workflow output");

    assert!(output.model_ready);
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
        fixed_length_policy(),
    )
    .expect("workflow output");

    assert!(!output.model_ready);
    assert!(output.model_input.is_none());
    assert_eq!(output.validation.warning_count, 1);
    assert_eq!(output.validation.error_count, 1);
    assert_eq!(output.readiness_issues[0].code, READINESS_ISSUE_CODE);
}

#[test]
fn workflow_marks_direct_empty_sequence_not_model_ready() {
    let output = prepare_protein_model_input_workflow(
        "fnv1a64:0000000000000000".to_string(),
        &[ProteinSequence {
            id: "empty".to_string(),
            sequence: Vec::new(),
        }],
        fixed_length_policy(),
    )
    .expect("workflow output");

    assert!(!output.model_ready);
    assert!(output.model_input.is_none());
    assert_eq!(output.validation.records, 1);
    assert_eq!(output.readiness_issues[0].code, READINESS_ISSUE_CODE);
    assert_eq!(output.readiness_issues[0].id, "empty");
    assert!(output.readiness_issues[0]
        .message
        .contains("empty sequences"));
}

fn fixed_length_policy() -> ModelInputPolicy {
    ModelInputPolicy {
        max_length: 6,
        pad_token_id: 0,
        padding: PaddingPolicy::FixedLength,
    }
}

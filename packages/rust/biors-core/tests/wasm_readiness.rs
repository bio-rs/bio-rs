use biors_core::model_input::{ModelInputPolicy, PaddingPolicy};
use biors_core::sequence::{validate_protein_sequence, ProteinSequence};
use biors_core::tokenizer::tokenize_protein;
use biors_core::workflow::prepare_protein_model_input_workflow;

#[test]
fn public_sequence_apis_do_not_panic_on_invalid_utf8_bytes() {
    let sequence = ProteinSequence {
        id: "invalid-bytes".to_string(),
        sequence: vec![b'A', 0xff, b'C'],
    };

    let validation = validate_protein_sequence(&sequence);
    assert!(!validation.valid);
    assert_eq!(validation.errors.len(), 1);

    let tokenized = tokenize_protein(&sequence);
    assert!(!tokenized.valid);
    assert_eq!(tokenized.tokens, vec![0, 20, 1]);

    let workflow = prepare_protein_model_input_workflow(
        "fnv1a64:0000000000000000".to_string(),
        &[sequence],
        ModelInputPolicy {
            max_length: 8,
            pad_token_id: 0,
            padding: PaddingPolicy::FixedLength,
        },
    )
    .expect("workflow should return a report instead of panicking");
    assert!(!workflow.model_ready);
    assert_eq!(workflow.readiness_issues[0].id, "invalid-bytes");
}

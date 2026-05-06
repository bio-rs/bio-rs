use biors_core::{
    build_model_inputs_checked, load_protein_tokenizer_config_json, tokenize_protein_with_config,
    ModelInputPolicy, PaddingPolicy, ProteinSequence,
};
use serde_json::Value;
use std::fs;
use std::path::Path;

#[test]
fn draft_model_input_contract_covers_special_token_profile() {
    let repo = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
    let config = load_protein_tokenizer_config_json(
        &fs::read_to_string(
            repo.join("examples/model-input-contract/protein-20-special.config.json"),
        )
        .expect("read tokenizer config"),
    )
    .expect("tokenizer config");
    let expected: Value = serde_json::from_str(
        &fs::read_to_string(
            repo.join("examples/model-input-contract/protein-20-special.expected.json"),
        )
        .expect("read expected model input"),
    )
    .expect("expected JSON");

    let tokenized = tokenize_protein_with_config(
        &ProteinSequence {
            id: "seq1".to_string(),
            sequence: b"ACDE".to_vec(),
        },
        &config,
    );
    let model_input = build_model_inputs_checked(
        &[tokenized],
        ModelInputPolicy {
            max_length: 8,
            pad_token_id: 21,
            padding: PaddingPolicy::FixedLength,
        },
    )
    .expect("model input");

    assert_eq!(
        serde_json::to_value(model_input).expect("model input JSON"),
        expected
    );
}

#[test]
fn reference_python_preprocessing_parity_fixture_matches_core_output() {
    let repo = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
    let expected: Value = serde_json::from_str(
        &fs::read_to_string(
            repo.join("examples/model-input-contract/reference-python-parity.json"),
        )
        .expect("read Python parity fixture"),
    )
    .expect("parity JSON");
    let config = load_protein_tokenizer_config_json(
        &fs::read_to_string(
            repo.join("examples/model-input-contract/protein-20-special.config.json"),
        )
        .expect("read tokenizer config"),
    )
    .expect("tokenizer config");

    let tokenized = tokenize_protein_with_config(
        &ProteinSequence {
            id: "seq1".to_string(),
            sequence: b"acde".to_vec(),
        },
        &config,
    );

    assert_eq!(
        serde_json::to_value(tokenized).expect("token JSON"),
        expected
    );
}

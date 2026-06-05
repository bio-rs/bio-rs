use wasm_bindgen_test::*;

mod wasm_schema_support;

use wasm_schema_support::assert_matches_shared_workflow_schema_contract;

const WORKFLOW_OUTPUT_SCHEMA: &str =
    include_str!("../../../schemas/sequence-workflow-output.v0.json");

#[wasm_bindgen_test]
fn test_run_workflow() {
    let config = workflow_config(">seq1\nACDE\n");
    js_sys::Reflect::set(&config, &"maxLength".into(), &8.into()).unwrap();
    js_sys::Reflect::set(&config, &"padding".into(), &"fixed_length".into()).unwrap();
    js_sys::Reflect::set(&config, &"padTokenId".into(), &0.into()).unwrap();
    js_sys::Reflect::set(&config, &"kind".into(), &"protein".into()).unwrap();
    js_sys::Reflect::set(&config, &"profile".into(), &"protein-20".into()).unwrap();

    let result = biors_wasm::run_workflow(config.into());
    let output = result.unwrap();
    let model_ready = js_sys::Reflect::get(&output, &"model_ready".into())
        .unwrap()
        .as_bool()
        .unwrap();
    assert!(model_ready);
    let model_input = js_sys::Reflect::get(&output, &"model_input".into()).unwrap();
    let records = js_sys::Reflect::get(&model_input, &"records".into()).unwrap();
    let first_record = js_sys::Array::from(&records).get(0);
    let input_ids =
        js_sys::Array::from(&js_sys::Reflect::get(&first_record, &"input_ids".into()).unwrap());
    assert_eq!(input_ids.length(), 8);

    let provenance = js_sys::Reflect::get(&output, &"provenance".into()).unwrap();
    let input_hash = js_sys::Reflect::get(&provenance, &"input_hash".into())
        .unwrap()
        .as_string()
        .unwrap();
    assert!(input_hash.starts_with("fnv1a64:"));
    let tokenizer = js_sys::Reflect::get(&provenance, &"tokenizer".into()).unwrap();
    let tokenizer_name = js_sys::Reflect::get(&tokenizer, &"name".into())
        .unwrap()
        .as_string()
        .unwrap();
    assert_eq!(tokenizer_name, "protein-20");

    assert_matches_shared_workflow_schema_contract(&output, WORKFLOW_OUTPUT_SCHEMA);
}

#[wasm_bindgen_test]
fn test_run_workflow_accepts_nucleotide_kind_and_profile() {
    let config = workflow_config(">seq1\nACGT\n");
    js_sys::Reflect::set(&config, &"maxLength".into(), &6.into()).unwrap();
    js_sys::Reflect::set(&config, &"kind".into(), &"dna".into()).unwrap();
    js_sys::Reflect::set(&config, &"profile".into(), &"dna-iupac".into()).unwrap();
    js_sys::Reflect::set(&config, &"padding".into(), &"fixed_length".into()).unwrap();
    let output = biors_wasm::run_workflow(config.into()).unwrap();
    let workflow = js_sys::Reflect::get(&output, &"workflow".into())
        .unwrap()
        .as_string()
        .unwrap();
    assert_eq!(workflow, "sequence_model_input.v0");
    let provenance = js_sys::Reflect::get(&output, &"provenance".into()).unwrap();
    let tokenizer = js_sys::Reflect::get(&provenance, &"tokenizer".into()).unwrap();
    let tokenizer_name = js_sys::Reflect::get(&tokenizer, &"name".into())
        .unwrap()
        .as_string()
        .unwrap();
    assert_eq!(tokenizer_name, "dna-iupac");

    let config = workflow_config(">seq1\nACGT\n");
    js_sys::Reflect::set(&config, &"maxLength".into(), &8.into()).unwrap();
    js_sys::Reflect::set(&config, &"kind".into(), &"auto".into()).unwrap();
    let output = biors_wasm::run_workflow(config.into()).unwrap();
    let provenance = js_sys::Reflect::get(&output, &"provenance".into()).unwrap();
    let tokenizer = js_sys::Reflect::get(&provenance, &"tokenizer".into()).unwrap();
    let tokenizer_name = js_sys::Reflect::get(&tokenizer, &"name".into())
        .unwrap()
        .as_string()
        .unwrap();
    assert_eq!(tokenizer_name, "dna-iupac");
}

#[wasm_bindgen_test]
fn test_run_workflow_rejects_kind_profile_mismatch() {
    let config = workflow_config(">seq1\nACGT\n");
    js_sys::Reflect::set(&config, &"maxLength".into(), &8.into()).unwrap();
    js_sys::Reflect::set(&config, &"kind".into(), &"dna".into()).unwrap();
    js_sys::Reflect::set(&config, &"profile".into(), &"protein-20".into()).unwrap();
    assert!(biors_wasm::run_workflow(config.into()).is_err());
}

#[wasm_bindgen_test]
fn test_run_workflow_accepts_special_profiles() {
    let config = workflow_config(">seq1\nACDE\n");
    js_sys::Reflect::set(&config, &"maxLength".into(), &8.into()).unwrap();
    js_sys::Reflect::set(&config, &"profile".into(), &"protein-20-special".into()).unwrap();
    assert!(biors_wasm::run_workflow(config.into()).is_ok());
}

#[wasm_bindgen_test]
fn test_run_workflow_rejects_fractional_numeric_config() {
    let config = workflow_config(">seq1\nACDE\n");
    js_sys::Reflect::set(&config, &"maxLength".into(), &8.5.into()).unwrap();

    assert!(biors_wasm::run_workflow(config.into()).is_err());

    let config = workflow_config(">seq1\nACDE\n");
    js_sys::Reflect::set(&config, &"maxLength".into(), &8.into()).unwrap();
    js_sys::Reflect::set(&config, &"padTokenId".into(), &21.5.into()).unwrap();

    assert!(biors_wasm::run_workflow(config.into()).is_err());
}

#[wasm_bindgen_test]
fn test_run_workflow_accepts_missing_pad_token_id() {
    let config = workflow_config(">seq1\nACDE\n");
    js_sys::Reflect::set(&config, &"maxLength".into(), &8.into()).unwrap();
    js_sys::Reflect::set(&config, &"padding".into(), &"fixed_length".into()).unwrap();

    let result = biors_wasm::run_workflow(config.into());
    assert!(result.is_ok());
}

#[wasm_bindgen_test]
fn test_run_workflow_rejects_invalid_pad_token_id_values() {
    assert_pad_token_id_rejected(&"21".into());
    assert_pad_token_id_rejected(&(-1).into());
    assert_pad_token_id_rejected(&256.into());
}

fn assert_pad_token_id_rejected(value: &wasm_bindgen::JsValue) {
    let config = workflow_config(">seq1\nACDE\n");
    js_sys::Reflect::set(&config, &"maxLength".into(), &8.into()).unwrap();
    js_sys::Reflect::set(&config, &"padTokenId".into(), value).unwrap();

    let error = biors_wasm::run_workflow(config.into()).expect_err("padTokenId should fail");
    assert!(error
        .as_string()
        .expect("error message")
        .contains("field padTokenId must be an integer between 0 and 255"));
}

fn workflow_config(fasta: &str) -> js_sys::Object {
    let config = js_sys::Object::new();
    js_sys::Reflect::set(
        &config,
        &"fastaBytes".into(),
        &js_sys::Uint8Array::from(fasta.as_bytes()).into(),
    )
    .unwrap();
    config
}

use wasm_bindgen_test::*;

const WORKFLOW_OUTPUT_SCHEMA: &str =
    include_str!("../../../../schemas/sequence-workflow-output.v0.json");

#[wasm_bindgen_test]
fn test_parse_fasta() {
    let fasta = ">seq1\nACDE\n>seq2\nFGHI\n";
    let bytes = fasta.as_bytes();
    let result = biors_wasm::parse_fasta(bytes);
    assert!(result.is_ok());
}

#[wasm_bindgen_test]
fn test_validate_fasta_protein() {
    let fasta = ">seq1\nACDE\n";
    let bytes = fasta.as_bytes();
    let result = biors_wasm::validate_fasta(bytes, "protein".to_string());
    assert!(result.is_ok());
}

#[wasm_bindgen_test]
fn test_validate_fasta_auto() {
    let fasta = ">seq1\nACDE\n";
    let bytes = fasta.as_bytes();
    let result = biors_wasm::validate_fasta(bytes, "auto".to_string());
    assert!(result.is_ok());
}

#[wasm_bindgen_test]
fn test_validate_fasta_invalid_kind() {
    let fasta = ">seq1\nACDE\n";
    let bytes = fasta.as_bytes();
    let result = biors_wasm::validate_fasta(bytes, "invalid".to_string());
    assert!(result.is_err());
}

#[wasm_bindgen_test]
fn test_tokenize() {
    let records = js_sys::JSON::parse(r#"[{"id":"seq1","sequence":"ACDE"}]"#).unwrap();
    let result = biors_wasm::tokenize(records, "protein-20".to_string());
    assert!(result.is_ok());
}

#[wasm_bindgen_test]
fn test_tokenize_invalid_profile() {
    let records = js_sys::JSON::parse(r#"[{"id":"seq1","sequence":"ACDE"}]"#).unwrap();
    let result = biors_wasm::tokenize(records, "invalid".to_string());
    assert!(result.is_err());
}

#[wasm_bindgen_test]
fn test_build_model_input() {
    let tokenized = js_sys::JSON::parse(r#"[{"id":"seq1","tokens":[0,1,2,3],"length":4,"alphabet":"protein-20","valid":true,"warnings":[],"errors":[]}]"#).unwrap();
    let result = biors_wasm::build_model_input(tokenized, 8);
    assert!(result.is_ok());
}

#[wasm_bindgen_test]
fn test_build_model_input_with_policy() {
    let tokenized = js_sys::JSON::parse(r#"[{"id":"seq1","tokens":[0,1,2,3],"length":4,"alphabet":"protein-20","valid":true,"warnings":[],"errors":[]}]"#).unwrap();
    let result =
        biors_wasm::build_model_input_with_policy(tokenized, 8, 21, "fixed_length".to_string());
    assert!(result.is_ok());
}

#[wasm_bindgen_test]
fn test_build_model_input_rejects_empty_token_sequence() {
    let tokenized = js_sys::JSON::parse(
        r#"[{"id":"empty","tokens":[],"length":0,"alphabet":"protein-20","valid":true,"warnings":[],"errors":[]}]"#,
    )
    .unwrap();
    let error = biors_wasm::build_model_input(tokenized, 8).expect_err("empty tokens fail");
    assert!(error.as_string().unwrap().contains("empty"));
}

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
fn test_run_workflow_rejects_unsupported_kind_and_profile() {
    let config = workflow_config(">seq1\nACGT\n");
    js_sys::Reflect::set(&config, &"maxLength".into(), &8.into()).unwrap();
    js_sys::Reflect::set(&config, &"kind".into(), &"dna".into()).unwrap();
    assert!(biors_wasm::run_workflow(config.into()).is_err());

    let config = workflow_config(">seq1\nACGT\n");
    js_sys::Reflect::set(&config, &"maxLength".into(), &8.into()).unwrap();
    js_sys::Reflect::set(&config, &"kind".into(), &"auto".into()).unwrap();
    assert!(biors_wasm::run_workflow(config.into()).is_err());

    let config = workflow_config(">seq1\nACDE\n");
    js_sys::Reflect::set(&config, &"maxLength".into(), &8.into()).unwrap();
    js_sys::Reflect::set(&config, &"profile".into(), &"protein-20-special".into()).unwrap();
    assert!(biors_wasm::run_workflow(config.into()).is_err());
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

fn assert_matches_shared_workflow_schema_contract(
    output: &wasm_bindgen::JsValue,
    schema_json: &str,
) {
    let output_json = js_sys::JSON::stringify(output)
        .unwrap()
        .as_string()
        .unwrap();
    let value: serde_json::Value = serde_json::from_str(&output_json).unwrap();
    let schema: serde_json::Value = serde_json::from_str(schema_json).unwrap();

    let properties = schema["properties"].as_object().unwrap();
    let required = schema["required"].as_array().unwrap();
    let value_object = value.as_object().unwrap();
    for key in required {
        assert!(
            value_object.contains_key(key.as_str().unwrap()),
            "WASM output missing required schema key {key}"
        );
    }
    for key in value_object.keys() {
        assert!(
            properties.contains_key(key),
            "WASM output has non-schema key {key}"
        );
    }

    assert_eq!(value["workflow"], schema["properties"]["workflow"]["const"]);
    let allowed_commands = schema["properties"]["provenance"]["properties"]["invocation"]
        ["properties"]["command"]["enum"]
        .as_array()
        .unwrap();
    assert!(
        allowed_commands.contains(&value["provenance"]["invocation"]["command"]),
        "WASM workflow command is outside the shared schema enum"
    );
    assert_eq!(
        value["provenance"]["normalization"],
        schema["properties"]["provenance"]["properties"]["normalization"]["const"]
    );
    assert_eq!(
        value["provenance"]["validation_alphabet"],
        schema["properties"]["provenance"]["properties"]["validation_alphabet"]["const"]
    );
    assert_eq!(
        value["provenance"]["tokenizer"]["name"],
        schema["properties"]["provenance"]["properties"]["tokenizer"]["properties"]["name"]
            ["const"]
    );
    assert_eq!(
        value["validation"]["sequences"][0]["alphabet"],
        schema["properties"]["validation"]["properties"]["sequences"]["items"]["properties"]
            ["alphabet"]["const"]
    );
}

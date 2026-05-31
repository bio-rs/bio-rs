use wasm_bindgen_test::*;

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
fn test_run_workflow() {
    let config = js_sys::Object::new();
    js_sys::Reflect::set(
        &config,
        &"fastaBytes".into(),
        &js_sys::Uint8Array::from(">seq1\nACDE\n".as_bytes()).into(),
    )
    .unwrap();
    js_sys::Reflect::set(&config, &"maxLength".into(), &8.into()).unwrap();
    js_sys::Reflect::set(&config, &"padding".into(), &"fixed_length".into()).unwrap();
    js_sys::Reflect::set(&config, &"padTokenId".into(), &0.into()).unwrap();

    let result = biors_wasm::run_workflow(config.into());
    let output = result.unwrap();
    let provenance = js_sys::Reflect::get(&output, &"provenance".into()).unwrap();
    let input_hash = js_sys::Reflect::get(&provenance, &"input_hash".into())
        .unwrap()
        .as_string()
        .unwrap();
    assert!(input_hash.starts_with("fnv1a64:"));
}

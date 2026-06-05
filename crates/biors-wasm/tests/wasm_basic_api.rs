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
fn test_tokenize_accepts_nucleotide_profiles() {
    let records = js_sys::JSON::parse(r#"[{"id":"dna","sequence":"ACGT"}]"#).unwrap();
    let result = biors_wasm::tokenize(records, "dna-iupac".to_string()).unwrap();
    let records = js_sys::Reflect::get(&result, &"records".into()).unwrap();
    let first_record = js_sys::Array::from(&records).get(0);
    let alphabet = js_sys::Reflect::get(&first_record, &"alphabet".into())
        .unwrap()
        .as_string()
        .unwrap();
    assert_eq!(alphabet, "dna-iupac");
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

use wasm_bindgen::JsValue;
use wasm_bindgen_test::*;

#[wasm_bindgen_test]
fn browser_policy_is_local_and_bounded() {
    let policy = biors_wasm::browser_execution_policy().expect("policy");
    assert_eq!(string_field(&policy, "network_access"), "none");
    assert_eq!(
        bool_field(&policy, "uploads_input_data"),
        Some(false),
        "browser helpers must not upload data"
    );
    assert_eq!(
        bool_field(&policy, "external_model_calls"),
        Some(false),
        "browser helpers must not call external models"
    );
    assert!(number_field(&policy, "max_input_bytes") >= 64.0 * 1024.0 * 1024.0);
}

#[wasm_bindgen_test]
fn browser_file_inspection_infers_fasta_and_hashes_content() {
    let input = browser_input("protein.fasta", None, b">seq1\nACDE\n");
    let inspected = biors_wasm::inspect_browser_file(input).expect("inspection");
    let file = js_sys::Reflect::get(&inspected, &"file".into()).unwrap();

    assert_eq!(string_field(&file, "format"), "fasta");
    assert!(string_field(&file, "content_sha256").starts_with("sha256:"));
}

#[wasm_bindgen_test]
fn browser_validation_accepts_supported_formats() {
    let fasta = biors_wasm::validate_browser_file(browser_input(
        "protein.fasta",
        Some("fasta"),
        b">seq1\nACDE\n",
    ));
    assert!(fasta.is_ok());

    let fastq = biors_wasm::validate_browser_file(browser_input(
        "reads.fastq",
        None,
        b"@r1\nACGT\n+\n!!!!\n",
    ));
    assert!(fastq.is_ok());

    let pdb = biors_wasm::validate_browser_file(browser_input(
        "structure.pdb",
        None,
        b"ATOM      1  N   MET A   1      11.104  13.207   9.701  1.00 20.00           N\nEND\n",
    ));
    assert!(pdb.is_ok());

    let smiles =
        biors_wasm::validate_browser_file(browser_input("molecules.smi", None, b"CCO ethanol\n"));
    assert!(smiles.is_ok());
}

#[wasm_bindgen_test]
fn browser_tokenization_returns_fasta_tokens() {
    let input = browser_input("protein.fasta", Some("fasta"), b">seq1\nACDE\n");
    js_sys::Reflect::set(&input, &"profile".into(), &"protein-20".into()).unwrap();
    let output = biors_wasm::tokenize_browser_file(input).expect("tokenized");
    let tokenization = js_sys::Reflect::get(&output, &"tokenization".into()).unwrap();
    let ids = js_sys::Array::from(&js_sys::Reflect::get(&tokenization, &"ids".into()).unwrap());
    assert_eq!(ids.length(), 1);
    assert_eq!(ids.get(0).as_string().as_deref(), Some("seq1"));
}

#[wasm_bindgen_test]
fn browser_tokenization_rejects_non_fasta_formats() {
    let input = browser_input("reads.fastq", Some("fastq"), b"@r1\nACGT\n+\n!!!!\n");
    let error = biors_wasm::tokenize_browser_file(input).expect_err("FASTQ tokenization fails");
    assert!(error.as_string().unwrap().contains("only FASTA"));
}

fn browser_input(name: &str, format: Option<&str>, bytes: &[u8]) -> JsValue {
    let input = js_sys::Object::new();
    js_sys::Reflect::set(&input, &"name".into(), &name.into()).unwrap();
    if let Some(format) = format {
        js_sys::Reflect::set(&input, &"format".into(), &format.into()).unwrap();
    }
    js_sys::Reflect::set(
        &input,
        &"bytes".into(),
        &js_sys::Uint8Array::from(bytes).into(),
    )
    .unwrap();
    input.into()
}

fn string_field(value: &JsValue, field: &str) -> String {
    js_sys::Reflect::get(value, &field.into())
        .unwrap()
        .as_string()
        .unwrap()
}

fn bool_field(value: &JsValue, field: &str) -> Option<bool> {
    js_sys::Reflect::get(value, &field.into())
        .unwrap()
        .as_bool()
}

fn number_field(value: &JsValue, field: &str) -> f64 {
    js_sys::Reflect::get(value, &field.into())
        .unwrap()
        .as_f64()
        .unwrap()
}

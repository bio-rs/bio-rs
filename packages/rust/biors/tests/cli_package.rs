use biors_core::hash::sha256_bytes_digest;
use serde_json::Value;
use std::fs;
use std::path::Path;
use std::process::{Command, Stdio};

mod common;
use common::ChildInputExt;

#[test]
fn package_inspect_outputs_manifest_summary() {
    let manifest = common::repo_root().join("examples/protein-package/manifest.json");

    let output = Command::new(env!("CARGO_BIN_EXE_biors"))
        .arg("package")
        .arg("inspect")
        .arg(manifest)
        .output()
        .expect("run biors package inspect");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let value: Value = serde_json::from_slice(&output.stdout).expect("valid JSON output");

    assert_eq!(value["data"]["schema_version"], "biors.package.v1");
    assert_eq!(value["data"]["name"], "protein-seed");
    assert_eq!(value["data"]["package_layout"]["models"], "models");
    assert_eq!(value["data"]["package_layout"]["pipelines"], "pipelines");
    assert_eq!(value["data"]["package_layout"]["docs"], "docs");
    assert_eq!(value["data"]["metadata"]["license"], "CC0-1.0");
    assert_eq!(
        value["data"]["metadata"]["model_card"],
        "docs/model-card.md"
    );
    assert_eq!(value["data"]["model_format"], "onnx");
    assert_eq!(value["data"]["has_model_checksum"], true);
    assert_eq!(value["data"]["runtime_backend"], "onnx-webgpu");
    assert_eq!(value["data"]["runtime_target"], "browser-wasm-webgpu");
    assert_eq!(value["data"]["fixtures"], 1);
    assert_eq!(value["data"]["layout"]["model"], "models/protein-seed.onnx");
    assert_eq!(
        value["data"]["layout"]["tokenizer"],
        "tokenizers/protein-20.json"
    );
    assert_eq!(value["data"]["layout"]["vocab"], "vocabs/protein-20.json");
    assert_eq!(
        value["data"]["layout"]["fixture_inputs"][0],
        "fixtures/tiny.fasta"
    );
    assert_eq!(
        value["data"]["layout"]["fixture_outputs"][0],
        "fixtures/tiny.output.json"
    );
    assert_eq!(
        value["data"]["layout"]["pipeline_configs"][0],
        "pipelines/protein.toml"
    );
}

#[test]
fn package_validate_fails_invalid_manifest() {
    let output = Command::new(env!("CARGO_BIN_EXE_biors"))
        .arg("package")
        .arg("validate")
        .arg("--json")
        .arg("-")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn biors package validate")
        .tap_stdin(
            r#"{
              "schema_version": "biors.package.v0",
              "name": "",
              "model": { "format": "onnx", "path": "" },
              "preprocessing": [],
              "postprocessing": [],
              "runtime": {
                "backend": "onnx-webgpu",
                "target": "browser-wasm-webgpu"
              },
              "fixtures": []
            }"#,
        );

    assert!(
        !output.status.success(),
        "expected validation failure, stdout: {}",
        String::from_utf8_lossy(&output.stdout)
    );

    assert!(output.stderr.is_empty());

    let value: Value = serde_json::from_slice(&output.stdout).expect("valid JSON error");

    assert_eq!(value["ok"], false);
    assert_eq!(value["error"]["code"], "package.validation_failed");
    assert_eq!(value["error"]["location"], "manifest");
    assert!(value["error"]["message"]
        .as_str()
        .unwrap()
        .contains("name is required"));
    assert_eq!(
        value["error"]["details"]["structured_issues"][0]["code"],
        "required_field"
    );
}

#[test]
fn package_validate_rejects_unknown_manifest_fields() {
    let output = Command::new(env!("CARGO_BIN_EXE_biors"))
        .arg("--json")
        .arg("package")
        .arg("validate")
        .arg("-")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn biors package validate")
        .tap_stdin(
            r#"{
              "schema_version": "biors.package.v0",
              "name": "protein-seed",
              "unexpected_top": true,
              "model": { "format": "onnx", "path": "model.onnx" },
              "preprocessing": [],
              "postprocessing": [],
              "runtime": {
                "backend": "onnx-webgpu",
                "target": "browser-wasm-webgpu"
              },
              "fixtures": [
                {
                  "name": "tiny-protein",
                  "input": "fixtures/tiny.fasta",
                  "expected_output": "fixtures/tiny.output.json"
                }
              ]
            }"#,
        );

    assert_eq!(output.status.code(), Some(2));
    assert!(output.stderr.is_empty());

    let value: Value = serde_json::from_slice(&output.stdout).expect("valid JSON error");
    assert_eq!(value["error"]["code"], "json.invalid");
    assert!(value["error"]["message"]
        .as_str()
        .expect("message")
        .contains("unknown field"));
}

#[test]
fn package_validate_rejects_empty_contract_identifiers() {
    let source_package = common::repo_root().join("examples/protein-package");
    let temp = common::TempDir::new("empty-contract-package");
    copy_dir_all(&source_package, temp.path());
    let manifest_path = temp.path().join("manifest.json");
    let mut manifest: Value =
        serde_json::from_str(&fs::read_to_string(&manifest_path).expect("read manifest"))
            .expect("manifest JSON");
    manifest["tokenizer"]["name"] = Value::String(String::new());
    manifest["tokenizer"]["contract_version"] = Value::String(String::new());
    manifest["vocab"]["name"] = Value::String(String::new());
    manifest["vocab"]["contract_version"] = Value::String(String::new());
    manifest["preprocessing"][0]["name"] = Value::String(String::new());
    manifest["preprocessing"][0]["implementation"] = Value::String(String::new());
    manifest["preprocessing"][0]["contract"] = Value::String(String::new());
    manifest["preprocessing"][0]["contract_version"] = Value::String(String::new());
    fs::write(&manifest_path, manifest.to_string()).expect("write manifest");

    let output = Command::new(env!("CARGO_BIN_EXE_biors"))
        .arg("--json")
        .arg("package")
        .arg("validate")
        .arg(&manifest_path)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("run biors package validate");

    assert_eq!(output.status.code(), Some(2));
    assert!(output.stderr.is_empty());

    let value: Value = serde_json::from_slice(&output.stdout).expect("valid JSON error");
    assert_eq!(value["error"]["code"], "package.validation_failed");
    let issues = value["error"]["details"]["structured_issues"]
        .as_array()
        .expect("structured issues");
    let fields: Vec<_> = issues
        .iter()
        .filter(|issue| issue["code"] == "required_field")
        .map(|issue| issue["field"].as_str().expect("field"))
        .collect();
    for field in [
        "tokenizer.name",
        "tokenizer.contract_version",
        "vocab.name",
        "vocab.contract_version",
        "preprocessing[0].name",
        "preprocessing[0].implementation",
        "preprocessing[0].contract",
        "preprocessing[0].contract_version",
    ] {
        assert!(fields.contains(&field), "missing {field}: {fields:?}");
    }
}

#[test]
fn package_validate_rejects_pipeline_config_with_zero_max_length() {
    let value = validate_package_with_pipeline_config(
        "invalid-pipeline-max-length",
        r#"schema_version = "biors.pipeline.v0"
name = "protein-package-fixture-pipeline"

[input]
format = "fasta"
path = "../fixtures/tiny.fasta"

[normalize]
policy = "strip_ascii_whitespace_uppercase"

[validate]
kind = "protein"

[tokenize]
profile = "protein-20"

[export]
format = "model-input-json"
max_length = 0
pad_token_id = 0
padding = "fixed_length"
"#,
    );

    assert_eq!(value["error"]["code"], "package.invalid_pipeline_config");
    assert_invalid_pipeline_config_issue(&value, "export.max_length");
}

#[test]
fn package_validate_rejects_pipeline_config_with_invalid_padding() {
    let value = validate_package_with_pipeline_config(
        "invalid-pipeline-padding",
        r#"schema_version = "biors.pipeline.v0"
name = "protein-package-fixture-pipeline"

[input]
format = "fasta"
path = "../fixtures/tiny.fasta"

[normalize]
policy = "strip_ascii_whitespace_uppercase"

[validate]
kind = "protein"

[tokenize]
profile = "protein-20"

[export]
format = "model-input-json"
max_length = 8
pad_token_id = 0
padding = "left"
"#,
    );

    assert_eq!(value["error"]["code"], "package.invalid_pipeline_config");
    assert_invalid_pipeline_config_issue(&value, "export.padding");
}

#[test]
fn package_validate_rejects_pipeline_config_with_unknown_field() {
    let value = validate_package_with_pipeline_config(
        "invalid-pipeline-unknown-field",
        r#"schema_version = "biors.pipeline.v0"
name = "protein-package-fixture-pipeline"
unexpected = true

[input]
format = "fasta"
path = "../fixtures/tiny.fasta"

[normalize]
policy = "strip_ascii_whitespace_uppercase"

[validate]
kind = "protein"

[tokenize]
profile = "protein-20"

[export]
format = "model-input-json"
max_length = 8
pad_token_id = 0
padding = "fixed_length"
"#,
    );

    assert_eq!(value["error"]["code"], "package.invalid_pipeline_config");
    assert_invalid_pipeline_config_issue(&value, "unknown field");
}

#[test]
fn package_validate_rejects_tokenizer_config_with_unknown_profile() {
    let value = validate_package_with_tokenizer_config(
        "invalid-tokenizer-profile",
        r#"{
  "profile": "bad",
  "add_special_tokens": false
}"#,
        None,
    );

    assert_eq!(value["error"]["code"], "package.invalid_tokenizer_config");
    assert_invalid_tokenizer_config_issue(&value, "unknown variant");
}

#[test]
fn package_validate_rejects_tokenizer_config_with_invalid_json_type() {
    let value = validate_package_with_tokenizer_config(
        "invalid-tokenizer-json-type",
        r#"{
  "profile": "protein-20",
  "add_special_tokens": "yes"
}"#,
        None,
    );

    assert_eq!(value["error"]["code"], "package.invalid_tokenizer_config");
    assert_invalid_tokenizer_config_issue(&value, "invalid tokenizer config JSON");
}

#[test]
fn package_validate_rejects_tokenizer_config_with_invalid_special_policy() {
    let value = validate_package_with_tokenizer_config(
        "invalid-tokenizer-special-policy",
        r#"{
  "profile": "protein-20-special",
  "add_special_tokens": false
}"#,
        Some(("protein-20-special", "protein-20-special.v0")),
    );

    assert_eq!(value["error"]["code"], "package.invalid_tokenizer_config");
    assert_invalid_tokenizer_config_issue(&value, "add_special_tokens");
}

#[test]
fn package_validate_rejects_tokenizer_manifest_profile_mismatch() {
    let value = validate_package_with_tokenizer_config(
        "invalid-tokenizer-manifest-mismatch",
        r#"{
  "profile": "protein-20-special",
  "add_special_tokens": true
}"#,
        None,
    );

    assert_eq!(value["error"]["code"], "package.invalid_tokenizer_config");
    assert_invalid_tokenizer_config_issue(&value, "tokenizer.name must match");
    assert_invalid_tokenizer_config_issue(&value, "tokenizer.contract_version must match");
}

#[test]
fn package_validate_rejects_vocab_with_string_tokens() {
    let value = validate_package_with_vocab(
        "invalid-vocab-string-tokens",
        r#"{
  "name": "protein-20",
  "unknown_token_id": 20,
  "tokens": ["A", "C"]
}"#,
        None,
    );

    assert_eq!(value["error"]["code"], "package.invalid_vocab_config");
    assert_invalid_vocab_config_issue(&value, "invalid vocabulary JSON");
}

#[test]
fn package_validate_rejects_vocab_manifest_contract_mismatch() {
    let vocab = valid_vocab_json();
    let value = validate_package_with_vocab(
        "invalid-vocab-contract-mismatch",
        &vocab,
        Some(("protein-20-alt", "protein-20-alt.v0")),
    );

    assert_eq!(value["error"]["code"], "package.invalid_vocab_config");
    assert_invalid_vocab_config_issue(&value, "vocab.name must match");
    assert_invalid_vocab_config_issue(&value, "vocab.contract_version must match");
}

#[test]
fn package_validate_rejects_vocab_with_wrong_token_order() {
    let value = validate_package_with_vocab(
        "invalid-vocab-token-order",
        &valid_vocab_json().replace(r#""token_id": 1"#, r#""token_id": 2"#),
        None,
    );

    assert_eq!(value["error"]["code"], "package.invalid_vocab_config");
    assert_invalid_vocab_config_issue(&value, "token order and IDs");
}

fn copy_dir_all(source: &Path, destination: &Path) {
    fs::create_dir_all(destination).expect("create destination");
    for entry in fs::read_dir(source).expect("read source directory") {
        let entry = entry.expect("read source entry");
        let file_type = entry.file_type().expect("read source file type");
        let destination_path = destination.join(entry.file_name());
        if file_type.is_dir() {
            copy_dir_all(&entry.path(), &destination_path);
        } else {
            fs::copy(entry.path(), destination_path).expect("copy package fixture file");
        }
    }
}

fn validate_package_with_pipeline_config(name: &str, config: &str) -> Value {
    let source_package = common::repo_root().join("examples/protein-package");
    let temp = common::TempDir::new(name);
    copy_dir_all(&source_package, temp.path());
    let config_path = temp.path().join("pipelines/protein.toml");
    fs::write(&config_path, config).expect("write pipeline config");

    let manifest_path = temp.path().join("manifest.json");
    let mut manifest: Value =
        serde_json::from_str(&fs::read_to_string(&manifest_path).expect("read manifest"))
            .expect("manifest JSON");
    let config_bytes = fs::read(&config_path).expect("read pipeline config");
    manifest["preprocessing"][0]["config"]["checksum"] =
        Value::String(sha256_bytes_digest(&config_bytes));
    fs::write(&manifest_path, manifest.to_string()).expect("write manifest");

    let output = Command::new(env!("CARGO_BIN_EXE_biors"))
        .arg("--json")
        .arg("package")
        .arg("validate")
        .arg(&manifest_path)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("run biors package validate");

    assert_eq!(output.status.code(), Some(2));
    assert!(output.stderr.is_empty());
    serde_json::from_slice(&output.stdout).expect("valid JSON error")
}

fn validate_package_with_tokenizer_config(
    name: &str,
    config: &str,
    manifest_identity: Option<(&str, &str)>,
) -> Value {
    let source_package = common::repo_root().join("examples/protein-package");
    let temp = common::TempDir::new(name);
    copy_dir_all(&source_package, temp.path());
    let config_path = temp.path().join("tokenizers/protein-20.json");
    fs::write(&config_path, config).expect("write tokenizer config");

    let manifest_path = temp.path().join("manifest.json");
    let mut manifest: Value =
        serde_json::from_str(&fs::read_to_string(&manifest_path).expect("read manifest"))
            .expect("manifest JSON");
    let config_bytes = fs::read(&config_path).expect("read tokenizer config");
    manifest["tokenizer"]["checksum"] = Value::String(sha256_bytes_digest(&config_bytes));
    if let Some((name, contract_version)) = manifest_identity {
        manifest["tokenizer"]["name"] = Value::String(name.to_string());
        manifest["tokenizer"]["contract_version"] = Value::String(contract_version.to_string());
    }
    fs::write(&manifest_path, manifest.to_string()).expect("write manifest");

    let output = Command::new(env!("CARGO_BIN_EXE_biors"))
        .arg("--json")
        .arg("package")
        .arg("validate")
        .arg(&manifest_path)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("run biors package validate");

    assert_eq!(output.status.code(), Some(2));
    assert!(output.stderr.is_empty());
    serde_json::from_slice(&output.stdout).expect("valid JSON error")
}

fn validate_package_with_vocab(
    name: &str,
    vocab: &str,
    manifest_identity: Option<(&str, &str)>,
) -> Value {
    let source_package = common::repo_root().join("examples/protein-package");
    let temp = common::TempDir::new(name);
    copy_dir_all(&source_package, temp.path());
    let vocab_path = temp.path().join("vocabs/protein-20.json");
    fs::write(&vocab_path, vocab).expect("write vocab");

    let manifest_path = temp.path().join("manifest.json");
    let mut manifest: Value =
        serde_json::from_str(&fs::read_to_string(&manifest_path).expect("read manifest"))
            .expect("manifest JSON");
    let vocab_bytes = fs::read(&vocab_path).expect("read vocab");
    manifest["vocab"]["checksum"] = Value::String(sha256_bytes_digest(&vocab_bytes));
    if let Some((name, contract_version)) = manifest_identity {
        manifest["vocab"]["name"] = Value::String(name.to_string());
        manifest["vocab"]["contract_version"] = Value::String(contract_version.to_string());
    }
    fs::write(&manifest_path, manifest.to_string()).expect("write manifest");

    let output = Command::new(env!("CARGO_BIN_EXE_biors"))
        .arg("--json")
        .arg("package")
        .arg("validate")
        .arg(&manifest_path)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("run biors package validate");

    assert_eq!(output.status.code(), Some(2));
    assert!(output.stderr.is_empty());
    serde_json::from_slice(&output.stdout).expect("valid JSON error")
}

fn valid_vocab_json() -> String {
    fs::read_to_string(common::repo_root().join("examples/protein-package/vocabs/protein-20.json"))
        .expect("read example vocab")
}

fn assert_invalid_pipeline_config_issue(value: &Value, expected_message: &str) {
    let issues = value["error"]["details"]["structured_issues"]
        .as_array()
        .expect("structured issues");
    assert!(issues.iter().any(|issue| {
        issue["code"] == "invalid_pipeline_config"
            && issue["field"] == "preprocessing[0].config"
            && issue["message"]
                .as_str()
                .expect("message")
                .contains(expected_message)
    }));
}

fn assert_invalid_tokenizer_config_issue(value: &Value, expected_message: &str) {
    let issues = value["error"]["details"]["structured_issues"]
        .as_array()
        .expect("structured issues");
    assert!(issues.iter().any(|issue| {
        issue["code"] == "invalid_tokenizer_config"
            && issue["message"]
                .as_str()
                .expect("message")
                .contains(expected_message)
    }));
}

fn assert_invalid_vocab_config_issue(value: &Value, expected_message: &str) {
    let issues = value["error"]["details"]["structured_issues"]
        .as_array()
        .expect("structured issues");
    assert!(issues.iter().any(|issue| {
        issue["code"] == "invalid_vocab_config"
            && issue["message"]
                .as_str()
                .expect("message")
                .contains(expected_message)
    }));
}

#[test]
fn package_validate_reports_invalid_checksum_format_code() {
    let output = Command::new(env!("CARGO_BIN_EXE_biors"))
        .arg("--json")
        .arg("package")
        .arg("validate")
        .arg("-")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn biors package validate")
        .tap_stdin(
            r#"{
              "schema_version": "biors.package.v0",
              "name": "protein-seed",
              "model": {
                "format": "onnx",
                "path": "missing.onnx",
                "checksum": "draft-model-checksum"
              },
              "preprocessing": [],
              "postprocessing": [],
              "runtime": {
                "backend": "onnx-webgpu",
                "target": "browser-wasm-webgpu"
              },
              "fixtures": [
                {
                  "name": "tiny-protein",
                  "input": "fixtures/tiny.fasta",
                  "expected_output": "fixtures/tiny.output.json"
                }
              ]
            }"#,
        );

    assert_eq!(output.status.code(), Some(2));
    assert!(output.stderr.is_empty());

    let value: Value = serde_json::from_slice(&output.stdout).expect("valid JSON error");
    assert_eq!(value["error"]["code"], "package.invalid_checksum_format");
    assert_eq!(
        value["error"]["details"]["structured_issues"][0]["code"],
        "invalid_checksum_format"
    );
    assert_eq!(
        value["error"]["details"]["structured_issues"][0]["field"],
        "model.checksum"
    );
    assert!(!value["error"]["message"]
        .as_str()
        .expect("message")
        .starts_with('['));
}

#[test]
fn package_bridge_outputs_runtime_plan() {
    let manifest = common::repo_root().join("examples/protein-package/manifest.json");

    let output = Command::new(env!("CARGO_BIN_EXE_biors"))
        .arg("package")
        .arg("bridge")
        .arg(manifest)
        .output()
        .expect("run biors package bridge");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let value: Value = serde_json::from_slice(&output.stdout).expect("valid JSON output");

    assert_eq!(value["data"]["ready"], true);
    assert_eq!(value["data"]["contract_ready"], true);
    assert_eq!(value["data"]["artifact_checked"], false);
    assert_eq!(value["data"]["execution_ready"], false);
    assert!(value["data"]["readiness_notes"][0]
        .as_str()
        .expect("readiness note")
        .contains("not format-validated"));
    assert_eq!(value["data"]["backend"], "onnx-webgpu");
    assert_eq!(value["data"]["target"], "browser-wasm-webgpu");
    assert_eq!(value["data"]["model_format"], "onnx");
    assert_eq!(
        value["data"]["model_metadata"]["name"],
        "protein-seed-linear-probe"
    );
    assert_eq!(
        value["data"]["backend_config"]["backend_id"],
        "protein-seed:onnx-webgpu"
    );
    assert_eq!(value["data"]["backend_config"]["provider"], "webgpu");
    assert_eq!(value["data"]["backend_config"]["version"], "onnx-webgpu.v0");
    assert_eq!(
        value["data"]["backend_config"]["model_artifact"],
        "models/protein-seed.onnx"
    );
    assert_eq!(value["data"]["execution_provider"], "webgpu");
    assert_eq!(
        value["data"]["compatibility_checks"][0]["code"],
        "runtime_model_pair"
    );
    assert_eq!(value["data"]["compatibility_checks"][0]["passed"], true);
    assert_eq!(
        value["data"]["blocking_issues"]
            .as_array()
            .expect("issues array")
            .len(),
        0
    );
}

#[test]
fn package_bridge_reports_structured_not_ready_details() {
    let output = Command::new(env!("CARGO_BIN_EXE_biors"))
        .arg("--json")
        .arg("package")
        .arg("bridge")
        .arg("-")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn biors package bridge")
        .tap_stdin(
            r#"{
              "schema_version": "biors.package.v0",
              "name": "protein-seed",
              "model": { "format": "onnx", "path": "missing.onnx" },
              "preprocessing": [],
              "postprocessing": [],
              "runtime": {
                "backend": "onnx-webgpu",
                "target": "browser-wasm-webgpu"
              },
              "fixtures": [
                {
                  "name": "tiny-protein",
                  "input": "fixtures/tiny.fasta",
                  "expected_output": "fixtures/tiny.output.json"
                }
              ]
            }"#,
        );

    assert_eq!(output.status.code(), Some(2));
    assert!(output.stderr.is_empty());

    let value: Value = serde_json::from_slice(&output.stdout).expect("valid JSON error");
    assert_eq!(value["error"]["code"], "package.bridge_not_ready");
    assert_eq!(value["error"]["location"], "manifest");
    assert_eq!(
        value["error"]["details"]["validation"]["structured_issues"][0]["code"],
        "asset_read_failed"
    );
    assert_eq!(value["error"]["details"]["bridge"]["ready"], true);
}

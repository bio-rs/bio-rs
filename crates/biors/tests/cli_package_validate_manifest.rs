use serde_json::Value;
use std::process::{Command, Stdio};

mod common;
mod package_support;
use common::ChildInputExt;

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
fn package_validate_accepts_declared_manifest_path() {
    let output = Command::new(env!("CARGO_BIN_EXE_biors"))
        .arg("package")
        .arg("validate")
        .arg(package_support::example_manifest_path())
        .output()
        .expect("run biors package validate");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let value: Value = serde_json::from_slice(&output.stdout).expect("valid JSON output");
    assert_eq!(value["data"]["valid"], true);
}

#[test]
fn package_validate_accepts_alternate_manifest_path_when_declared() {
    let temp = common::TempDir::new("alternate-manifest-path");
    package_support::copy_dir_all(&package_support::example_package_path(), temp.path());
    let original_manifest_path = temp.path().join("manifest.json");
    let alternate_manifest_path = temp.path().join("other-manifest.json");
    let mut manifest = package_support::read_manifest(&original_manifest_path);
    manifest["package_layout"]["manifest"] = Value::String("other-manifest.json".to_string());
    package_support::write_manifest(&alternate_manifest_path, &manifest);

    let output = Command::new(env!("CARGO_BIN_EXE_biors"))
        .arg("package")
        .arg("validate")
        .arg(&alternate_manifest_path)
        .output()
        .expect("run biors package validate");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let value: Value = serde_json::from_slice(&output.stdout).expect("valid JSON output");
    assert_eq!(value["data"]["valid"], true);
}

#[test]
fn package_validate_rejects_mismatched_declared_manifest_path() {
    let temp = common::TempDir::new("mismatched-manifest-path");
    package_support::copy_dir_all(&package_support::example_package_path(), temp.path());
    let manifest_path = temp.path().join("manifest.json");
    let mut manifest = package_support::read_manifest(&manifest_path);
    manifest["package_layout"]["manifest"] = Value::String("other-manifest.json".to_string());
    package_support::write_manifest(&manifest_path, &manifest);

    let output = package_support::run_package_validate(&manifest_path);

    assert_eq!(output.status.code(), Some(2));
    assert!(output.stderr.is_empty());
    let value: Value = serde_json::from_slice(&output.stdout).expect("valid JSON error");
    assert_eq!(value["error"]["code"], "package.layout_mismatch");
    assert!(value["error"]["details"]["structured_issues"]
        .as_array()
        .expect("structured issues")
        .iter()
        .any(|issue| {
            issue["code"] == "layout_mismatch" && issue["field"] == "package_layout.manifest"
        }));
}

#[test]
fn package_validate_rejects_empty_contract_identifiers() {
    let temp = common::TempDir::new("empty-contract-package");
    package_support::copy_dir_all(&package_support::example_package_path(), temp.path());
    let manifest_path = temp.path().join("manifest.json");
    let mut manifest = package_support::read_manifest(&manifest_path);
    manifest["tokenizer"]["name"] = Value::String(String::new());
    manifest["tokenizer"]["contract_version"] = Value::String(String::new());
    manifest["vocab"]["name"] = Value::String(String::new());
    manifest["vocab"]["contract_version"] = Value::String(String::new());
    manifest["preprocessing"][0]["name"] = Value::String(String::new());
    manifest["preprocessing"][0]["implementation"] = Value::String(String::new());
    manifest["preprocessing"][0]["contract"] = Value::String(String::new());
    manifest["preprocessing"][0]["contract_version"] = Value::String(String::new());
    package_support::write_manifest(&manifest_path, &manifest);

    let output = package_support::run_package_validate(&manifest_path);

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
fn package_validate_rejects_empty_shape_dimensions() {
    let temp = common::TempDir::new("empty-shape-dimension-package");
    package_support::copy_dir_all(&package_support::example_package_path(), temp.path());
    let manifest_path = temp.path().join("manifest.json");
    let mut manifest = package_support::read_manifest(&manifest_path);
    manifest["expected_input"]["shape"] = serde_json::json!(["", "256"]);
    manifest["expected_output"]["shape"] = serde_json::json!([" "]);
    package_support::write_manifest(&manifest_path, &manifest);

    let output = package_support::run_package_validate(&manifest_path);

    assert_eq!(output.status.code(), Some(2));
    assert!(output.stderr.is_empty());

    let value: Value = serde_json::from_slice(&output.stdout).expect("valid JSON error");
    assert_eq!(value["error"]["code"], "package.validation_failed");
    let issues = value["error"]["details"]["structured_issues"]
        .as_array()
        .expect("structured issues");
    let fields: Vec<_> = issues
        .iter()
        .map(|issue| issue["field"].as_str().expect("issue field"))
        .collect();
    assert!(fields.contains(&"expected_input.shape[0]"));
    assert!(fields.contains(&"expected_output.shape[0]"));
}

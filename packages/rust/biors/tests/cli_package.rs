use serde_json::Value;
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

use serde_json::Value;
use std::path::Path;
use std::process::{Command, Stdio};

mod common;
use common::ChildInputExt;

#[test]
fn package_inspect_outputs_manifest_summary() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let manifest = manifest_dir.join("../../../examples/protein-package/manifest.json");

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
        .expect("message")
        .contains("name is required"));
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
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let manifest = manifest_dir.join("../../../examples/protein-package/manifest.json");

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
    assert_eq!(value["data"]["execution_provider"], "webgpu");
    assert_eq!(
        value["data"]["blocking_issues"]
            .as_array()
            .expect("issues array")
            .len(),
        0
    );
}

#[test]
fn package_verify_outputs_fixture_report() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let manifest = manifest_dir.join("../../../examples/protein-package/manifest.json");
    let observations = manifest_dir.join("../../../examples/protein-package/observations.json");

    let output = Command::new(env!("CARGO_BIN_EXE_biors"))
        .arg("package")
        .arg("verify")
        .arg(manifest)
        .arg(observations)
        .output()
        .expect("run biors package verify");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let value: Value = serde_json::from_slice(&output.stdout).expect("valid JSON output");

    assert_eq!(value["data"]["package"], "protein-seed");
    assert_eq!(value["data"]["fixtures"], 1);
    assert_eq!(value["data"]["passed"], 1);
    assert_eq!(value["data"]["failed"], 0);
    assert_eq!(value["data"]["results"][0]["status"], "passed");
    assert_eq!(
        value["data"]["results"][0]["expected_output_path"],
        "fixtures/tiny.output.json"
    );
    assert_eq!(
        value["data"]["results"][0]["observed_output_path"],
        "observed/tiny.output.json"
    );
    assert_eq!(value["data"]["results"][0]["checksum_mismatch"], false);
    assert_eq!(value["data"]["results"][0]["content_mismatch"], false);
}

#[test]
fn package_verify_reports_missing_observed_output_code() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let manifest = manifest_dir.join("../../../examples/protein-package/manifest.json");

    let output = Command::new(env!("CARGO_BIN_EXE_biors"))
        .arg("--json")
        .arg("package")
        .arg("verify")
        .arg(manifest)
        .arg("-")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn biors package verify")
        .tap_stdin(
            r#"[
              {
                "name": "tiny-protein",
                "path": "observed/missing.json"
              }
            ]"#,
        );

    assert_eq!(output.status.code(), Some(2));
    assert!(output.stderr.is_empty());

    let value: Value = serde_json::from_slice(&output.stdout).expect("valid JSON error");
    assert_eq!(value["error"]["code"], "package.observed_output_missing");
    assert_eq!(value["error"]["location"], "fixtures");
}

#[test]
fn package_verify_reports_content_mismatch_code() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let manifest = manifest_dir.join("../../../examples/protein-package/manifest.json");
    let observations =
        manifest_dir.join("../../../examples/protein-package/observations.mismatch.json");

    let output = Command::new(env!("CARGO_BIN_EXE_biors"))
        .arg("--json")
        .arg("package")
        .arg("verify")
        .arg(manifest)
        .arg(observations)
        .output()
        .expect("run biors package verify");

    assert_eq!(output.status.code(), Some(2));
    assert!(output.stderr.is_empty());

    let value: Value = serde_json::from_slice(&output.stdout).expect("valid JSON error");
    assert_eq!(value["error"]["code"], "package.output_content_mismatch");
    assert_eq!(value["error"]["location"], "fixtures");
}

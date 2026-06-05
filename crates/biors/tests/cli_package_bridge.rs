use serde_json::Value;
use std::process::{Command, Stdio};

mod common;
mod package_support;
use common::ChildInputExt;

#[test]
fn package_bridge_outputs_runtime_plan() {
    let output = Command::new(env!("CARGO_BIN_EXE_biors"))
        .arg("package")
        .arg("bridge")
        .arg(package_support::example_manifest_path())
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

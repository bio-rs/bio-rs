use serde_json::Value;
use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};

#[test]
fn tokenize_multi_fasta_outputs_json_array() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let example = manifest_dir.join("../../../examples/multi.fasta");

    let output = Command::new(env!("CARGO_BIN_EXE_biors"))
        .arg("tokenize")
        .arg(example)
        .output()
        .expect("run biors tokenize");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let value: Value = serde_json::from_slice(&output.stdout).expect("valid JSON output");
    let records = value.as_array().expect("multi-FASTA output is an array");

    assert_eq!(records.len(), 2);
    assert_eq!(records[0]["id"], "seq1");
    assert_eq!(records[1]["id"], "seq2");
}

#[test]
fn tokenize_stdin_outputs_json_array() {
    let output = run_with_stdin("tokenize", ">seq1\nACDE\n");
    let value: Value = serde_json::from_slice(&output).expect("valid JSON output");
    let records = value.as_array().expect("tokenize output is an array");

    assert_eq!(records.len(), 1);
    assert_eq!(records[0]["id"], "seq1");
    assert_eq!(records[0]["tokens"], serde_json::json!([0, 1, 2, 3]));
}

#[test]
fn inspect_stdin_outputs_json_summary() {
    let output = run_with_stdin("inspect", ">seq1\nACX\n>seq2\nM*\n");
    let value: Value = serde_json::from_slice(&output).expect("valid JSON output");

    assert_eq!(value["records"], 2);
    assert_eq!(value["total_length"], 5);
    assert_eq!(value["valid_records"], 0);
    assert_eq!(value["warning_count"], 1);
    assert_eq!(value["error_count"], 1);
}

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

    assert_eq!(value["schema_version"], "biors.package.v0");
    assert_eq!(value["name"], "protein-seed");
    assert_eq!(value["model_format"], "onnx");
    assert_eq!(value["runtime_backend"], "onnx-webgpu");
    assert_eq!(value["runtime_target"], "browser-wasm-webgpu");
    assert_eq!(value["fixtures"], 1);
}

#[test]
fn package_validate_fails_invalid_manifest() {
    let output = Command::new(env!("CARGO_BIN_EXE_biors"))
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

    let value: Value = serde_json::from_slice(&output.stdout).expect("valid JSON output");

    assert_eq!(value["valid"], false);
    assert_eq!(value["issues"][0], "name is required");
    assert_eq!(value["issues"][1], "model.path is required");
    assert_eq!(
        value["issues"][2],
        "fixtures must include at least one fixture"
    );
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

    assert_eq!(value["ready"], true);
    assert_eq!(value["backend"], "onnx-webgpu");
    assert_eq!(value["target"], "browser-wasm-webgpu");
    assert_eq!(value["execution_provider"], "webgpu");
    assert_eq!(
        value["blocking_issues"]
            .as_array()
            .expect("issues array")
            .len(),
        0
    );
}

fn run_with_stdin(command: &str, input: &str) -> Vec<u8> {
    let mut child = Command::new(env!("CARGO_BIN_EXE_biors"))
        .arg(command)
        .arg("-")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn biors");

    child
        .stdin
        .as_mut()
        .expect("stdin pipe")
        .write_all(input.as_bytes())
        .expect("write stdin");

    let output = child.wait_with_output().expect("wait for biors");
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    output.stdout
}

trait ChildInputExt {
    fn tap_stdin(self, input: &str) -> std::process::Output;
}

impl ChildInputExt for std::process::Child {
    fn tap_stdin(mut self, input: &str) -> std::process::Output {
        self.stdin
            .as_mut()
            .expect("stdin pipe")
            .write_all(input.as_bytes())
            .expect("write stdin");

        self.wait_with_output().expect("wait for biors")
    }
}

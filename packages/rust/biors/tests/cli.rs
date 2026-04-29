use serde_json::Value;
use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};

#[test]
fn cli_version_flag_reports_published_package_version() {
    let output = Command::new(env!("CARGO_BIN_EXE_biors"))
        .arg("--version")
        .output()
        .expect("run biors --version");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stderr.is_empty());

    let stdout = String::from_utf8(output.stdout).expect("version output is UTF-8");
    assert_eq!(
        stdout.trim(),
        format!("biors {}", env!("CARGO_PKG_VERSION"))
    );
}

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
    let records = value["data"]
        .as_array()
        .expect("multi-FASTA output is an array");

    assert_eq!(value["ok"], true);
    assert_eq!(value["biors_version"], env!("CARGO_PKG_VERSION"));
    assert!(value["input_hash"]
        .as_str()
        .expect("input hash")
        .starts_with("fnv1a64:"));
    assert_eq!(records.len(), 2);
    assert_eq!(records[0]["id"], "seq1");
    assert_eq!(records[1]["id"], "seq2");
}

#[test]
fn tokenize_stdin_outputs_json_array() {
    let output = run_with_stdin("tokenize", ">seq1\nACDE\n");
    let value: Value = serde_json::from_slice(&output).expect("valid JSON output");
    let records = value["data"]
        .as_array()
        .expect("tokenize output is an array");

    assert_eq!(records.len(), 1);
    assert_eq!(records[0]["id"], "seq1");
    assert_eq!(records[0]["tokens"], serde_json::json!([0, 1, 2, 3]));
}

#[test]
fn inspect_stdin_outputs_json_summary() {
    let output = run_with_stdin("inspect", ">seq1\nACX\n>seq2\nM*\n");
    let value: Value = serde_json::from_slice(&output).expect("valid JSON output");

    assert_eq!(value["data"]["records"], 2);
    assert_eq!(value["data"]["total_length"], 5);
    assert_eq!(value["data"]["valid_records"], 0);
    assert_eq!(value["data"]["warning_count"], 1);
    assert_eq!(value["data"]["error_count"], 1);
}

#[test]
fn tokenize_preserves_unknown_token_positions() {
    let output = run_with_stdin("tokenize", ">seq1\nAX*\n");
    let value: Value = serde_json::from_slice(&output).expect("valid JSON output");
    let record = &value["data"][0];

    assert_eq!(record["length"], 3);
    assert_eq!(record["tokens"], serde_json::json!([0, 20, 20]));
    assert_eq!(record["warnings"].as_array().expect("warnings").len(), 1);
    assert_eq!(record["errors"].as_array().expect("errors").len(), 1);
}

#[test]
fn model_input_stdin_outputs_model_ready_json() {
    let output = Command::new(env!("CARGO_BIN_EXE_biors"))
        .arg("model-input")
        .arg("--max-length")
        .arg("6")
        .arg("-")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn biors model-input")
        .tap_stdin(">seq1\nACDE\n");

    assert!(output.status.success());
    assert!(output.stderr.is_empty());

    let value: Value = serde_json::from_slice(&output.stdout).expect("valid JSON output");
    assert_eq!(value["ok"], true);
    assert_eq!(value["data"]["policy"]["max_length"], 6);
    assert_eq!(value["data"]["policy"]["pad_token_id"], 0);
    assert_eq!(value["data"]["policy"]["padding"], "fixed_length");
    assert_eq!(
        value["data"]["records"][0]["input_ids"],
        serde_json::json!([0, 1, 2, 3, 0, 0])
    );
    assert_eq!(
        value["data"]["records"][0]["attention_mask"],
        serde_json::json!([1, 1, 1, 1, 0, 0])
    );
    assert_eq!(value["data"]["records"][0]["truncated"], false);
}

#[test]
fn fasta_validate_outputs_validation_report() {
    let output = Command::new(env!("CARGO_BIN_EXE_biors"))
        .arg("fasta")
        .arg("validate")
        .arg("-")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn biors fasta validate")
        .tap_stdin(">seq1\nAX*\n");

    assert!(output.status.success());
    assert!(output.stderr.is_empty());

    let value: Value = serde_json::from_slice(&output.stdout).expect("valid JSON output");
    assert_eq!(value["ok"], true);
    assert_eq!(value["data"]["records"], 1);
    assert_eq!(value["data"]["warning_count"], 1);
    assert_eq!(value["data"]["error_count"], 1);
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

    assert_eq!(value["data"]["schema_version"], "biors.package.v0");
    assert_eq!(value["data"]["name"], "protein-seed");
    assert_eq!(value["data"]["model_format"], "onnx");
    assert_eq!(value["data"]["has_model_checksum"], true);
    assert_eq!(value["data"]["runtime_backend"], "onnx-webgpu");
    assert_eq!(value["data"]["runtime_target"], "browser-wasm-webgpu");
    assert_eq!(value["data"]["fixtures"], 1);
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

#[test]
fn json_error_mode_outputs_contract_shape() {
    let output = Command::new(env!("CARGO_BIN_EXE_biors"))
        .arg("--json")
        .arg("tokenize")
        .arg("-")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn biors tokenize")
        .tap_stdin("ACDE\n");

    assert_eq!(output.status.code(), Some(2));
    assert!(output.stderr.is_empty());

    let value: Value = serde_json::from_slice(&output.stdout).expect("valid JSON error");
    assert_eq!(value["ok"], false);
    assert_eq!(value["error"]["code"], "fasta.missing_header");
    assert_eq!(value["error"]["location"]["line"], 1);
}

#[test]
fn human_error_mode_uses_stderr_and_exit_code() {
    let output = Command::new(env!("CARGO_BIN_EXE_biors"))
        .arg("tokenize")
        .arg("-")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn biors tokenize")
        .tap_stdin("ACDE\n");

    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8_lossy(&output.stderr).contains("error: FASTA input"));
}

#[test]
fn public_behavior_snapshot_for_tokenize_stdout() {
    let output = run_with_stdin("tokenize", ">seq1\nACDE\n");
    let value: Value = serde_json::from_slice(&output).expect("valid JSON output");

    assert_eq!(
        value["data"],
        serde_json::json!([
            {
                "id": "seq1",
                "length": 4,
                "alphabet": "protein-20",
                "valid": true,
                "tokens": [0, 1, 2, 3],
                "warnings": [],
                "errors": []
            }
        ])
    );
}

#[test]
fn public_behavior_snapshot_for_model_input_stdout() {
    let output = Command::new(env!("CARGO_BIN_EXE_biors"))
        .arg("model-input")
        .arg("--max-length")
        .arg("4")
        .arg("-")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn biors model-input")
        .tap_stdin(">seq1\nACDEFG\n");
    let value: Value = serde_json::from_slice(&output.stdout).expect("valid JSON output");

    assert_eq!(
        value["data"],
        serde_json::json!({
            "policy": {
                "max_length": 4,
                "pad_token_id": 0,
                "padding": "fixed_length"
            },
            "records": [
                {
                    "id": "seq1",
                    "input_ids": [0, 1, 2, 3],
                    "attention_mask": [1, 1, 1, 1],
                    "truncated": true
                }
            ]
        })
    );
}

#[test]
fn model_input_json_error_rejects_non_model_ready_sequences() {
    let output = Command::new(env!("CARGO_BIN_EXE_biors"))
        .arg("--json")
        .arg("model-input")
        .arg("--max-length")
        .arg("8")
        .arg("-")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn biors model-input")
        .tap_stdin(">seq1\nAX*\n");

    assert_eq!(output.status.code(), Some(2));
    assert!(output.stderr.is_empty());

    let value: Value = serde_json::from_slice(&output.stdout).expect("valid JSON error");
    assert_eq!(value["ok"], false);
    assert_eq!(value["error"]["code"], "model_input.invalid_sequence");
    assert_eq!(value["error"]["location"], "seq1");
}

#[test]
fn model_input_json_error_rejects_zero_max_length() {
    let output = Command::new(env!("CARGO_BIN_EXE_biors"))
        .arg("--json")
        .arg("model-input")
        .arg("--max-length")
        .arg("0")
        .arg("-")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn biors model-input")
        .tap_stdin(">seq1\nACDE\n");

    assert_eq!(output.status.code(), Some(2));
    assert!(output.stderr.is_empty());

    let value: Value = serde_json::from_slice(&output.stdout).expect("valid JSON error");
    assert_eq!(value["ok"], false);
    assert_eq!(value["error"]["code"], "model_input.invalid_policy");
    assert!(value["error"]["message"]
        .as_str()
        .expect("message")
        .contains("max_length must be greater than zero"));
}

#[test]
fn json_error_mode_rejects_empty_fasta_identifier() {
    let output = Command::new(env!("CARGO_BIN_EXE_biors"))
        .arg("--json")
        .arg("tokenize")
        .arg("-")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn biors tokenize")
        .tap_stdin(">\nACDE\n");

    assert_eq!(output.status.code(), Some(2));
    assert!(output.stderr.is_empty());

    let value: Value = serde_json::from_slice(&output.stdout).expect("valid JSON error");
    assert_eq!(value["error"]["code"], "fasta.missing_identifier");
    assert_eq!(value["error"]["location"]["line"], 1);
    assert_eq!(value["error"]["location"]["record_index"], 0);
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

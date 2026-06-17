use serde_json::{json, Value};
use std::fs;
use std::process::Command;

mod common;
use common::ChildInputExt;

fn run_json_stdin(args: &[&str], input: &str) -> std::process::Output {
    common::spawn_biors(args).tap_stdin(input)
}

fn json_output(output: &std::process::Output) -> Value {
    assert!(output.stderr.is_empty());
    serde_json::from_slice(&output.stdout).expect("valid JSON output")
}

fn json_error(output: &std::process::Output) -> Value {
    assert!(
        !output.status.success(),
        "command unexpectedly succeeded: {}",
        String::from_utf8_lossy(&output.stdout)
    );
    let value = json_output(output);
    common::assert_json_value_matches_schema(&value, "schemas/cli-error.v0.json");
    value
}

fn assert_recovery_hint(value: &Value, expected_code: &str, expected_fragment: &str) {
    assert_eq!(value["error"]["code"], expected_code);
    assert!(value["error"]["recovery_hint"]
        .as_str()
        .expect("recovery_hint")
        .contains(expected_fragment));
}

#[test]
fn invalid_sequence_errors_include_recovery_hint() {
    let output = run_json_stdin(
        &["--json", "model-input", "--max-length", "16", "-"],
        ">bad\nACDZ\n",
    );

    let value = json_error(&output);
    assert_recovery_hint(&value, "model_input.invalid_sequence", "validation");
}

#[test]
fn profile_mismatch_workflow_reports_readiness_recovery_hint() {
    let output = run_json_stdin(
        &[
            "--json",
            "workflow",
            "--profile",
            "dna-iupac",
            "--max-length",
            "16",
            "-",
        ],
        ">protein\nACDEFGHIK\n",
    );

    assert!(output.status.success());
    let value = json_output(&output);
    assert_eq!(value["data"]["model_ready"], false);
    assert_eq!(
        value["data"]["readiness_issues"][0]["code"],
        "sequence.not_model_ready"
    );
    assert!(value["data"]["readiness_issues"][0]["recovery_hint"]
        .as_str()
        .expect("readiness recovery_hint")
        .contains("matching sequence kind"));
}

#[test]
fn package_path_traversal_errors_include_recovery_hint() {
    let output = run_json_stdin(
        &["--json", "package", "validate", "-"],
        r#"{
          "schema_version": "biors.package.v0",
          "name": "bad-path",
          "model": { "format": "onnx", "path": "../escape.onnx" },
          "preprocessing": [],
          "postprocessing": [],
          "runtime": {
            "backend": "onnx-webgpu",
            "target": "browser-wasm-webgpu"
          },
          "fixtures": []
        }"#,
    );

    let value = json_error(&output);
    assert_recovery_hint(&value, "package.invalid_asset_path", "package-relative");
}

#[test]
fn checksum_mismatch_errors_include_recovery_hint() {
    let temp = common::TempDir::new("biors-recovery-checksum");
    std::fs::create_dir_all(temp.path().join("models")).expect("create models dir");
    std::fs::write(
        temp.path().join("models/protein.onnx"),
        "not-the-declared-checksum",
    )
    .expect("write model");
    let manifest = temp.write(
        "manifest.json",
        &json!({
            "schema_version": "biors.package.v0",
            "name": "checksum-mismatch",
            "model": {
                "format": "onnx",
                "path": "models/protein.onnx",
                "checksum": "sha256:0000000000000000000000000000000000000000000000000000000000000000"
            },
            "preprocessing": [],
            "postprocessing": [],
            "runtime": {
                "backend": "onnx-webgpu",
                "target": "browser-wasm-webgpu"
            },
            "fixtures": []
        })
        .to_string(),
    );

    let output = Command::new(env!("CARGO_BIN_EXE_biors"))
        .args(["--json", "package", "validate"])
        .arg(manifest)
        .output()
        .expect("run package validate");

    let value = json_error(&output);
    assert_recovery_hint(&value, "package.checksum_mismatch", "sha256");
}

#[test]
fn unsupported_runtime_errors_include_recovery_hint() {
    let output = run_json_stdin(
        &["--json", "package", "bridge", "-"],
        r#"{
          "schema_version": "biors.package.v0",
          "name": "external-runtime",
          "model": {
            "format": "onnx",
            "path": "testdata/protein-package/models/protein-seed.onnx"
          },
          "preprocessing": [],
          "postprocessing": [],
          "runtime": {
            "backend": "external-process",
            "target": "local-cpu"
          },
          "fixtures": []
        }"#,
    );

    let value = json_error(&output);
    assert_recovery_hint(&value, "package.bridge_not_ready", "public package runtime");
}

#[test]
fn malformed_json_errors_include_recovery_hint() {
    let output = run_json_stdin(&["--json", "package", "validate", "-"], "{not valid json");

    let value = json_error(&output);
    assert_recovery_hint(&value, "json.invalid", "valid JSON");
}

#[test]
fn missing_file_errors_include_recovery_hint() {
    let missing_path = format!(
        "/tmp/biors-missing-manifest-{}-{}.json",
        std::process::id(),
        "recovery"
    );
    let output = Command::new(env!("CARGO_BIN_EXE_biors"))
        .args(["--json", "package", "validate"])
        .arg(&missing_path)
        .output()
        .expect("run package validate");

    let value = json_error(&output);
    assert_recovery_hint(&value, "io.read_failed", "path exists");
}

#[test]
fn workflow_docs_cover_recovery_hint_failure_types() {
    let repo = common::repo_root();
    let workflows =
        fs::read_to_string(repo.join("docs/researcher-workflows.md")).expect("read workflows doc");
    let cli_contract =
        fs::read_to_string(repo.join("docs/cli-contract.md")).expect("read CLI contract");

    for expected in [
        "invalid FASTA",
        "unsupported residues",
        "kind/profile mismatch",
        "path traversal",
        "checksum mismatch",
        "unsupported public package runtime",
        "malformed JSON",
        "missing local files",
        "recovery_hint",
    ] {
        assert!(
            workflows.contains(expected) || cli_contract.contains(expected),
            "recovery docs missing {expected}"
        );
    }
}

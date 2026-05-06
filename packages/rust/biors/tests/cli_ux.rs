use serde_json::Value;
use std::process::{Command, Stdio};

mod common;
use common::ChildInputExt;

#[test]
fn human_errors_include_stable_error_code() {
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

    let stderr = String::from_utf8(output.stderr).expect("stderr is UTF-8");
    assert!(stderr.contains("error[fasta.missing_header]:"));
    assert!(stderr.contains("FASTA input must start with a header line"));
}

#[test]
fn help_snapshot_lists_commands_and_global_json_flag() {
    let output = Command::new(env!("CARGO_BIN_EXE_biors"))
        .arg("--help")
        .output()
        .expect("run biors help");

    assert!(output.status.success());
    assert!(output.stderr.is_empty());

    let stdout = String::from_utf8(output.stdout).expect("help is UTF-8");
    for expected in [
        "Usage: biors [OPTIONS] <COMMAND>",
        "--json",
        "doctor",
        "completions",
        "fasta",
        "inspect",
        "model-input",
        "package",
        "seq",
        "tokenize",
    ] {
        assert!(stdout.contains(expected), "help output missing {expected}");
    }
}

#[test]
fn completions_command_outputs_shell_script() {
    let output = Command::new(env!("CARGO_BIN_EXE_biors"))
        .arg("completions")
        .arg("bash")
        .output()
        .expect("run biors completions");

    assert!(output.status.success());
    assert!(output.stderr.is_empty());

    let stdout = String::from_utf8(output.stdout).expect("completion output is UTF-8");
    assert!(stdout.contains("_biors"));
    assert!(stdout.contains("COMPREPLY"));
}

#[test]
fn malformed_json_input_fails_without_panic() {
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
        .tap_stdin("{not valid json");

    assert_eq!(output.status.code(), Some(2));
    assert!(output.stderr.is_empty());

    let value: Value = serde_json::from_slice(&output.stdout).expect("valid JSON error");
    assert_eq!(value["error"]["code"], "json.invalid");
}

#[test]
fn invalid_utf8_fasta_fails_without_panic() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_biors"))
        .arg("--json")
        .arg("tokenize")
        .arg("-")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn biors tokenize");

    use std::io::Write;
    child
        .stdin
        .as_mut()
        .expect("stdin pipe")
        .write_all(b">seq1\nAC\xffDE\n")
        .expect("write stdin");

    let output = child.wait_with_output().expect("wait for biors");
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stderr.is_empty());

    let value: Value = serde_json::from_slice(&output.stdout).expect("valid JSON error");
    assert_eq!(value["error"]["code"], "io.read_failed");
    assert!(value["error"]["message"]
        .as_str()
        .expect("message")
        .contains("invalid UTF-8"));
}

#[test]
fn manifest_path_traversal_fails_without_panic() {
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

    assert_eq!(output.status.code(), Some(2));
    assert!(output.stderr.is_empty());

    let value: Value = serde_json::from_slice(&output.stdout).expect("valid JSON error");
    assert_eq!(value["error"]["code"], "package.invalid_asset_path");
}

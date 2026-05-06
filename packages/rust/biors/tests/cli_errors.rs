use serde_json::Value;
use std::process::{Command, Stdio};

mod common;
use common::ChildInputExt;

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
    assert!(String::from_utf8_lossy(&output.stderr).contains("error[fasta.missing_header]:"));
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

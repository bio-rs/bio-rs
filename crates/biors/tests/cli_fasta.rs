use serde_json::Value;
use std::process::{Command, Stdio};

mod common;
use common::ChildInputExt;

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

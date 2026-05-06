use serde_json::Value;
use std::process::{Command, Stdio};

mod common;
use common::ChildInputExt;

#[test]
fn workflow_outputs_validated_tokenized_model_ready_json_with_provenance() {
    let output = Command::new(env!("CARGO_BIN_EXE_biors"))
        .arg("workflow")
        .arg("--max-length")
        .arg("6")
        .arg("-")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn biors workflow")
        .tap_stdin(">seq1\nacde\n");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stderr.is_empty());

    let value: Value = serde_json::from_slice(&output.stdout).expect("valid JSON output");
    assert_eq!(value["ok"], true);
    assert_eq!(value["data"]["workflow"], "protein_model_input.v0");
    assert_eq!(value["data"]["model_ready"], true);
    assert_eq!(
        value["data"]["provenance"]["input_hash"],
        value["input_hash"]
    );
    assert_eq!(
        value["data"]["provenance"]["normalization"],
        "strip_ascii_whitespace_uppercase"
    );
    assert_eq!(
        value["data"]["provenance"]["tokenizer"]["name"],
        "protein-20"
    );
    assert_eq!(
        value["data"]["provenance"]["tokenizer"]["unknown_token_policy"],
        "warn_or_error_with_unknown_token"
    );
    assert_eq!(value["data"]["validation"]["records"], 1);
    assert_eq!(
        value["data"]["validation"]["sequences"][0]["sequence"],
        "ACDE"
    );
    assert_eq!(value["data"]["tokenization"]["summary"]["records"], 1);
    assert_eq!(
        value["data"]["tokenization"]["records"][0]["tokens"],
        serde_json::json!([0, 1, 2, 3])
    );
    assert_eq!(
        value["data"]["model_input"]["records"][0]["input_ids"],
        serde_json::json!([0, 1, 2, 3, 0, 0])
    );
    assert_eq!(value["data"]["readiness_issues"], serde_json::json!([]));
}

#[test]
fn workflow_reports_non_model_ready_sequences_without_losing_validation_context() {
    let output = Command::new(env!("CARGO_BIN_EXE_biors"))
        .arg("workflow")
        .arg("--max-length")
        .arg("6")
        .arg("-")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn biors workflow")
        .tap_stdin(">seq1\nAX*\n");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stderr.is_empty());

    let value: Value = serde_json::from_slice(&output.stdout).expect("valid JSON output");
    assert_eq!(value["data"]["model_ready"], false);
    assert_eq!(value["data"]["model_input"], Value::Null);
    assert_eq!(value["data"]["validation"]["warning_count"], 1);
    assert_eq!(value["data"]["validation"]["error_count"], 1);
    assert_eq!(
        value["data"]["readiness_issues"][0]["code"],
        "sequence.not_model_ready"
    );
    assert_eq!(value["data"]["readiness_issues"][0]["id"], "seq1");
}

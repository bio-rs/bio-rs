use serde_json::Value;
use std::process::{Command, Stdio};

mod common;
use common::ChildInputExt;

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

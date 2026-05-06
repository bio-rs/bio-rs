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

#[test]
fn workflow_records_invocation_and_reproducibility_hashes() {
    let first = run_workflow_json(">seq1\nacde\n");
    let second = run_workflow_json(">seq1\nacde\n");

    let provenance = &first["data"]["provenance"];
    assert_eq!(provenance["invocation"]["command"], "biors workflow");
    assert_eq!(
        provenance["invocation"]["arguments"],
        serde_json::json!([
            "--max-length",
            "6",
            "--pad-token-id",
            "0",
            "--padding",
            "fixed_length",
            "-"
        ])
    );

    for key in ["vocabulary_sha256", "output_data_sha256"] {
        assert!(
            provenance["hashes"][key]
                .as_str()
                .expect("workflow hash")
                .starts_with("sha256:"),
            "{key} should be a sha256 digest"
        );
    }
    assert_eq!(
        first["data"]["provenance"]["hashes"],
        second["data"]["provenance"]["hashes"]
    );
}

fn run_workflow_json(input: &str) -> Value {
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
        .tap_stdin(input);

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stderr.is_empty());
    serde_json::from_slice(&output.stdout).expect("valid JSON output")
}

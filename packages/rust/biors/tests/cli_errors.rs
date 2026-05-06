use serde_json::Value;

mod common;
use common::ChildInputExt;

fn run_biors(args: &[&str], input: &str) -> std::process::Output {
    common::spawn_biors(args).tap_stdin(input)
}

fn json_error(output: &std::process::Output) -> Value {
    assert!(output.stderr.is_empty());
    serde_json::from_slice(&output.stdout).expect("valid JSON error")
}

#[test]
fn json_error_mode_outputs_contract_shape() {
    let output = run_biors(&["--json", "tokenize", "-"], "ACDE\n");

    assert_eq!(output.status.code(), Some(2));

    let value = json_error(&output);
    assert_eq!(value["ok"], false);
    assert_eq!(value["error"]["code"], "fasta.missing_header");
    assert_eq!(value["error"]["location"]["line"], 1);
}

#[test]
fn human_error_mode_uses_stderr_and_exit_code() {
    let output = run_biors(&["tokenize", "-"], "ACDE\n");

    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8_lossy(&output.stderr).contains("error[fasta.missing_header]:"));
}

#[test]
fn json_error_mode_rejects_empty_fasta_identifier() {
    let output = run_biors(&["--json", "tokenize", "-"], ">\nACDE\n");

    assert_eq!(output.status.code(), Some(2));

    let value = json_error(&output);
    assert_eq!(value["error"]["code"], "fasta.missing_identifier");
    assert_eq!(value["error"]["location"]["line"], 1);
    assert_eq!(value["error"]["location"]["record_index"], 0);
}

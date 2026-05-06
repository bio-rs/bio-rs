use serde_json::Value;

mod common;
use common::run_with_stdin;

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

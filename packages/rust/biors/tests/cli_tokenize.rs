use serde_json::Value;
use std::path::Path;
use std::process::Command;

mod common;

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
    let output = common::run_biors_stdin(&["tokenize", "-"], ">seq1\nACDE\n").stdout;
    let value: Value = serde_json::from_slice(&output).expect("valid JSON output");
    let records = value["data"]
        .as_array()
        .expect("tokenize output is an array");

    assert_eq!(records.len(), 1);
    assert_eq!(records[0]["id"], "seq1");
    assert_eq!(records[0]["tokens"], serde_json::json!([0, 1, 2, 3]));
}

#[test]
fn tokenize_preserves_unknown_token_positions() {
    let output = common::run_biors_stdin(&["tokenize", "-"], ">seq1\nAX*\n").stdout;
    let value: Value = serde_json::from_slice(&output).expect("valid JSON output");
    let record = &value["data"][0];

    assert_eq!(record["length"], 3);
    assert_eq!(record["tokens"], serde_json::json!([0, 20, 20]));
    assert_eq!(record["warnings"].as_array().expect("warnings").len(), 1);
    assert_eq!(record["errors"].as_array().expect("errors").len(), 1);
}

#[test]
fn public_behavior_snapshot_for_tokenize_stdout() {
    let output = common::run_biors_stdin(&["tokenize", "-"], ">seq1\nACDE\n").stdout;
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

use serde_json::Value;
use std::process::{Command, Stdio};

mod common;
use common::ChildInputExt;

#[test]
fn formats_list_outputs_support_matrix() {
    let output = common::run_biors(&["formats", "list"]);
    common::assert_payload_matches_schema(
        &output.stdout,
        "schemas/format-capabilities-output.v0.json",
    );

    let value: Value = serde_json::from_slice(&output.stdout).expect("valid JSON output");
    let rows = value["data"].as_array().expect("capability rows");
    assert!(rows.iter().any(|row| {
        row["format"] == "fastq"
            && row["status"] == "supported"
            && row["record_contract"]
                .as_str()
                .unwrap()
                .contains("FastqRecord")
    }));
    assert!(rows
        .iter()
        .any(|row| { row["format"] == "vcf" && row["status"] == "reviewed_candidate" }));
    assert!(rows
        .iter()
        .any(|row| { row["format"] == "pdb" && row["status"] == "supported" }));
    assert!(rows
        .iter()
        .any(|row| { row["format"] == "mmcif" && row["status"] == "reviewed_candidate" }));
}

#[test]
fn formats_validate_fastq_reports_sequence_and_quality_diagnostics() {
    let output = Command::new(env!("CARGO_BIN_EXE_biors"))
        .args(["formats", "validate", "--format", "fastq", "-"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn biors formats validate")
        .tap_stdin("@r1 instrument\nACGN*\n+\n!!!! \n");

    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    common::assert_payload_matches_schema(
        &output.stdout,
        "schemas/fastq-validation-output.v0.json",
    );

    let value: Value = serde_json::from_slice(&output.stdout).expect("valid JSON output");
    assert_eq!(value["ok"], true);
    assert_eq!(value["data"]["format"], "fastq");
    assert_eq!(value["data"]["records"], 1);
    assert_eq!(value["data"]["valid_records"], 0);
    assert_eq!(value["data"]["warning_count"], 1);
    assert_eq!(value["data"]["error_count"], 2);
    assert_eq!(
        value["data"]["record_reports"][0]["description"],
        "instrument"
    );
    assert_eq!(
        value["data"]["record_reports"][0]["warnings"][0]["code"],
        "ambiguous_symbol"
    );
    assert_eq!(
        value["data"]["record_reports"][0]["errors"][0]["code"],
        "invalid_symbol"
    );
    assert_eq!(
        value["data"]["record_reports"][0]["errors"][1]["code"],
        "invalid_quality_character"
    );
}

#[test]
fn formats_validate_fastq_parse_error_uses_fastq_code() {
    let output = Command::new(env!("CARGO_BIN_EXE_biors"))
        .args(["--json", "formats", "validate", "--format", "fastq", "-"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn biors formats validate")
        .tap_stdin("@r1\nACG\n+\n!!!!\n");

    assert_eq!(output.status.code(), Some(2));
    assert!(output.stderr.is_empty());
    let value: Value = serde_json::from_slice(&output.stdout).expect("valid JSON error");
    assert_eq!(value["error"]["code"], "fastq.quality_length_mismatch");
    assert_eq!(value["error"]["location"]["line"], 4);
    assert_eq!(value["error"]["location"]["record_index"], 0);
}

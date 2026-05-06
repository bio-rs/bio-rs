use serde_json::Value;
use std::process::Output;

mod common;

#[test]
fn fasta_validate_kind_flag_outputs_kind_aware_report() {
    let output = run_with_stdin(
        ["fasta", "validate", "--kind", "dna", "-"],
        ">seq1\nACGNU\n",
    );

    let value: Value = serde_json::from_slice(&output.stdout).expect("valid JSON output");
    assert_eq!(value["data"]["kind_counts"]["dna"], 1);
    assert_eq!(value["data"]["sequences"][0]["kind"], "dna");
    assert_eq!(value["data"]["sequences"][0]["alphabet"], "dna-iupac");
    assert_eq!(
        value["data"]["sequences"][0]["warnings"][0]["code"],
        "ambiguous_symbol"
    );
    assert_eq!(
        value["data"]["sequences"][0]["errors"][0]["code"],
        "invalid_symbol"
    );
}

#[test]
fn fasta_validate_auto_detects_mixed_sequence_kinds() {
    let output = run_with_stdin(
        ["fasta", "validate", "--kind", "auto", "-"],
        ">dna\nACGN\n>rna\nACGU\n>protein\nMEEPQSDPSV\n",
    );

    assert_mixed_kind_counts(&output);
}

#[test]
fn seq_validate_defaults_to_auto_detected_sequence_kinds() {
    let output = run_with_stdin(
        ["seq", "validate", "-"],
        ">dna\nACGN\n>rna\nACGU\n>protein\nMEEPQSDPSV\n",
    );

    assert_mixed_kind_counts(&output);
}

#[test]
fn seq_validate_kind_override_uses_kind_specific_messages() {
    let output = run_with_stdin(
        ["seq", "validate", "--kind", "rna", "-"],
        ">dna-looking\nACGT\n",
    );

    let value: Value = serde_json::from_slice(&output.stdout).expect("valid JSON output");
    assert_eq!(value["data"]["sequences"][0]["kind"], "rna");
    assert_eq!(value["data"]["sequences"][0]["errors"][0]["symbol"], "T");
    assert!(value["data"]["sequences"][0]["errors"][0]["message"]
        .as_str()
        .expect("message")
        .contains("RNA"));
}

fn run_with_stdin<const N: usize>(args: [&str; N], input: &str) -> Output {
    common::run_biors_stdin(&args, input)
}

fn assert_mixed_kind_counts(output: &Output) {
    let value: Value = serde_json::from_slice(&output.stdout).expect("valid JSON output");
    assert_eq!(value["data"]["kind_counts"]["dna"], 1);
    assert_eq!(value["data"]["kind_counts"]["rna"], 1);
    assert_eq!(value["data"]["kind_counts"]["protein"], 1);
    assert_eq!(value["data"]["sequences"][0]["kind"], "dna");
    assert_eq!(value["data"]["sequences"][1]["kind"], "rna");
    assert_eq!(value["data"]["sequences"][2]["kind"], "protein");
}

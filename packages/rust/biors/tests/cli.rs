use serde_json::Value;
use std::path::Path;
use std::process::Command;

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
    let records = value.as_array().expect("multi-FASTA output is an array");

    assert_eq!(records.len(), 2);
    assert_eq!(records[0]["id"], "seq1");
    assert_eq!(records[1]["id"], "seq2");
}

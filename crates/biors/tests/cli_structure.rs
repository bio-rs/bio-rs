use serde_json::Value;
use std::process::{Command, Stdio};

mod common;
use common::ChildInputExt;

const PDB: &str = "\
HEADER    TEST STRUCTURE                          01-JAN-00   1ABC
TITLE     EXAMPLE STRUCTURE
SEQRES   1 A    4  ALA CYS ASP GLU
REMARK 465     CYS A    2
ATOM      1  N   ALA A   1      11.104  13.207  14.100  1.00 20.00           N
ATOM      2  CA  ALA A   1      12.000  13.500  14.700  1.00 21.00           C
ATOM      3  N   GLU A   4      15.104  17.207  18.100  1.00 20.00           N
END
";

#[test]
fn structure_validate_pdb_reports_chain_mapping_and_missing_residue() {
    let output = common::run_biors_stdin(&["structure", "validate", "--format", "pdb", "-"], PDB);
    common::assert_payload_matches_schema(
        &output.stdout,
        "schemas/structure-validation-output.v0.json",
    );

    let value: Value = serde_json::from_slice(&output.stdout).expect("valid JSON output");
    assert_eq!(value["data"]["format"], "pdb");
    assert_eq!(value["data"]["valid"], true);
    assert_eq!(value["data"]["chains"], 1);
    assert_eq!(
        value["data"]["chain_reports"][0]["sequence_mapping"]["status"],
        "coordinate_subsequence"
    );
    assert_eq!(value["data"]["warnings"][0]["code"], "missing_residue");
}

#[test]
fn structure_sequence_pdb_outputs_coordinate_and_seqres_sequences() {
    let output = common::run_biors_stdin(&["structure", "sequence", "--format", "pdb", "-"], PDB);
    common::assert_payload_matches_schema(
        &output.stdout,
        "schemas/structure-sequence-output.v0.json",
    );

    let value: Value = serde_json::from_slice(&output.stdout).expect("valid JSON output");
    assert_eq!(value["data"]["chains"][0]["chain_id"], "A");
    assert_eq!(value["data"]["chains"][0]["coordinate_sequence"], "AE");
    assert_eq!(value["data"]["chains"][0]["seqres_sequence"], "ACDE");
    assert_eq!(
        value["data"]["chains"][0]["mapping"]["coordinate_to_seqres_positions"],
        serde_json::json!([1, 4])
    );
}

#[test]
fn structure_validate_pdb_parse_error_uses_pdb_code() {
    let output = Command::new(env!("CARGO_BIN_EXE_biors"))
        .args(["--json", "structure", "validate", "--format", "pdb", "-"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn biors structure validate")
        .tap_stdin(
            "ATOM      1  N   ALA A   1      not-num 13.207  14.100  1.00 20.00           N\n",
        );

    assert_eq!(output.status.code(), Some(2));
    assert!(output.stderr.is_empty());
    let value: Value = serde_json::from_slice(&output.stdout).expect("valid JSON error");
    assert_eq!(value["error"]["code"], "pdb.invalid_atom_field");
    assert_eq!(value["error"]["location"]["line"], 1);
}

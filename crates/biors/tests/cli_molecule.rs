use serde_json::Value;
use std::process::{Command, Stdio};

mod common;
use common::ChildInputExt;

const SMILES: &str = "\
CC(=O)O acetate
c1ccccc1 benzene
[NH4+] ammonium
";

const SDF: &str = "\
ethanol
  bio-rs

  3  2  0  0  0  0            999 V2000
    0.0000    0.0000    0.0000 C   0  0  0  0  0  0  0  0  0  0  0  0
    1.5000    0.0000    0.0000 C   0  0  0  0  0  0  0  0  0  0  0  0
    2.1000    1.2000    0.0000 O   0  0  0  0  0  0  0  0  0  0  0  0
  1  2  1  0  0  0  0
  2  3  1  0  0  0  0
M  END
>  <ASSAY>
active

$$$$
";

const MOL2: &str = "\
@<TRIPOS>MOLECULE
ethanol
3 2 0 0 0
SMALL
USER_CHARGES
@<TRIPOS>ATOM
1 C1 0.000 0.000 0.000 C.3 1 ETO -0.1
2 C2 1.500 0.000 0.000 C.3 1 ETO 0.1
3 O3 2.100 1.200 0.000 O.3 1 ETO -0.2
@<TRIPOS>BOND
1 1 2 1
2 2 3 1
";

#[test]
fn molecule_validate_smiles_reports_graph_and_validation_limits() {
    let output =
        common::run_biors_stdin(&["molecule", "validate", "--format", "smiles", "-"], SMILES);
    common::assert_payload_matches_schema(
        &output.stdout,
        "schemas/molecule-validation-output.v0.json",
    );

    let value: Value = serde_json::from_slice(&output.stdout).expect("valid JSON output");
    assert_eq!(value["data"]["format"], "smiles");
    assert_eq!(value["data"]["valid"], true);
    assert_eq!(value["data"]["records"], 3);
    assert_eq!(value["data"]["atom_count"], 11);
    assert_eq!(
        value["data"]["record_reports"][1]["warnings"][0]["code"],
        "aromaticity_not_verified"
    );
    assert_eq!(
        value["data"]["record_reports"][0]["derived"]["fingerprint"]["algorithm"],
        "biors-ecfp-lite-v0"
    );
    assert!(
        value["data"]["record_reports"][0]["derived"]["canonical_graph_key"]
            .as_str()
            .expect("canonical graph key")
            .starts_with("biors-graph-v0;")
    );
}

#[test]
fn molecule_inspect_smiles_outputs_atom_and_bond_graph() {
    let output =
        common::run_biors_stdin(&["molecule", "inspect", "--format", "smiles", "-"], SMILES);
    common::assert_payload_matches_schema(
        &output.stdout,
        "schemas/molecule-records-output.v0.json",
    );

    let value: Value = serde_json::from_slice(&output.stdout).expect("valid JSON output");
    let first = &value["data"][0];
    assert_eq!(first["id"], "acetate");
    assert_eq!(first["metadata"]["branch_count"], 1);
    assert_eq!(first["graph"]["atoms"]["atoms"][0]["element"], "C");
    assert_eq!(first["graph"]["bonds"]["bonds"][1]["order"], "double");
}

#[test]
fn molecule_validate_and_inspect_sdf_preserves_properties_and_coordinates() {
    let validated = common::run_biors_stdin(&["molecule", "validate", "--format", "sdf", "-"], SDF);
    common::assert_payload_matches_schema(
        &validated.stdout,
        "schemas/molecule-validation-output.v0.json",
    );
    let validation: Value = serde_json::from_slice(&validated.stdout).expect("valid JSON output");
    assert_eq!(validation["data"]["format"], "sdf");
    assert_eq!(validation["data"]["valid"], true);
    assert_eq!(
        validation["data"]["record_reports"][0]["derived"]["formula"],
        "C2O"
    );

    let inspected = common::run_biors_stdin(&["molecule", "inspect", "--format", "sdf", "-"], SDF);
    common::assert_payload_matches_schema(
        &inspected.stdout,
        "schemas/molecule-records-output.v0.json",
    );
    let value: Value = serde_json::from_slice(&inspected.stdout).expect("valid JSON output");
    assert_eq!(value["data"][0]["properties"][0]["name"], "ASSAY");
    assert_eq!(
        value["data"][0]["graph"]["atoms"]["atoms"][2]["coordinate"]["y"],
        1.2
    );
}

#[test]
fn molecule_validate_and_inspect_mol2_preserves_atom_types_and_charges() {
    let validated =
        common::run_biors_stdin(&["molecule", "validate", "--format", "mol2", "-"], MOL2);
    common::assert_payload_matches_schema(
        &validated.stdout,
        "schemas/molecule-validation-output.v0.json",
    );
    let validation: Value = serde_json::from_slice(&validated.stdout).expect("valid JSON output");
    assert_eq!(validation["data"]["format"], "mol2");
    assert_eq!(validation["data"]["atom_count"], 3);

    let inspected =
        common::run_biors_stdin(&["molecule", "inspect", "--format", "mol2", "-"], MOL2);
    common::assert_payload_matches_schema(
        &inspected.stdout,
        "schemas/molecule-records-output.v0.json",
    );
    let value: Value = serde_json::from_slice(&inspected.stdout).expect("valid JSON output");
    assert_eq!(
        value["data"][0]["graph"]["atoms"]["atoms"][2]["atom_type"],
        "O.3"
    );
    assert_eq!(
        value["data"][0]["graph"]["atoms"]["atoms"][0]["partial_charge"],
        -0.1
    );
}

#[test]
fn molecule_validate_smiles_reports_valence_errors() {
    let output = common::run_biors_stdin(
        &["molecule", "validate", "--format", "smiles", "-"],
        "C(=O)(=O)(=O) overfilled\n",
    );

    let value: Value = serde_json::from_slice(&output.stdout).expect("valid JSON output");
    assert_eq!(value["data"]["valid"], false);
    assert_eq!(
        value["data"]["record_reports"][0]["errors"][0]["code"],
        "valence_exceeded"
    );
}

#[test]
fn molecule_validate_smiles_parse_error_uses_smiles_code() {
    let output = Command::new(env!("CARGO_BIN_EXE_biors"))
        .args(["--json", "molecule", "validate", "--format", "smiles", "-"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn biors molecule validate")
        .tap_stdin("C1CC\n");

    assert_eq!(output.status.code(), Some(2));
    assert!(output.stderr.is_empty());
    let value: Value = serde_json::from_slice(&output.stdout).expect("valid JSON error");
    assert_eq!(value["error"]["code"], "smiles.unclosed_ring");
    assert_eq!(value["error"]["location"]["line"], 1);
}

use serde_json::Value;
use std::process::Command;

mod common;

#[test]
fn templates_list_outputs_stable_catalog() {
    let output = common::run_biors(&["templates", "list"]);
    let value: Value = serde_json::from_slice(&output.stdout).expect("valid JSON output");
    let templates = value["data"].as_array().expect("template array");

    let ids: Vec<_> = templates
        .iter()
        .map(|template| template["id"].as_str().expect("template id"))
        .collect();
    assert_eq!(
        ids,
        vec![
            "protein-classification-v0",
            "protein-embedding-generation-v0",
            "variant-effect-prediction-v0",
            "molecule-property-prediction-v0",
            "structure-validation-v0",
            "sequence-similarity-preprocess-v0",
        ]
    );
    assert!(templates
        .iter()
        .all(|template| template["execution"]["external_model_calls"] == false));
}

#[test]
fn templates_show_outputs_required_fields_for_one_template() {
    let output = common::run_biors(&["templates", "show", "molecule-property-prediction-v0"]);
    let value: Value = serde_json::from_slice(&output.stdout).expect("valid JSON output");

    assert_eq!(value["data"]["id"], "molecule-property-prediction-v0");
    assert_eq!(
        value["data"]["supported_inputs"][0]["formats"][0]["format"],
        "smiles"
    );
    assert!(value["data"]["model_ready_fields"]
        .as_array()
        .expect("fields")
        .iter()
        .any(|field| field["name"] == "fingerprint" && field["required"] == true));
    assert!(value["data"]["output_expectations"]
        .as_array()
        .expect("outputs")
        .iter()
        .any(|output| output["name"] == "property_value" && output["required"] == true));
}

#[test]
fn templates_show_missing_id_reports_json_error() {
    let output = Command::new(env!("CARGO_BIN_EXE_biors"))
        .args(["--json", "templates", "show", "missing-template"])
        .output()
        .expect("run biors templates show");

    assert_eq!(output.status.code(), Some(2));
    assert!(output.stderr.is_empty());
    let value: Value = serde_json::from_slice(&output.stdout).expect("valid JSON output");
    assert_eq!(value["error"]["code"], "template.not_found");
    assert_eq!(value["error"]["location"], "missing-template");
}

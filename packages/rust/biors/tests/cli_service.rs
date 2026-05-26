use serde_json::Value;

mod common;

#[test]
fn service_contract_outputs_stable_json_boundary() {
    let output = common::run_biors_paths(&["service", "contract"], &[]);
    let value: Value = serde_json::from_slice(&output.stdout).expect("valid JSON output");

    assert_eq!(value["ok"], true);
    assert_eq!(
        value["data"]["schema_version"],
        "biors.service_interface.v0"
    );
    assert_eq!(value["data"]["server_runtime"], "not_included");
    assert_eq!(value["data"]["openapi"]["status"], "offline_contract");
    assert!(value["data"]["routes"]
        .as_array()
        .expect("routes")
        .iter()
        .any(|route| route["operation_id"] == "package.bridge.plan"));
}

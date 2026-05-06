use serde_json::Value;
use std::process::Command;

#[test]
fn doctor_reports_local_readiness_checks() {
    let output = Command::new(env!("CARGO_BIN_EXE_biors"))
        .arg("doctor")
        .output()
        .expect("run biors doctor");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stderr.is_empty());

    let value: Value = serde_json::from_slice(&output.stdout).expect("valid JSON output");
    assert_eq!(value["ok"], true);
    assert_eq!(value["data"]["cli_version"], env!("CARGO_PKG_VERSION"));
    assert!(value["data"]["platform"]["os"].is_string());
    assert!(value["data"]["platform"]["arch"].is_string());
    assert!(value["data"]["checks"].as_array().expect("checks").len() >= 4);

    let check_names: Vec<_> = value["data"]["checks"]
        .as_array()
        .expect("checks")
        .iter()
        .filter_map(|check| check["name"].as_str())
        .collect();
    assert!(check_names.contains(&"rust.toolchain"));
    assert!(check_names.contains(&"cargo.toolchain"));
    assert!(check_names.contains(&"wasm32.target"));
    assert!(check_names.contains(&"demo.dataset"));
    assert!(check_names.contains(&"package.fixture"));

    let status_values: Vec<_> = value["data"]["checks"]
        .as_array()
        .expect("checks")
        .iter()
        .filter_map(|check| check["status"].as_str())
        .collect();
    assert!(status_values
        .iter()
        .all(|status| matches!(*status, "pass" | "warn" | "fail")));
}

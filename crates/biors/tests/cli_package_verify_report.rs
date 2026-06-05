use serde_json::Value;
use std::process::Command;

mod common;
mod package_support;

#[test]
fn package_verify_outputs_fixture_report() {
    let manifest = package_support::example_manifest_path();
    let observations = package_support::example_package_path().join("observations.json");

    let output = Command::new(env!("CARGO_BIN_EXE_biors"))
        .arg("package")
        .arg("verify")
        .arg(manifest)
        .arg(observations)
        .output()
        .expect("run biors package verify");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let value: Value = serde_json::from_slice(&output.stdout).expect("valid JSON output");

    assert_eq!(value["data"]["package"], "protein-seed");
    assert_eq!(value["data"]["fixtures"], 1);
    assert_eq!(value["data"]["passed"], 1);
    assert_eq!(value["data"]["failed"], 0);
    assert_eq!(
        value["data"]["observation_issues"]
            .as_array()
            .expect("observation issues")
            .len(),
        0
    );
    assert_eq!(value["data"]["results"][0]["status"], "passed");
    assert_eq!(
        value["data"]["results"][0]["expected_output_path"],
        "fixtures/tiny.output.json"
    );
    assert_eq!(
        value["data"]["results"][0]["observed_output_path"],
        "observed/tiny.output.json"
    );
    assert_eq!(value["data"]["results"][0]["checksum_mismatch"], false);
    assert_eq!(value["data"]["results"][0]["content_mismatch"], false);
}

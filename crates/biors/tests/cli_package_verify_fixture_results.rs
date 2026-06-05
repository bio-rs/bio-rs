use serde_json::Value;
use std::process::{Command, Stdio};

mod common;
mod package_support;
use common::ChildInputExt;

#[test]
fn package_verify_reports_missing_observed_output_code() {
    let output = Command::new(env!("CARGO_BIN_EXE_biors"))
        .arg("--json")
        .arg("package")
        .arg("verify")
        .arg(package_support::example_manifest_path())
        .arg("-")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn biors package verify")
        .tap_stdin(
            r#"[
              {
                "name": "tiny-protein",
                "path": "observed/missing.json"
              }
            ]"#,
        );

    assert_eq!(output.status.code(), Some(2));
    assert!(output.stderr.is_empty());

    let value: Value = serde_json::from_slice(&output.stdout).expect("valid JSON error");
    assert_eq!(value["error"]["code"], "package.observed_output_missing");
    assert_eq!(value["error"]["location"], "fixtures");
    assert_eq!(
        value["error"]["details"]["results"][0]["issue_code"],
        "observed_output_read_failed"
    );
    assert!(!value["error"]["message"]
        .as_str()
        .expect("message")
        .starts_with('['));
}

#[test]
fn package_verify_reports_content_mismatch_code() {
    let observations = package_support::example_package_path().join("observations.mismatch.json");

    let output = Command::new(env!("CARGO_BIN_EXE_biors"))
        .arg("--json")
        .arg("package")
        .arg("verify")
        .arg(package_support::example_manifest_path())
        .arg(observations)
        .output()
        .expect("run biors package verify");

    assert_eq!(output.status.code(), Some(2));
    assert!(output.stderr.is_empty());

    let value: Value = serde_json::from_slice(&output.stdout).expect("valid JSON error");
    assert_eq!(value["error"]["code"], "package.output_content_mismatch");
    assert_eq!(value["error"]["location"], "fixtures");
    assert_eq!(
        value["error"]["details"]["results"][0]["content_mismatch"],
        true
    );
}

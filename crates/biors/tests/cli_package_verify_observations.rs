use serde_json::Value;
use std::process::{Command, Stdio};

mod common;
mod package_support;
use common::ChildInputExt;

#[test]
fn package_verify_reports_duplicate_observation_code() {
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
                "path": "observed/tiny.output.json"
              },
              {
                "name": "tiny-protein",
                "path": "observed/tiny.reordered.json"
              }
            ]"#,
        );

    assert_eq!(output.status.code(), Some(2));
    assert!(output.stderr.is_empty());

    let value: Value = serde_json::from_slice(&output.stdout).expect("valid JSON error");
    assert_eq!(value["error"]["code"], "package.duplicate_observation");
    assert_eq!(
        value["error"]["details"]["results"][0]["issue_code"],
        "duplicate_observation"
    );
}

#[test]
fn package_verify_reports_unexpected_observation_code() {
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
                "path": "observed/tiny.output.json"
              },
              {
                "name": "stale-output",
                "path": "observed/stale.output.json"
              }
            ]"#,
        );

    assert_eq!(output.status.code(), Some(2));
    assert!(output.stderr.is_empty());

    let value: Value = serde_json::from_slice(&output.stdout).expect("valid JSON error");
    assert_eq!(value["error"]["code"], "package.unexpected_observation");
    assert_eq!(
        value["error"]["details"]["observation_issues"][0]["code"],
        "unexpected_observation"
    );
}

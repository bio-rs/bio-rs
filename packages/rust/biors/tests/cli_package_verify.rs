use serde_json::Value;
use std::process::{Command, Stdio};

mod common;
use common::ChildInputExt;

#[test]
fn package_verify_outputs_fixture_report() {
    let manifest = common::repo_root().join("examples/protein-package/manifest.json");
    let observations = common::repo_root().join("examples/protein-package/observations.json");

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

#[test]
fn package_verify_reports_duplicate_observation_code() {
    let manifest = common::repo_root().join("examples/protein-package/manifest.json");

    let output = Command::new(env!("CARGO_BIN_EXE_biors"))
        .arg("--json")
        .arg("package")
        .arg("verify")
        .arg(manifest)
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
    let manifest = common::repo_root().join("examples/protein-package/manifest.json");

    let output = Command::new(env!("CARGO_BIN_EXE_biors"))
        .arg("--json")
        .arg("package")
        .arg("verify")
        .arg(manifest)
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

#[test]
fn package_verify_reports_missing_observed_output_code() {
    let manifest = common::repo_root().join("examples/protein-package/manifest.json");

    let output = Command::new(env!("CARGO_BIN_EXE_biors"))
        .arg("--json")
        .arg("package")
        .arg("verify")
        .arg(manifest)
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
    let manifest = common::repo_root().join("examples/protein-package/manifest.json");
    let observations =
        common::repo_root().join("examples/protein-package/observations.mismatch.json");

    let output = Command::new(env!("CARGO_BIN_EXE_biors"))
        .arg("--json")
        .arg("package")
        .arg("verify")
        .arg(manifest)
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

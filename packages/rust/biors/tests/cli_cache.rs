use serde_json::Value;
use std::fs;
use std::process::Command;

mod common;
use common::TempDir;

#[test]
fn cache_inspect_reports_artifact_store_policy() {
    let temp = TempDir::new("biors-cache");
    let root = temp.path().join(".biors/artifacts");
    fs::create_dir_all(root.join("datasets")).expect("create cache dirs");
    fs::write(root.join("datasets/example.fasta"), ">seq\nACDE\n").expect("write cache file");

    let output = Command::new(env!("CARGO_BIN_EXE_biors"))
        .arg("cache")
        .arg("inspect")
        .arg("--root")
        .arg(&root)
        .output()
        .expect("run biors cache inspect");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stderr.is_empty());

    let value: Value = serde_json::from_slice(&output.stdout).expect("valid JSON output");
    assert_eq!(value["data"]["action"], "inspect");
    assert_eq!(value["data"]["exists"], true);
    assert_eq!(value["data"]["files"], 1);
    assert_eq!(
        value["data"]["policy"]["environment_variable"],
        "BIORS_ARTIFACT_STORE"
    );
    assert!(value["data"]["layout"]
        .as_array()
        .expect("layout")
        .iter()
        .any(|entry| entry["name"] == "datasets/"));
}

#[test]
fn cache_clean_requires_dry_run_or_confirmation() {
    let temp = TempDir::new("biors-cache-clean");
    let root = temp.path().join(".biors/artifacts");
    fs::create_dir_all(&root).expect("create cache root");

    let output = Command::new(env!("CARGO_BIN_EXE_biors"))
        .arg("--json")
        .arg("cache")
        .arg("clean")
        .arg("--root")
        .arg(&root)
        .output()
        .expect("run biors cache clean without confirmation");

    assert_eq!(output.status.code(), Some(2));
    assert!(output.stderr.is_empty());
    let value: Value = serde_json::from_slice(&output.stdout).expect("valid JSON error");
    assert_eq!(value["error"]["code"], "cache.clean_requires_confirmation");
}

use serde_json::Value;
use std::fs;
use std::process::Command;

mod common;
use common::TempDir;

#[test]
fn cache_inspect_reports_artifact_store_policy() {
    let temp = TempDir::new("biors-cache");
    let root = temp.path().join(".biors/artifacts");
    fs::create_dir_all(root.join("datasets")).unwrap();
    fs::write(root.join("datasets/example.fasta"), ">seq\nACDE\n").unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_biors"))
        .arg("cache")
        .arg("inspect")
        .arg("--root")
        .arg(&root)
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stderr.is_empty());

    let value: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(value["data"]["action"], "inspect");
    assert_eq!(value["data"]["exists"], true);
    assert_eq!(value["data"]["files"], 1);
    assert_eq!(
        value["data"]["policy"]["environment_variable"],
        "BIORS_ARTIFACT_STORE"
    );
    assert_eq!(
        value["data"]["policy"]["clean_requires_artifact_store_root"],
        true
    );
    assert!(value["data"]["layout"]
        .as_array()
        .unwrap()
        .iter()
        .any(|entry| entry["name"] == "datasets/"));
}

#[test]
fn cache_clean_requires_dry_run_or_confirmation() {
    let temp = TempDir::new("biors-cache-clean");
    let root = temp.path().join(".biors/artifacts");
    fs::create_dir_all(&root).unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_biors"))
        .arg("--json")
        .arg("cache")
        .arg("clean")
        .arg("--root")
        .arg(&root)
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(2));
    assert!(output.stderr.is_empty());
    let value: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(value["error"]["code"], "cache.clean_requires_confirmation");
}

#[test]
fn cache_clean_rejects_broad_and_generic_roots() {
    let tmp_output = cache_clean_json_error(std::path::Path::new("/tmp"));
    assert_eq!(tmp_output["error"]["code"], "cache.invalid_root");

    let temp = TempDir::new("biors-cache-generic-root");
    let generic_root = temp.path().join("project");
    fs::create_dir_all(generic_root.join("src")).unwrap();
    fs::write(generic_root.join("not-cache.txt"), "ordinary file\n").unwrap();
    fs::write(generic_root.join("src/main.rs"), "fn main() {}\n").unwrap();

    let generic_output = cache_clean_json_error(&generic_root);
    assert_eq!(generic_output["error"]["code"], "cache.invalid_root");
    assert!(generic_root.join("not-cache.txt").exists());
    assert!(generic_root.join("src/main.rs").exists());
}

#[test]
fn cache_clean_accepts_artifact_store_root() {
    let temp = TempDir::new("biors-cache-valid-root");
    let root = temp.path().join(".biors/artifacts");
    fs::create_dir_all(root.join("packages")).unwrap();
    fs::create_dir_all(root.join("datasets")).unwrap();
    fs::create_dir_all(root.join("locks")).unwrap();
    let cached_file = root.join("datasets/example.fasta");
    fs::write(&cached_file, ">seq\nACDE\n").unwrap();

    let dry_run = Command::new(env!("CARGO_BIN_EXE_biors"))
        .arg("cache")
        .arg("clean")
        .arg("--root")
        .arg(&root)
        .arg("--dry-run")
        .output()
        .unwrap();
    assert!(
        dry_run.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&dry_run.stderr)
    );
    let dry_run_value: Value = serde_json::from_slice(&dry_run.stdout).unwrap();
    assert_eq!(dry_run_value["data"]["removed_files"], 1);
    assert!(cached_file.exists());

    let clean = Command::new(env!("CARGO_BIN_EXE_biors"))
        .arg("cache")
        .arg("clean")
        .arg("--root")
        .arg(&root)
        .arg("--yes")
        .output()
        .unwrap();
    assert!(
        clean.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&clean.stderr)
    );
    assert!(!cached_file.exists());
}

fn cache_clean_json_error(root: &std::path::Path) -> Value {
    let output = Command::new(env!("CARGO_BIN_EXE_biors"))
        .arg("--json")
        .arg("cache")
        .arg("clean")
        .arg("--root")
        .arg(root)
        .arg("--dry-run")
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(2));
    assert!(output.stderr.is_empty());
    serde_json::from_slice(&output.stdout).unwrap()
}

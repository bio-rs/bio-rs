use serde_json::Value;
use std::fs;
use std::process::Command;

mod common;
mod package_support;

#[test]
fn package_verify_rejects_manifest_missing_v1_metadata_before_fixture_compare() {
    let package = package_support::copy_example_package("verify-missing-metadata");
    let manifest_path = package.path().join("manifest.json");
    let observations_path = package.path().join("observations.json");
    let mut manifest = package_support::read_manifest(&manifest_path);
    manifest
        .as_object_mut()
        .expect("manifest object")
        .remove("metadata");
    fs::write(
        &manifest_path,
        serde_json::to_string_pretty(&manifest).expect("serialize manifest"),
    )
    .expect("write invalid manifest");

    let output = Command::new(env!("CARGO_BIN_EXE_biors"))
        .arg("--json")
        .arg("package")
        .arg("verify")
        .arg(manifest_path)
        .arg(observations_path)
        .output()
        .expect("run biors package verify");

    assert_eq!(output.status.code(), Some(2));
    assert!(output.stderr.is_empty());

    let value: Value = serde_json::from_slice(&output.stdout).expect("valid JSON error");
    assert_eq!(value["error"]["code"], "package.validation_failed");
    assert_eq!(value["error"]["location"], "manifest");
    assert_eq!(
        value["error"]["details"]["validation"]["structured_issues"][0]["field"],
        "metadata"
    );
    assert_eq!(
        value["error"]["details"]["validation"]["structured_issues"][0]["code"],
        "required_field"
    );
    assert!(value["error"]["details"]["results"].is_null());
}

#[test]
fn package_verify_rejects_invalid_layout_before_fixture_compare() {
    let package = package_support::copy_example_package("verify-invalid-layout");
    let manifest_path = package.path().join("manifest.json");
    let observations_path = package.path().join("observations.json");
    fs::create_dir_all(package.path().join("artifacts")).expect("create artifacts dir");
    fs::copy(
        package.path().join("models/protein-seed.onnx"),
        package.path().join("artifacts/protein-seed.onnx"),
    )
    .expect("copy model into invalid layout path");

    let mut manifest = package_support::read_manifest(&manifest_path);
    manifest["model"]["path"] = Value::String("artifacts/protein-seed.onnx".to_string());
    fs::write(
        &manifest_path,
        serde_json::to_string_pretty(&manifest).expect("serialize manifest"),
    )
    .expect("write invalid manifest");

    let output = Command::new(env!("CARGO_BIN_EXE_biors"))
        .arg("--json")
        .arg("package")
        .arg("verify")
        .arg(manifest_path)
        .arg(observations_path)
        .output()
        .expect("run biors package verify");

    assert_eq!(output.status.code(), Some(2));
    assert!(output.stderr.is_empty());

    let value: Value = serde_json::from_slice(&output.stdout).expect("valid JSON error");
    assert_eq!(value["error"]["code"], "package.layout_mismatch");
    assert_eq!(value["error"]["location"], "manifest");
    assert_eq!(
        value["error"]["details"]["validation"]["structured_issues"][0]["code"],
        "layout_mismatch"
    );
    assert_eq!(
        value["error"]["details"]["validation"]["structured_issues"][0]["field"],
        "model.path"
    );
}

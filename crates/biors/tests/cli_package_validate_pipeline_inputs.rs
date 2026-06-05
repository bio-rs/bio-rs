use biors_core::hash::sha256_bytes_digest;
use serde_json::Value;
use std::fs;

mod common;
mod package_support;

#[test]
fn package_validate_rejects_pipeline_config_input_that_escapes_package() {
    let temp = common::TempDir::new("invalid-pipeline-escaping-input");
    let package_dir = temp.path().join("package");
    package_support::copy_dir_all(&package_support::example_package_path(), &package_dir);
    fs::write(temp.path().join("outside.fasta"), ">external\nACDE\n").expect("write outside");
    let config_path = package_dir.join("pipelines/protein.toml");
    fs::write(
        &config_path,
        package_support::valid_pipeline_config_with_input("../../outside.fasta"),
    )
    .expect("write pipeline config");
    let manifest_path = package_dir.join("manifest.json");
    let mut manifest = package_support::read_manifest(&manifest_path);
    let config_bytes = fs::read(&config_path).expect("read pipeline config");
    manifest["preprocessing"][0]["config"]["checksum"] =
        Value::String(sha256_bytes_digest(&config_bytes));
    package_support::write_manifest(&manifest_path, &manifest);

    let output = package_support::run_package_validate(&manifest_path);

    assert_eq!(output.status.code(), Some(2));
    assert!(output.stderr.is_empty());
    let value: Value = serde_json::from_slice(&output.stdout).expect("valid JSON error");
    assert_eq!(value["error"]["code"], "package.invalid_pipeline_config");
    package_support::assert_invalid_pipeline_config_issue(&value, "package root");
}

#[test]
fn package_validate_accepts_pipeline_config_input_inside_package() {
    let temp = common::TempDir::new("valid-pipeline-package-input");
    package_support::copy_dir_all(&package_support::example_package_path(), temp.path());
    let manifest_path = temp.path().join("manifest.json");

    let output = package_support::run_package_validate(&manifest_path);

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

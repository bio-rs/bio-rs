#![allow(dead_code, unused_imports)]

mod fixture_data;
mod init;
mod metadata_args;

use biors_core::hash::sha256_bytes_digest;
use serde_json::Value;
use std::fs;
use std::path::Path;
use std::process::{Command, Stdio};

use crate::common;

pub use fixture_data::{valid_dna_vocab_json, V0_MANIFEST};
pub use init::run_package_init_with_model;
pub use metadata_args::{conversion_metadata_args, skeleton_metadata_args};

pub fn copy_dir_all(source: &Path, destination: &Path) {
    fs::create_dir_all(destination).expect("create destination");
    for entry in fs::read_dir(source).expect("read source directory") {
        let entry = entry.expect("read source entry");
        let file_type = entry.file_type().expect("read source file type");
        let destination_path = destination.join(entry.file_name());
        if file_type.is_dir() {
            copy_dir_all(&entry.path(), &destination_path);
        } else {
            fs::copy(entry.path(), destination_path).expect("copy package fixture file");
        }
    }
}

pub fn example_package_path() -> std::path::PathBuf {
    common::repo_root().join("examples/protein-package")
}

pub fn example_manifest_path() -> std::path::PathBuf {
    example_package_path().join("manifest.json")
}

pub fn copy_example_package(name: &str) -> common::TempDir {
    let package = common::TempDir::new(name);
    copy_dir_all(&example_package_path(), package.path());
    package
}

pub fn read_manifest(path: &Path) -> Value {
    serde_json::from_str(&fs::read_to_string(path).expect("read manifest")).expect("manifest JSON")
}

pub fn write_manifest(path: &Path, manifest: &Value) {
    fs::write(path, manifest.to_string()).expect("write manifest");
}

pub fn run_package_validate(manifest_path: &Path) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_biors"))
        .arg("--json")
        .arg("package")
        .arg("validate")
        .arg(manifest_path)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("run biors package validate")
}

pub fn validate_package_with_pipeline_config(name: &str, config: &str) -> Value {
    let temp = common::TempDir::new(name);
    copy_dir_all(&example_package_path(), temp.path());
    let config_path = temp.path().join("pipelines/protein.toml");
    fs::write(&config_path, config).expect("write pipeline config");

    let manifest_path = temp.path().join("manifest.json");
    let mut manifest = read_manifest(&manifest_path);
    let config_bytes = fs::read(&config_path).expect("read pipeline config");
    manifest["preprocessing"][0]["config"]["checksum"] =
        Value::String(sha256_bytes_digest(&config_bytes));
    write_manifest(&manifest_path, &manifest);

    let output = run_package_validate(&manifest_path);
    assert_eq!(output.status.code(), Some(2));
    assert!(output.stderr.is_empty());
    serde_json::from_slice(&output.stdout).expect("valid JSON error")
}

pub fn valid_pipeline_config_with_input(input_path: &str) -> String {
    format!(
        r#"schema_version = "biors.pipeline.v0"
name = "protein-package-fixture-pipeline"

[input]
format = "fasta"
path = "{input_path}"

[normalize]
policy = "strip_ascii_whitespace_uppercase"

[validate]
kind = "protein"

[tokenize]
profile = "protein-20"

[export]
format = "model-input-json"
max_length = 8
pad_token_id = 0
padding = "fixed_length"
"#
    )
}

pub fn validate_package_with_tokenizer_config(
    name: &str,
    config: &str,
    manifest_identity: Option<(&str, &str)>,
) -> Value {
    let output = package_validate_with_tokenizer_config(name, config, manifest_identity);

    assert_eq!(output.status.code(), Some(2));
    assert!(output.stderr.is_empty());
    serde_json::from_slice(&output.stdout).expect("valid JSON error")
}

pub fn package_validate_with_tokenizer_config(
    name: &str,
    config: &str,
    manifest_identity: Option<(&str, &str)>,
) -> std::process::Output {
    let temp = common::TempDir::new(name);
    copy_dir_all(&example_package_path(), temp.path());
    let config_path = temp.path().join("tokenizers/protein-20.json");
    fs::write(&config_path, config).expect("write tokenizer config");

    let manifest_path = temp.path().join("manifest.json");
    let mut manifest = read_manifest(&manifest_path);
    let config_bytes = fs::read(&config_path).expect("read tokenizer config");
    manifest["tokenizer"]["checksum"] = Value::String(sha256_bytes_digest(&config_bytes));
    if let Some((name, contract_version)) = manifest_identity {
        manifest["tokenizer"]["name"] = Value::String(name.to_string());
        manifest["tokenizer"]["contract_version"] = Value::String(contract_version.to_string());
    }
    write_manifest(&manifest_path, &manifest);

    run_package_validate(&manifest_path)
}

pub fn validate_package_with_vocab(
    name: &str,
    vocab: &str,
    manifest_identity: Option<(&str, &str)>,
) -> Value {
    let output = package_validate_with_vocab(name, vocab, manifest_identity);

    assert_eq!(output.status.code(), Some(2));
    assert!(output.stderr.is_empty());
    serde_json::from_slice(&output.stdout).expect("valid JSON error")
}

pub fn package_validate_with_vocab(
    name: &str,
    vocab: &str,
    manifest_identity: Option<(&str, &str)>,
) -> std::process::Output {
    let temp = common::TempDir::new(name);
    copy_dir_all(&example_package_path(), temp.path());
    let vocab_path = temp.path().join("vocabs/protein-20.json");
    fs::write(&vocab_path, vocab).expect("write vocab");

    let manifest_path = temp.path().join("manifest.json");
    let mut manifest = read_manifest(&manifest_path);
    let vocab_bytes = fs::read(&vocab_path).expect("read vocab");
    manifest["vocab"]["checksum"] = Value::String(sha256_bytes_digest(&vocab_bytes));
    if let Some((name, contract_version)) = manifest_identity {
        manifest["vocab"]["name"] = Value::String(name.to_string());
        manifest["vocab"]["contract_version"] = Value::String(contract_version.to_string());
    }
    write_manifest(&manifest_path, &manifest);

    run_package_validate(&manifest_path)
}

pub fn valid_vocab_json() -> String {
    fs::read_to_string(example_package_path().join("vocabs/protein-20.json"))
        .expect("read example vocab")
}

pub fn assert_invalid_pipeline_config_issue(value: &Value, expected_message: &str) {
    let issues = value["error"]["details"]["structured_issues"]
        .as_array()
        .expect("structured issues");
    assert!(issues.iter().any(|issue| {
        issue["code"] == "invalid_pipeline_config"
            && issue["field"] == "preprocessing[0].config"
            && issue["message"]
                .as_str()
                .expect("message")
                .contains(expected_message)
    }));
}

pub fn assert_invalid_tokenizer_config_issue(value: &Value, expected_message: &str) {
    let issues = value["error"]["details"]["structured_issues"]
        .as_array()
        .expect("structured issues");
    assert!(issues.iter().any(|issue| {
        issue["code"] == "invalid_tokenizer_config"
            && issue["message"]
                .as_str()
                .expect("message")
                .contains(expected_message)
    }));
}

pub fn assert_invalid_vocab_config_issue(value: &Value, expected_message: &str) {
    let issues = value["error"]["details"]["structured_issues"]
        .as_array()
        .expect("structured issues");
    assert!(issues.iter().any(|issue| {
        issue["code"] == "invalid_vocab_config"
            && issue["message"]
                .as_str()
                .expect("message")
                .contains(expected_message)
    }));
}

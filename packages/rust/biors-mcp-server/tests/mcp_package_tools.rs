use serde_json::Value;
use std::fs;

mod mcp_support;

use mcp_support::{
    assert_issue_code, assert_json_value_matches_schema, call_tool_error, call_tool_json,
    repo_root, rewrite_manifest_checksum, temp_dir, temp_package,
};

#[tokio::test]
async fn test_package_validate_tool() {
    let mut args = serde_json::Map::new();
    args.insert(
        "manifest_path".to_string(),
        serde_json::Value::String(
            repo_root()
                .join("examples/protein-package/manifest.json")
                .display()
                .to_string(),
        ),
    );

    let json = call_tool_json("package_validate", args).await;
    assert_eq!(json["valid"], true);
    assert_json_value_matches_schema(&json, "schemas/package-validation-report.v0.json");
}

#[tokio::test]
async fn test_package_validate_fields_tool_keeps_field_only_scope() {
    let manifest = fs::read_to_string(repo_root().join("examples/protein-package/manifest.json"))
        .expect("read example manifest");
    let mut args = serde_json::Map::new();
    args.insert(
        "manifest_json".to_string(),
        serde_json::Value::String(manifest),
    );

    let json = call_tool_json("package_validate_fields", args).await;
    assert_eq!(json["valid"], true);
    assert_json_value_matches_schema(&json, "schemas/package-validation-report.v0.json");
}

#[tokio::test]
async fn test_package_validate_tool_requires_filesystem_context() {
    let manifest = fs::read_to_string(repo_root().join("examples/protein-package/manifest.json"))
        .expect("read example manifest");
    let mut args = serde_json::Map::new();
    args.insert(
        "manifest_json".to_string(),
        serde_json::Value::String(manifest),
    );

    let error = call_tool_error("package_validate", args).await;
    assert!(error.contains("requires base_dir"));
}

#[tokio::test]
async fn test_package_validate_tool_reports_missing_artifacts() {
    let temp = temp_dir("mcp-missing-package");
    fs::create_dir_all(&temp).expect("create temp package dir");
    let manifest = fs::read_to_string(repo_root().join("examples/protein-package/manifest.json"))
        .expect("read example manifest");
    let mut args = serde_json::Map::new();
    args.insert(
        "manifest_json".to_string(),
        serde_json::Value::String(manifest),
    );
    args.insert(
        "base_dir".to_string(),
        serde_json::Value::String(temp.display().to_string()),
    );

    let json = call_tool_json("package_validate", args).await;
    assert_eq!(json["valid"], false);
    assert_issue_code(&json, "asset_read_failed");
    assert_json_value_matches_schema(&json, "schemas/package-validation-report.v0.json");
}

#[tokio::test]
async fn test_package_validate_tool_reports_checksum_mismatch() {
    let package_dir = temp_package("mcp-checksum-package");
    fs::write(
        package_dir.join("models/protein-seed.onnx"),
        b"changed model",
    )
    .expect("change model");
    let mut args = serde_json::Map::new();
    args.insert(
        "manifest_path".to_string(),
        serde_json::Value::String(package_dir.join("manifest.json").display().to_string()),
    );

    let json = call_tool_json("package_validate", args).await;
    assert_eq!(json["valid"], false);
    assert_issue_code(&json, "checksum_mismatch");
}

#[tokio::test]
async fn test_package_validate_tool_reports_invalid_tokenizer_config() {
    let package_dir = temp_package("mcp-invalid-tokenizer-package");
    let invalid = br#"{"profile":"protein-20-special","add_special_tokens":false}"#;
    fs::write(package_dir.join("tokenizers/protein-20.json"), invalid)
        .expect("write invalid tokenizer");
    rewrite_manifest_checksum(
        &package_dir.join("manifest.json"),
        &["tokenizer", "checksum"],
        invalid,
    );
    let mut args = serde_json::Map::new();
    args.insert(
        "manifest_path".to_string(),
        serde_json::Value::String(package_dir.join("manifest.json").display().to_string()),
    );

    let json = call_tool_json("package_validate", args).await;
    assert_eq!(json["valid"], false);
    assert_issue_code(&json, "invalid_tokenizer_config");
}

#[tokio::test]
async fn test_package_validate_tool_reports_invalid_pipeline_config() {
    let package_dir = temp_package("mcp-invalid-pipeline-package");
    let invalid = br#"schema_version = "biors.pipeline.v0"
name = "protein"

[input]
format = "fasta"
path = "../outside.fasta"

[normalize]
policy = "strip_ascii_whitespace_uppercase"

[validate]
kind = "protein"

[tokenize]
profile = "protein-20"

[export]
format = "model-input-json"
max_length = 0
"#;
    fs::write(package_dir.join("pipelines/protein.toml"), invalid).expect("write invalid pipeline");
    rewrite_manifest_checksum(
        &package_dir.join("manifest.json"),
        &["preprocessing", "0", "config", "checksum"],
        invalid,
    );
    let mut args = serde_json::Map::new();
    args.insert(
        "manifest_path".to_string(),
        serde_json::Value::String(package_dir.join("manifest.json").display().to_string()),
    );

    let json = call_tool_json("package_validate", args).await;
    assert_eq!(json["valid"], false);
    assert_issue_code(&json, "invalid_pipeline_config");
}

#[tokio::test]
async fn test_package_validate_tool_rejects_unknown_manifest_fields() {
    let mut manifest: Value = serde_json::from_str(
        &fs::read_to_string(repo_root().join("examples/protein-package/manifest.json"))
            .expect("read example manifest"),
    )
    .expect("manifest JSON");
    manifest["unexpected_top"] = Value::Bool(true);

    let mut args = serde_json::Map::new();
    args.insert(
        "manifest_json".to_string(),
        serde_json::Value::String(manifest.to_string()),
    );

    let error = call_tool_error("package_validate_fields", args).await;
    assert!(error.contains("unknown field"));
}

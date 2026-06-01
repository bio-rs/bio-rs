use biors_mcp_server::server::BiorsMcpServer;
use jsonschema::JSONSchema;
use rmcp::model::*;
use rmcp::{ClientHandler, ServiceExt};
use serde_json::Value;
use std::{
    fs,
    path::{Path, PathBuf},
};
use tokio::io::duplex;

#[derive(Default, Clone)]
struct TestClient;

impl ClientHandler for TestClient {}

#[tokio::test]
async fn test_doctor_tool() {
    let json = call_tool_json("doctor", serde_json::Map::new()).await;
    assert_eq!(json["mcp_server_ready"], true);
    assert!(json["biors_version"].as_str().unwrap().starts_with("0."));
}

#[tokio::test]
async fn test_tokenize_tool() {
    let mut args = serde_json::Map::new();
    args.insert(
        "fasta_text".to_string(),
        serde_json::Value::String(">seq1\nACDEFGHIKLMNPQRSTVWY\n".to_string()),
    );
    args.insert(
        "profile".to_string(),
        serde_json::Value::String("protein-20".to_string()),
    );
    let json = call_tool_json("tokenize", args).await;
    assert!(json.is_array());
    assert_eq!(json[0]["id"], "seq1");
}

#[tokio::test]
async fn test_tokenize_tool_accepts_nucleotide_profiles() {
    let mut args = serde_json::Map::new();
    args.insert(
        "fasta_text".to_string(),
        serde_json::Value::String(">dna\nACGTN\n".to_string()),
    );
    args.insert(
        "profile".to_string(),
        serde_json::Value::String("dna-iupac".to_string()),
    );
    let json = call_tool_json("tokenize", args).await;
    assert_eq!(json[0]["alphabet"], "dna-iupac");
    assert_eq!(json[0]["tokens"], serde_json::json!([0, 1, 2, 3, 4]));
    assert_eq!(json[0]["valid"], false);
}

#[tokio::test]
async fn test_validate_tool() {
    let mut args = serde_json::Map::new();
    args.insert(
        "fasta_text".to_string(),
        serde_json::Value::String(">seq1\nACDEFGHIKLMNPQRSTVWY\n".to_string()),
    );
    args.insert(
        "kind".to_string(),
        serde_json::Value::String("protein".to_string()),
    );
    let json = call_tool_json("validate", args).await;
    assert_eq!(json["records"], 1);
}

#[tokio::test]
async fn test_workflow_tool_matches_core_contract_defaults() {
    let mut args = serde_json::Map::new();
    args.insert(
        "fasta_text".to_string(),
        serde_json::Value::String(">seq1\nACDE\n".to_string()),
    );
    args.insert("max_length".to_string(), serde_json::json!(6));

    let json = call_tool_json("workflow", args).await;
    assert_eq!(json["workflow"], "protein_model_input.v0");
    assert_eq!(json["model_ready"], true);
    assert_eq!(
        json["provenance"]["invocation"]["command"],
        "biors-mcp workflow"
    );
    assert_eq!(
        json["provenance"]["model_input_policy"]["padding"],
        "fixed_length"
    );
    assert_eq!(json["provenance"]["model_input_policy"]["pad_token_id"], 0);
    assert!(json["provenance"]["input_hash"]
        .as_str()
        .expect("input_hash")
        .starts_with("fnv1a64:"));
    assert_eq!(
        json["model_input"]["records"][0]["input_ids"],
        serde_json::json!([0, 1, 2, 3, 0, 0])
    );
    assert_json_value_matches_schema(&json, "schemas/sequence-workflow-output.v0.json");
}

#[tokio::test]
async fn test_workflow_tool_accepts_nucleotide_profiles() {
    let mut args = serde_json::Map::new();
    args.insert(
        "fasta_text".to_string(),
        serde_json::Value::String(">dna\nACGT\n".to_string()),
    );
    args.insert(
        "kind".to_string(),
        serde_json::Value::String("dna".to_string()),
    );
    args.insert(
        "profile".to_string(),
        serde_json::Value::String("dna-iupac".to_string()),
    );
    args.insert("max_length".to_string(), serde_json::json!(6));

    let json = call_tool_json("workflow", args).await;
    assert_eq!(json["workflow"], "sequence_model_input.v0");
    assert_eq!(json["model_ready"], true);
    assert_eq!(json["provenance"]["tokenizer"]["name"], "dna-iupac");
    assert_eq!(json["provenance"]["validation_alphabet"], "dna-iupac");
    assert_eq!(
        json["model_input"]["records"][0]["input_ids"],
        serde_json::json!([0, 1, 2, 3, 0, 0])
    );
    assert_json_value_matches_schema(&json, "schemas/sequence-workflow-output.v0.json");

    let mut auto_args = serde_json::Map::new();
    auto_args.insert(
        "fasta_text".to_string(),
        serde_json::Value::String(">rna\nACGU\n".to_string()),
    );
    auto_args.insert("max_length".to_string(), serde_json::json!(4));
    let auto_json = call_tool_json("workflow", auto_args).await;
    assert_eq!(auto_json["provenance"]["tokenizer"]["name"], "rna-iupac");
}

#[tokio::test]
async fn test_workflow_tool_reports_non_model_ready_residues() {
    let mut args = serde_json::Map::new();
    args.insert(
        "fasta_text".to_string(),
        serde_json::Value::String(">seq1\nAC*X\n".to_string()),
    );
    args.insert("max_length".to_string(), serde_json::json!(8));

    let json = call_tool_json("workflow", args).await;
    assert_eq!(json["model_ready"], false);
    assert!(json["model_input"].is_null());
    assert_eq!(
        json["readiness_issues"][0]["code"],
        "sequence.not_model_ready"
    );
}

#[tokio::test]
async fn test_workflow_tool_rejects_empty_sequence_records() {
    let mut args = serde_json::Map::new();
    args.insert(
        "fasta_text".to_string(),
        serde_json::Value::String(">empty\n".to_string()),
    );
    args.insert("max_length".to_string(), serde_json::json!(8));

    let error = call_tool_error("workflow", args).await;
    assert!(error.contains("missing sequence") || error.contains("empty"));
}

#[tokio::test]
async fn test_sequence_tools_classify_invalid_fasta_as_invalid_params() {
    for tool_name in ["tokenize", "validate", "workflow"] {
        let mut args = serde_json::Map::new();
        args.insert(
            "fasta_text".to_string(),
            serde_json::Value::String("ACDE\n".to_string()),
        );
        if tool_name == "workflow" {
            args.insert("max_length".to_string(), serde_json::json!(8));
        }

        let error = call_tool_error_debug(tool_name, args).await;
        assert!(
            error.contains("ErrorCode(-32602)"),
            "{tool_name} did not return invalid params code: {error}"
        );
        assert!(
            error.contains("fasta.missing_header"),
            "{tool_name} did not include FASTA diagnostic code: {error}"
        );
    }
}

#[tokio::test]
async fn test_validate_tool_classifies_empty_input_as_invalid_params() {
    let mut args = serde_json::Map::new();
    args.insert(
        "fasta_text".to_string(),
        serde_json::Value::String("".to_string()),
    );

    let error = call_tool_error_debug("validate", args).await;
    assert!(error.contains("ErrorCode(-32602)"));
    assert!(error.contains("fasta.empty_input"));
}

#[tokio::test]
async fn test_workflow_tool_rejects_kind_profile_mismatch_and_bad_padding() {
    let mut dna_args = serde_json::Map::new();
    dna_args.insert(
        "fasta_text".to_string(),
        serde_json::Value::String(">seq1\nACGT\n".to_string()),
    );
    dna_args.insert(
        "kind".to_string(),
        serde_json::Value::String("dna".to_string()),
    );
    dna_args.insert(
        "profile".to_string(),
        serde_json::Value::String("protein-20".to_string()),
    );
    let dna_error = call_tool_error("workflow", dna_args).await;
    assert!(dna_error.contains("workflow kind/profile mismatch"));

    let mut padding_args = serde_json::Map::new();
    padding_args.insert(
        "fasta_text".to_string(),
        serde_json::Value::String(">seq1\nACDE\n".to_string()),
    );
    padding_args.insert(
        "padding".to_string(),
        serde_json::Value::String("bad".to_string()),
    );
    let padding_error = call_tool_error("workflow", padding_args).await;
    assert!(padding_error.contains("invalid padding"));
}

#[tokio::test]
async fn test_tokenize_tool_rejects_invalid_profile() {
    let mut args = serde_json::Map::new();
    args.insert(
        "fasta_text".to_string(),
        serde_json::Value::String(">seq1\nACDE\n".to_string()),
    );
    args.insert(
        "profile".to_string(),
        serde_json::Value::String("dna".to_string()),
    );
    let error = call_tool_error("tokenize", args).await;
    assert!(error.contains("invalid profile"));
}

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
        &std::fs::read_to_string(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../..")
                .join("examples/protein-package/manifest.json"),
        )
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

fn assert_issue_code(value: &Value, expected_code: &str) {
    assert!(
        value["structured_issues"]
            .as_array()
            .expect("structured issues")
            .iter()
            .any(|issue| issue["code"] == expected_code),
        "missing issue code {expected_code}: {value}"
    );
}

fn rewrite_manifest_checksum(manifest_path: &Path, path: &[&str], bytes: &[u8]) {
    let mut manifest: Value =
        serde_json::from_str(&fs::read_to_string(manifest_path).expect("read manifest"))
            .expect("manifest JSON");
    let mut cursor = &mut manifest;
    for segment in &path[..path.len() - 1] {
        cursor = if let Ok(index) = segment.parse::<usize>() {
            &mut cursor[index]
        } else {
            &mut cursor[*segment]
        };
    }
    cursor[path[path.len() - 1]] = Value::String(biors_core::hash::sha256_bytes_digest(bytes));
    fs::write(
        manifest_path,
        serde_json::to_string_pretty(&manifest).expect("serialize manifest"),
    )
    .expect("write manifest");
}

fn temp_package(name: &str) -> PathBuf {
    let destination = temp_dir(name);
    copy_dir_recursive(&repo_root().join("examples/protein-package"), &destination);
    destination
}

fn temp_dir(name: &str) -> PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("clock after epoch")
        .as_nanos();
    std::env::temp_dir().join(format!("{name}-{nanos}"))
}

fn copy_dir_recursive(source: &Path, destination: &Path) {
    fs::create_dir_all(destination).expect("create destination directory");
    for entry in fs::read_dir(source).expect("read source directory") {
        let entry = entry.expect("read directory entry");
        let source_path = entry.path();
        let destination_path = destination.join(entry.file_name());
        if entry.file_type().expect("read file type").is_dir() {
            copy_dir_recursive(&source_path, &destination_path);
        } else {
            fs::copy(&source_path, &destination_path).expect("copy file");
        }
    }
}

fn assert_json_value_matches_schema(value: &Value, schema_path: &str) {
    let schema: Value = serde_json::from_str(
        &fs::read_to_string(repo_root().join(schema_path)).expect("read payload schema"),
    )
    .expect("schema JSON");
    let compiled = JSONSchema::compile(&schema).expect("compile schema");
    let errors: Vec<_> = compiled
        .validate(value)
        .err()
        .into_iter()
        .flat_map(|errors| errors.map(|error| error.to_string()))
        .collect();
    assert!(
        errors.is_empty(),
        "JSON did not match schema {schema_path}: {errors:?}"
    );
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../..")
}

async fn call_tool_json(
    name: &str,
    args: serde_json::Map<String, serde_json::Value>,
) -> serde_json::Value {
    let response = call_tool(name, args)
        .await
        .expect("tool call should succeed");
    let ServerResult::CallToolResult(result) = response else {
        panic!("expected CallToolResult, got {response:?}");
    };

    let text = result
        .content
        .into_iter()
        .find_map(|c| c.as_text().map(|t| t.text.to_string()))
        .expect("expected text content");

    serde_json::from_str(&text).expect("tool returned JSON text")
}

async fn call_tool_error(name: &str, args: serde_json::Map<String, serde_json::Value>) -> String {
    call_tool(name, args)
        .await
        .expect_err("tool call should fail")
        .to_string()
}

async fn call_tool_error_debug(
    name: &str,
    args: serde_json::Map<String, serde_json::Value>,
) -> String {
    format!(
        "{:?}",
        call_tool(name, args)
            .await
            .expect_err("tool call should fail")
    )
}

async fn call_tool(
    name: &str,
    args: serde_json::Map<String, serde_json::Value>,
) -> Result<ServerResult, rmcp::service::ServiceError> {
    let server = BiorsMcpServer;
    let (server_transport, client_transport) = duplex(4096);

    let _server_handle = tokio::spawn(async move {
        let service = server.serve(server_transport).await.unwrap();
        service.waiting().await.unwrap();
    });

    let client = TestClient;
    let client_service = client.serve(client_transport).await.unwrap();

    let params = if args.is_empty() {
        CallToolRequestParams::new(name.to_string())
    } else {
        CallToolRequestParams::new(name.to_string()).with_arguments(args)
    };
    let response = client_service
        .send_request(ClientRequest::CallToolRequest(Request::new(params)))
        .await;

    client_service.cancel().await.unwrap();
    response
}

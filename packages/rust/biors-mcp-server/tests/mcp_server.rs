use biors_mcp_server::server::BiorsMcpServer;
use jsonschema::JSONSchema;
use rmcp::model::*;
use rmcp::{ClientHandler, ServiceExt};
use serde_json::Value;
use std::{fs, path::PathBuf};
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
async fn test_workflow_tool_rejects_unsupported_kind_and_padding() {
    let mut dna_args = serde_json::Map::new();
    dna_args.insert(
        "fasta_text".to_string(),
        serde_json::Value::String(">seq1\nACGT\n".to_string()),
    );
    dna_args.insert(
        "kind".to_string(),
        serde_json::Value::String("dna".to_string()),
    );
    let dna_error = call_tool_error("workflow", dna_args).await;
    assert!(dna_error.contains("unsupported workflow kind"));

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
    let manifest = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../..")
            .join("examples/protein-package/manifest.json"),
    )
    .expect("read example manifest");
    let mut args = serde_json::Map::new();
    args.insert(
        "manifest_json".to_string(),
        serde_json::Value::String(manifest),
    );

    let json = call_tool_json("package_validate", args).await;
    assert_eq!(json["valid"], true);
    assert_json_value_matches_schema(&json, "schemas/package-validation-report.v0.json");
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

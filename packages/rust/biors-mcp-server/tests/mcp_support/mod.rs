#![allow(dead_code)]

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

pub fn assert_issue_code(value: &Value, expected_code: &str) {
    assert!(
        value["structured_issues"]
            .as_array()
            .expect("structured issues")
            .iter()
            .any(|issue| issue["code"] == expected_code),
        "missing issue code {expected_code}: {value}"
    );
}

pub fn rewrite_manifest_checksum(manifest_path: &Path, path: &[&str], bytes: &[u8]) {
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

pub fn temp_package(name: &str) -> PathBuf {
    let destination = temp_dir(name);
    copy_dir_recursive(&repo_root().join("examples/protein-package"), &destination);
    destination
}

pub fn temp_dir(name: &str) -> PathBuf {
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

pub fn assert_json_value_matches_schema(value: &Value, schema_path: &str) {
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

pub fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../..")
}

pub async fn call_tool_json(
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

pub async fn call_tool_error(
    name: &str,
    args: serde_json::Map<String, serde_json::Value>,
) -> String {
    call_tool(name, args)
        .await
        .expect_err("tool call should fail")
        .to_string()
}

pub async fn call_tool_error_debug(
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

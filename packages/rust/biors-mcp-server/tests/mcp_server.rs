use biors_mcp_server::server::BiorsMcpServer;
use rmcp::model::*;
use rmcp::{ClientHandler, ServiceExt};
use tokio::io::duplex;

#[derive(Default, Clone)]
struct TestClient;

impl ClientHandler for TestClient {}

#[tokio::test]
async fn test_doctor_tool() {
    let server = BiorsMcpServer;
    let (server_transport, client_transport) = duplex(4096);

    let _server_handle = tokio::spawn(async move {
        let service = server.serve(server_transport).await.unwrap();
        service.waiting().await.unwrap();
    });

    let client = TestClient;
    let client_service = client.serve(client_transport).await.unwrap();

    let params = CallToolRequestParams::new("doctor");
    let response = client_service
        .send_request(ClientRequest::CallToolRequest(Request::new(params)))
        .await
        .unwrap();

    let ServerResult::CallToolResult(result) = response else {
        panic!("expected CallToolResult, got {response:?}");
    };

    let text = result
        .content
        .into_iter()
        .find_map(|c| c.as_text().map(|t| t.text.to_string()))
        .expect("expected text content");

    let json: serde_json::Value = serde_json::from_str(&text).unwrap();
    assert_eq!(json["mcp_server_ready"], true);
    assert!(json["biors_version"].as_str().unwrap().starts_with("0."));

    client_service.cancel().await.unwrap();
}

#[tokio::test]
async fn test_tokenize_tool() {
    let server = BiorsMcpServer;
    let (server_transport, client_transport) = duplex(4096);

    let _server_handle = tokio::spawn(async move {
        let service = server.serve(server_transport).await.unwrap();
        service.waiting().await.unwrap();
    });

    let client = TestClient;
    let client_service = client.serve(client_transport).await.unwrap();

    let mut args = serde_json::Map::new();
    args.insert(
        "fasta_text".to_string(),
        serde_json::Value::String(">seq1\nACDEFGHIKLMNPQRSTVWY\n".to_string()),
    );
    args.insert(
        "profile".to_string(),
        serde_json::Value::String("protein-20".to_string()),
    );
    let params = CallToolRequestParams::new("tokenize").with_arguments(args);
    let response = client_service
        .send_request(ClientRequest::CallToolRequest(Request::new(params)))
        .await
        .unwrap();

    let ServerResult::CallToolResult(result) = response else {
        panic!("expected CallToolResult, got {response:?}");
    };

    let text = result
        .content
        .into_iter()
        .find_map(|c| c.as_text().map(|t| t.text.to_string()))
        .expect("expected text content");

    let json: serde_json::Value = serde_json::from_str(&text).unwrap();
    assert!(json.is_array());
    assert_eq!(json[0]["id"], "seq1");

    client_service.cancel().await.unwrap();
}

#[tokio::test]
async fn test_validate_tool() {
    let server = BiorsMcpServer;
    let (server_transport, client_transport) = duplex(4096);

    let _server_handle = tokio::spawn(async move {
        let service = server.serve(server_transport).await.unwrap();
        service.waiting().await.unwrap();
    });

    let client = TestClient;
    let client_service = client.serve(client_transport).await.unwrap();

    let mut args = serde_json::Map::new();
    args.insert(
        "fasta_text".to_string(),
        serde_json::Value::String(">seq1\nACDEFGHIKLMNPQRSTVWY\n".to_string()),
    );
    args.insert(
        "kind".to_string(),
        serde_json::Value::String("protein".to_string()),
    );
    let params = CallToolRequestParams::new("validate").with_arguments(args);
    let response = client_service
        .send_request(ClientRequest::CallToolRequest(Request::new(params)))
        .await
        .unwrap();

    let ServerResult::CallToolResult(result) = response else {
        panic!("expected CallToolResult, got {response:?}");
    };

    let text = result
        .content
        .into_iter()
        .find_map(|c| c.as_text().map(|t| t.text.to_string()))
        .expect("expected text content");

    let json: serde_json::Value = serde_json::from_str(&text).unwrap();
    assert_eq!(json["records"], 1);

    client_service.cancel().await.unwrap();
}

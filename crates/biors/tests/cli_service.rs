use serde_json::Value;
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::process::Child;
use std::time::{Duration, Instant};

mod common;

#[test]
fn service_contract_outputs_stable_json_boundary() {
    let output = common::run_biors_paths(&["service", "contract"], &[]);
    let value: Value = serde_json::from_slice(&output.stdout).expect("valid JSON output");

    assert_eq!(value["ok"], true);
    assert_eq!(
        value["data"]["schema_version"],
        "biors.service_interface.v0"
    );
    assert_eq!(value["data"]["server_runtime"], "cli_local_http_server");
    assert_eq!(value["data"]["openapi"]["status"], "served_by_cli_runtime");
    assert!(value["data"]["routes"]
        .as_array()
        .expect("routes")
        .iter()
        .any(|route| route["operation_id"] == "sequence.batch_validate"));
    assert!(value["data"]["routes"]
        .as_array()
        .expect("routes")
        .iter()
        .any(|route| route["operation_id"] == "package.bridge.plan"));
}

#[test]
fn serve_exposes_health_openapi_and_batch_validation() {
    let port = allocate_port();
    let mut child =
        common::spawn_biors(&["serve", "--host", "127.0.0.1", "--port", &port.to_string()]);
    let address: SocketAddr = format!("127.0.0.1:{port}").parse().expect("socket");
    wait_for_server(address, &mut child);

    let health = http_request(address, "GET /health HTTP/1.1\r\nHost: 127.0.0.1\r\n\r\n");
    assert!(health.starts_with("HTTP/1.1 200 OK"), "{health}");
    let health_body = response_body(&health);
    let health_json: Value = serde_json::from_str(health_body).expect("health JSON");
    assert_eq!(health_json["schema_version"], "biors.service_health.v0");
    common::assert_json_value_matches_schema(&health_json, "schemas/service-health-output.v0.json");

    let openapi = http_request(
        address,
        "GET /openapi.json HTTP/1.1\r\nHost: 127.0.0.1\r\n\r\n",
    );
    assert!(openapi.starts_with("HTTP/1.1 200 OK"), "{openapi}");
    let openapi_json: Value = serde_json::from_str(response_body(&openapi)).expect("OpenAPI JSON");
    assert_eq!(openapi_json["openapi"], "3.1.0");
    common::assert_json_value_matches_schema(
        &openapi_json,
        "schemas/service-openapi-output.v0.json",
    );

    let payload = r#"{"kind":"protein","inputs":[{"id":"sample1","fasta_text":">seq1\nACDE\n"}]}"#;
    let request = format!(
        "POST /v0/batch/sequence/validate HTTP/1.1\r\nHost: 127.0.0.1\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
        payload.len(),
        payload
    );
    let batch = http_request(address, &request);
    assert!(batch.starts_with("HTTP/1.1 200 OK"), "{batch}");
    let batch_json: Value = serde_json::from_str(response_body(&batch)).expect("batch JSON");
    assert_eq!(
        batch_json["schema_version"],
        "biors.service_batch_sequence_validate.v0"
    );
    assert_eq!(batch_json["summary"]["fasta_records"], 1);
    common::assert_json_value_matches_schema(
        &batch_json,
        "schemas/service-batch-sequence-validate-output.v0.json",
    );

    let _ = child.kill();
    let _ = child.wait();
}

fn allocate_port() -> u16 {
    std::net::TcpListener::bind("127.0.0.1:0")
        .expect("allocate test port")
        .local_addr()
        .expect("local address")
        .port()
}

fn wait_for_server(address: SocketAddr, child: &mut Child) {
    let deadline = Instant::now() + Duration::from_secs(20);
    while Instant::now() < deadline {
        if TcpStream::connect_timeout(&address, Duration::from_millis(100)).is_ok() {
            return;
        }
        if let Some(status) = child.try_wait().expect("poll service child") {
            let mut stderr = String::new();
            child
                .stderr
                .as_mut()
                .expect("service stderr")
                .read_to_string(&mut stderr)
                .expect("read service stderr");
            panic!("server exited before start with {status}: {stderr}");
        }
        std::thread::sleep(Duration::from_millis(25));
    }
    let _ = child.kill();
    let mut stderr = String::new();
    child
        .stderr
        .as_mut()
        .expect("service stderr")
        .read_to_string(&mut stderr)
        .expect("read service stderr");
    panic!("server did not start at {address}; stderr: {stderr}");
}

fn http_request(address: SocketAddr, request: &str) -> String {
    let mut stream =
        TcpStream::connect_timeout(&address, Duration::from_secs(2)).expect("connect to service");
    stream.write_all(request.as_bytes()).expect("write request");
    let mut response = String::new();
    stream.read_to_string(&mut response).expect("read response");
    response
}

fn response_body(response: &str) -> &str {
    response
        .split_once("\r\n\r\n")
        .map(|(_, body)| body)
        .expect("response body")
}

use biors_mcp_server::server::BiorsMcpServer;
use criterion::{criterion_group, criterion_main, Criterion};
use rmcp::model::{CallToolRequestParams, ClientRequest, Request};
use rmcp::{ClientHandler, ServiceExt};
use tokio::io::duplex;
use tokio::runtime::Runtime;

#[derive(Default, Clone)]
struct BenchClient;

impl ClientHandler for BenchClient {}

fn bench_mcp_request_overhead(c: &mut Criterion) {
    let runtime = Runtime::new().expect("tokio runtime");
    let (server_transport, client_transport) = duplex(64 * 1024);
    let _server_handle = runtime.spawn(async move {
        let service = BiorsMcpServer.serve(server_transport).await.unwrap();
        service.waiting().await.unwrap();
    });
    let client_service = runtime
        .block_on(BenchClient.serve(client_transport))
        .expect("mcp client");

    c.bench_function("mcp_doctor_request_duplex", |b| {
        b.iter(|| {
            runtime
                .block_on(client_service.send_request(ClientRequest::CallToolRequest(
                    Request::new(CallToolRequestParams::new("doctor")),
                )))
                .expect("doctor request")
        })
    });

    runtime
        .block_on(client_service.cancel())
        .expect("cancel mcp client");
}

criterion_group!(benches, bench_mcp_request_overhead);
criterion_main!(benches);

use biors_mcp_server::server::BiorsMcpServer;
use rmcp::{transport::stdio, ServiceExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let server = BiorsMcpServer;
    let service = server.serve(stdio()).await?;
    service.waiting().await?;
    Ok(())
}

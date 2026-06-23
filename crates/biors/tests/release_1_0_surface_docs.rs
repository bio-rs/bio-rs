use std::fs;
use std::process::Command;

mod common;

#[test]
fn product_surface_docs_account_for_current_surfaces() {
    let repo = common::repo_root();
    let versioning =
        fs::read_to_string(repo.join("docs/versioning.md")).expect("read versioning doc");
    let cli_contract =
        fs::read_to_string(repo.join("docs/cli-contract.md")).expect("read CLI contract");
    let service_doc =
        fs::read_to_string(repo.join("docs/service-interface.md")).expect("read service doc");
    let mcp_catalog = fs::read_to_string(repo.join("crates/biors-mcp-server/README.md"))
        .expect("read MCP README");

    for required in [
        "Primary researcher interface",
        "Primary agent interface",
        "Embedding interface",
        "Secondary local integration",
        "Package/artifact assurance",
        "Preview/internal",
        "Schema Versioning",
        "product workflow stability",
    ] {
        assert!(
            versioning.contains(required),
            "versioning doc missing required surface text: {required}"
        );
    }

    for command in cli_commands() {
        if command == "help" {
            continue;
        }
        assert!(
            cli_contract.contains(&format!("`biors {command}")),
            "CLI contract missing CLI command row: {command}"
        );
    }

    let mcp_server = fs::read_to_string(repo.join("crates/biors-mcp-server/src/server.rs"))
        .expect("read MCP server");
    for tool in mcp_tools(&mcp_server) {
        assert!(
            mcp_catalog.contains(&format!("| `{tool}` |")),
            "MCP agent catalog missing tool row: {tool}"
        );
    }
    for required in [
        "include_records",
        "include_payload",
        "biors.mcp.compact.v0",
        "summary/counts/issues by default",
        "not an autonomous research agent",
    ] {
        assert!(
            mcp_catalog.contains(required),
            "MCP agent catalog missing compact-output policy text: {required}"
        );
    }

    for route in service_routes(&repo) {
        assert!(
            service_doc.contains(&format!("| `{route}` |")),
            "service doc missing service route row: {route}"
        );
    }
}

fn cli_commands() -> Vec<String> {
    let output = Command::new(env!("CARGO_BIN_EXE_biors"))
        .arg("--help")
        .output()
        .expect("run biors help");
    assert!(output.status.success());

    let help = String::from_utf8(output.stdout).expect("help is UTF-8");
    let mut commands = Vec::new();
    let mut in_commands = false;
    for line in help.lines() {
        if line == "Commands:" {
            in_commands = true;
            continue;
        }
        if line == "Options:" {
            break;
        }
        if in_commands {
            if let Some(command) = line.split_whitespace().next() {
                commands.push(command.to_string());
            }
        }
    }
    commands
}

fn mcp_tools(source: &str) -> Vec<String> {
    let mut tools = Vec::new();
    let mut pending_tool = false;
    for line in source.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("#[tool") {
            pending_tool = true;
            continue;
        }
        if pending_tool {
            if let Some(rest) = trimmed.strip_prefix("fn ") {
                if let Some((name, _)) = rest.split_once('(') {
                    tools.push(name.to_string());
                    pending_tool = false;
                }
            }
        }
    }
    tools.sort();
    tools
}

fn service_routes(repo: &std::path::Path) -> Vec<String> {
    let service_doc =
        fs::read_to_string(repo.join("docs/service-interface.md")).expect("read service doc");
    let mut routes = Vec::new();
    for line in service_doc.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("| `") {
            if let Some((route, _)) = rest.split_once('`') {
                if route.starts_with("GET ") || route.starts_with("POST ") {
                    routes.push(route.to_string());
                }
            }
        }
    }
    routes.sort();
    routes
}

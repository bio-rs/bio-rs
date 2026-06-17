use std::fs;
use std::process::Command;

mod common;

#[test]
fn product_role_doc_accounts_for_current_surfaces() {
    let repo = common::repo_root();
    let doc =
        fs::read_to_string(repo.join("docs/1-0-stability.md")).expect("read 1.0 stability doc");

    for required in [
        "primary researcher interface",
        "primary agent interface",
        "embedding interface",
        "secondary local integration",
        "package/artifact assurance",
        "preview/internal",
        "public stable",
        "public experimental",
        "internal-only",
        "candidate for merge",
        "product 1.0 classification",
    ] {
        assert!(
            doc.contains(required),
            "1.0 stability doc missing required classification text: {required}"
        );
    }

    for crate_name in workspace_crates(&repo) {
        assert!(
            doc.contains(&format!("| `{crate_name}` |")),
            "1.0 stability doc missing workspace crate row: {crate_name}"
        );
    }

    for command in cli_commands() {
        if command == "help" {
            continue;
        }
        assert!(
            doc.contains(&format!("| `biors {command}` |")),
            "1.0 stability doc missing CLI command row: {command}"
        );
    }

    let mcp_server = fs::read_to_string(repo.join("crates/biors-mcp-server/src/server.rs"))
        .expect("read MCP server");
    let mcp_catalog =
        fs::read_to_string(repo.join("docs/mcp-agent-tools.md")).expect("read MCP catalog");
    for tool in mcp_tools(&mcp_server) {
        assert!(
            doc.contains(&format!("| `{tool}` |")),
            "1.0 stability doc missing MCP tool row: {tool}"
        );
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
            doc.contains(&format!("| `{route}` |")),
            "1.0 stability doc missing service route row: {route}"
        );
    }

    for schema in schema_files(&repo) {
        let marker = format!("| `{schema}` |");
        assert_eq!(
            doc.matches(&marker).count(),
            1,
            "schema must appear in exactly one stability row: {schema}"
        );
    }
}

fn workspace_crates(repo: &std::path::Path) -> Vec<String> {
    let crates_dir = repo.join("crates");
    let mut crates = Vec::new();
    for entry in fs::read_dir(crates_dir).expect("read crates dir") {
        let entry = entry.expect("read crate entry");
        let manifest = entry.path().join("Cargo.toml");
        if !manifest.exists() {
            continue;
        }
        let contents = fs::read_to_string(manifest).expect("read crate manifest");
        for line in contents.lines() {
            let trimmed = line.trim();
            if let Some(name) = trimmed.strip_prefix("name = \"") {
                crates.push(name.trim_end_matches('"').to_string());
                break;
            }
        }
    }
    crates.sort();
    crates
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

fn schema_files(repo: &std::path::Path) -> Vec<String> {
    let mut schemas = Vec::new();
    for entry in fs::read_dir(repo.join("schemas")).expect("read schemas dir") {
        let entry = entry.expect("read schema entry");
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) == Some("json") {
            schemas.push(
                path.file_name()
                    .and_then(|name| name.to_str())
                    .expect("schema file name")
                    .to_string(),
            );
        }
    }
    schemas.sort();
    schemas
}

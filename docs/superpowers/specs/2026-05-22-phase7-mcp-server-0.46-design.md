# bio-rs v0.46.0 MCP Server Integration Design Document

## 1. Executive Summary

This document designs an agent-callable tool interface and Model Context Protocol (MCP) server integration for bio-rs. The MCP server exposes bio-rs CLI commands as deterministic, schema-validated tools that AI agents (Claude, Cursor, etc.) can discover and invoke via the MCP specification (version 2025-06-18). The server is implemented as a **new standalone crate** (`biors-mcp-server`) that reuses existing `biors-core` and `biors` domain logic without duplicating business rules or forking the CLI binary.

---

## 2. MCP Specification Research Summary

### 2.1 Protocol Foundation
- **Transport**: JSON-RPC 2.0 over stdio (primary for local agent integration) or Streamable HTTP
- **Lifecycle**: `initialize` -> capability negotiation -> `tools/list` -> `tools/call`
- **Per-request metadata**: `_meta` fields carrying `protocolVersion`, `clientInfo`, `clientCapabilities`

### 2.2 Tool Contract
- **Discovery**: `tools/list` returns tool definitions with `name`, `title`, `description`, `inputSchema` (JSON Schema 2020-12), optional `outputSchema`, and optional `annotations`
- **Invocation**: `tools/call` receives `name` and `arguments`; returns `content` array (text/image/audio/resource) plus `isError` boolean
- **Structured output**: Tools may return `structuredContent` (JSON object) alongside text content; if `outputSchema` is declared, servers MUST conform to it
- **Error model**: Two layers:
  1. **Protocol errors**: JSON-RPC error codes (`-32602` invalid params, `-32601` method not found, etc.)
  2. **Tool execution errors**: `isError: true` in the result payload for business-logic failures

### 2.3 Capability Declaration
Servers MUST declare:
```json
{
  "capabilities": {
    "tools": { "listChanged": false }
  }
}
```

### 2.4 Rust SDK Landscape
| SDK | Maturity | Recommendation |
|-----|----------|----------------|
| `rmcp` (official Anthropic) | High | **Primary choice** -- official SDK, tokio-based, `#[tool]` macros, `schemars` integration, stdio/HTTP transports |
| `rust-mcp-sdk` | High | Community alternative, full 2025-11-25 spec, Axum HTTP server |
| `mcp-protocol-sdk` | Medium | Production-ready, explicit `ToolHandler` trait |
| `tmcp` | Medium | Ergonomic macros, OAuth support |
| `mcp-core` | Lower | Early community effort |

**Decision**: Use `rmcp` as the primary SDK dependency. It is the official Anthropic Rust SDK, actively maintained, supports stdio transport natively, and provides procedural macros (`#[tool]`, `#[tool_router]`) that reduce boilerplate when mapping bio-rs commands to MCP tools.

---

## 3. Tool Manifest Schema Definition

### 3.1 `ToolManifest` Struct (Rust)

```rust
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// A manifest entry declaring one bio-rs CLI command as an MCP-exposable tool.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ToolManifest {
    /// Unique tool identifier (kebab-case, stable across versions).
    pub name: String,
    /// Human-readable display title.
    pub title: String,
    /// Detailed description for the LLM to understand when to invoke this tool.
    pub description: String,
    /// JSON Schema 2020-12 object defining the tool's input parameters.
    pub input_schema: serde_json::Value,
    /// Optional JSON Schema 2020-12 object defining the tool's structured output.
    /// When present, the MCP server guarantees responses conform to this schema.
    pub output_schema: Option<serde_json::Value>,
    /// Reference to the bio-rs CLI command this tool maps to.
    pub cli_command: CliCommandRef,
    /// Whether this tool reads local filesystem paths (security-relevant).
    pub accesses_filesystem: bool,
    /// Whether this tool performs destructive operations (security-relevant).
    pub destructive: bool,
    /// bio-rs version range this manifest is compatible with.
    pub biors_version: String,
}

/// Identifies the backing CLI command and subcommand path.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CliCommandRef {
    pub binary: String,      // e.g., "biors"
    pub subcommand: Vec<String>, // e.g., ["tokenize"] or ["cache", "inspect"]
}
```

### 3.2 bio-rs Tools to Expose

| MCP Tool Name | CLI Mapping | Description | Filesystem | Destructive |
|---------------|-------------|-------------|------------|-------------|
| `tokenize` | `biors tokenize` | Tokenize protein FASTA into stable token IDs | Yes | No |
| `validate` | `biors seq validate` | Validate biological sequences (protein/DNA/RNA/auto) | Yes | No |
| `workflow` | `biors workflow` | End-to-end validate -> tokenize -> model-input with provenance | Yes | No |
| `pipeline` | `biors pipeline` | Config-driven or no-config preprocessing pipeline | Yes | No |
| `batch_validate` | `biors batch validate` | Validate multiple FASTA files/directories/globs | Yes | No |
| `dataset_inspect` | `biors dataset inspect` | Inspect dataset descriptors, SHA-256 provenance, sample mapping | Yes | No |
| `cache_inspect` | `biors cache inspect` | Report local artifact store inventory | Yes | No |
| `cache_clean` | `biors cache clean` | Clean local artifact store (requires `--yes` equivalent) | Yes | **Yes** |
| `package_inspect` | `biors package inspect` | Inspect package manifest metadata | Yes | No |
| `package_validate` | `biors package validate` | Validate package manifest and artifacts | Yes | No |
| `package_bridge` | `biors package bridge` | Plan runtime bridge for package execution | Yes | No |
| `package_compatibility` | `biors package compatibility` | Compare two package manifest schemas | Yes | No |
| `package_diff` | `biors package diff` | Canonical diff between two package manifests | Yes | No |
| `package_verify` | `biors package verify` | Verify package outputs against observations | Yes | No |
| `package_migrate` | `biors package migrate` | Migrate package manifest between schema versions | Yes | No |
| `doctor` | `biors doctor` | Platform readiness diagnostics | No | No |

**Security note**: `cache_clean` is the only destructive tool. The MCP server will require explicit confirmation logic (or refuse the call) rather than silently executing destructive operations.

---

## 4. Stable JSON Output Schema for Tool Responses

### 4.1 Design Principles
- **Deterministic**: Same inputs produce byte-identical JSON (sorted keys, fixed precision, no timestamps in payload)
- **Verifiable**: Includes a `deterministic_hash` so agents can verify response consistency across invocations
- **Backward-compatible**: Extends the existing bio-rs success envelope rather than replacing it
- **MCP-compliant**: Wraps bio-rs output into MCP `content` + `structuredContent` format

### 4.2 `BiorsToolResponse` Schema

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "https://bio-rs.dev/schemas/mcp-tool-response.v0.json",
  "title": "bio-rs MCP Tool Response",
  "type": "object",
  "required": ["tool_name", "invocation_id", "status", "output", "timing", "deterministic_hash"],
  "properties": {
    "tool_name": {
      "type": "string",
      "description": "The MCP tool name that was invoked."
    },
    "invocation_id": {
      "type": "string",
      "format": "uuid",
      "description": "Unique identifier for this invocation, generated by the MCP server."
    },
    "status": {
      "type": "string",
      "enum": ["success", "error"],
      "description": "High-level execution status."
    },
    "output": {
      "type": "object",
      "description": "The bio-rs CLI success envelope or error envelope.",
      "required": ["ok"],
      "properties": {
        "ok": { "type": "boolean" },
        "biors_version": { "type": "string" },
        "input_hash": {
          "type": ["string", "null"],
          "pattern": "^fnv1a64:[0-9a-f]{16}$"
        },
        "data": {},
        "error": {
          "type": ["object", "null"],
          "required": ["code", "message", "location"],
          "properties": {
            "code": { "type": "string" },
            "message": { "type": "string" },
            "location": {
              "oneOf": [
                { "type": "string" },
                {
                  "type": "object",
                  "required": ["line", "record_index"],
                  "properties": {
                    "line": { "type": ["integer", "null"], "minimum": 1 },
                    "record_index": { "type": ["integer", "null"], "minimum": 0 }
                  }
                },
                { "type": "null" }
              ]
            }
          }
        }
      }
    },
    "timing": {
      "type": "object",
      "required": ["started_at", "elapsed_ms"],
      "properties": {
        "started_at": {
          "type": "string",
          "format": "date-time",
          "description": "ISO 8601 timestamp when execution began."
        },
        "elapsed_ms": {
          "type": "integer",
          "minimum": 0,
          "description": "Wall-clock milliseconds spent executing the tool."
        }
      }
    },
    "deterministic_hash": {
      "type": "string",
      "pattern": "^sha256:[0-9a-f]{64}$",
      "description": "SHA-256 of the canonical JSON serialization of this response object EXCLUDING this field and the timing.started_at field."
    }
  },
  "additionalProperties": false
}
```

### 4.3 Deterministic Hash Computation

```rust
use serde_json::{json, Value};
use sha2::{Sha256, Digest};

fn compute_deterministic_hash(response: &BiorsToolResponse) -> String {
    let mut canonical = json!(response);
    canonical.as_object_mut().unwrap().remove("deterministic_hash");
    if let Some(timing) = canonical.get_mut("timing") {
        timing.as_object_mut().unwrap().remove("started_at");
    }
    let canonical_bytes = canonicalize_json(&canonical);
    let hash = Sha256::digest(&canonical_bytes);
    format!("sha256:{:x}", hash)
}

fn canonicalize_json(value: &Value) -> Vec<u8> {
    // Implementation uses serde_json::to_vec with a custom formatter
    // that sorts object keys lexicographically.
}
```

---

## 5. MCP Server Architecture

### 5.1 Crate Structure

```
packages/rust/biors-mcp-server/
├── Cargo.toml
├── src/
│   ├── main.rs              # Entry point: stdio transport, serve loop
│   ├── server.rs             # MCP ServerHandler implementation
│   ├── tools/
│   │   ├── mod.rs            # Tool registry and dispatch
│   │   ├── tokenize.rs       # tokenize tool handler
│   │   ├── validate.rs       # validate tool handler
│   │   ├── workflow.rs       # workflow tool handler
│   │   ├── pipeline.rs       # pipeline tool handler
│   │   ├── batch.rs          # batch_validate tool handler
│   │   ├── dataset.rs        # dataset_inspect tool handler
│   │   ├── cache.rs          # cache_inspect / cache_clean handlers
│   │   ├── package.rs        # package_* tool handlers
│   │   └── doctor.rs         # doctor tool handler
│   ├── manifest.rs           # ToolManifest definitions and JSON Schema generation
│   ├── response.rs           # BiorsToolResponse builder + deterministic hash
│   ├── error.rs              # MCP error code mapping
│   └── cli_adapter.rs        # Thin adapter layer to existing biors handlers
├── schemas/
│   └── mcp-tool-response.v0.json
└── tests/
    ├── integration_tests.rs  # End-to-end stdio transport tests
    └── deterministic_hash.rs # Hash consistency tests
```

### 5.2 Dependencies

```toml
[package]
name = "biors-mcp-server"
version = "0.46.0"
edition = "2021"

[dependencies]
rmcp = { version = "0.16", features = ["server", "transport-io", "schemars"] }
biors-core = { workspace = true }
biors = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }
schemars = "0.8"
uuid = { version = "1.0", features = ["v4", "serde"] }
sha2 = "0.10"
hex = "0.4"
tokio = { version = "1.0", features = ["rt-multi-thread", "macros", "io-std", "io-util", "time"] }
thiserror = "2.0"
```

### 5.3 Transport: stdio

```rust
use rmcp::{ServiceExt, transport::io::ServerIoTransport};
use tokio::io::{stdin, stdout};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let transport = ServerIoTransport::new(stdin(), stdout());
    let server = BiorsMcpServer::new().serve(transport).await?;
    server.wait().await?;
    Ok(())
}
```

### 5.4 ServerHandler Implementation

```rust
use rmcp::handler::server::ServerHandler;
use rmcp::model::*;

pub struct BiorsMcpServer {
    tool_registry: ToolRegistry,
}

#[rmcp::tool_router(server_handler)]
impl BiorsMcpServer {
    #[tool]
    async fn tokenize(&self, params: TokenizeParams) -> ToolResult<BiorsToolResponse> {
        self.dispatch("tokenize", params).await
    }
    // ... additional #[tool] methods
}

impl ServerHandler for BiorsMcpServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            name: "bio-rs-mcp-server".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            capabilities: ServerCapabilities {
                tools: Some(ToolCapabilities { list_changed: false }),
                ..Default::default()
            },
            ..Default::default()
        }
    }
}
```

### 5.5 Error Mapping to MCP Error Codes

| bio-rs Error | MCP Protocol Error Code | MCP Error Message |
|--------------|------------------------|-------------------|
| Unknown tool name | `-32601` | `Method not found` |
| Invalid tool arguments | `-32602` | `Invalid params` |
| Missing required field | `-32602` | `Invalid params: missing required field '...'` |
| Internal server panic | `-32603` | `Internal error` |
| Tool execution timeout | `-32001` | `Request timeout` |
| bio-rs CLI returns error envelope | *Not a protocol error* -- returned as `isError: true` in `tools/call` result |

---

## 6. Mapping to Existing biors CLI Commands

### 6.1 Design Principle: Adapter Layer, Not Duplication

The MCP server does **not** reimplement bio-rs logic. A thin `cli_adapter` module maps MCP tool arguments to existing `biors` CLI handler functions.

**Refactoring needed in `biors` crate** (minimal, non-breaking):
- Extract the "compute" logic from `handlers.rs` into public `compute_*` functions that return the data structs directly, without calling `print_success`.
- Keep the existing `run()` function as the CLI entry point that calls `compute_*` + `print_success`.

### 6.2 Example: Tokenize Tool Flow

```
Agent (Claude Desktop)
  -> MCP Client
    -> JSON-RPC: tools/call { name: "tokenize", arguments: { path: "/data/protein.fasta", profile: "protein-20" } }
      -> biors-mcp-server stdin
        -> ServerHandler::tokenize()
          -> CliAdapter::run_tokenize()
            -> biors::compute_tokenize(path, profile)  // reuses core logic
              -> returns Vec<TokenizedRecord>
          -> BiorsToolResponse::success("tokenize", tokenized_records)
            -> deterministic_hash computed
          -> rmcp serializes to MCP result { content: [...], structuredContent: {...} }
        -> stdout JSON-RPC response
      -> MCP Client parses result
    -> Agent receives structured content
```

---

## 7. Security Design

### 7.1 Threat Model
The MCP server runs as a local subprocess with the same privileges as the host agent.

### 7.2 Mitigations

| Risk | Mitigation |
|------|------------|
| Path traversal | All path arguments are canonicalized and validated against an **allowlist** of permitted directories (configurable via `BIORS_MCP_ALLOWLIST` env var). |
| Arbitrary command execution | The server only invokes **predefined, hardcoded tool mappings**. There is no generic "run shell command" tool. |
| Destructive operations | `cache_clean` requires an explicit `confirm: true` argument. Without it, the tool returns `isError: true`. |
| Resource exhaustion | All FASTA-backed tools enforce a **configurable max file size** (default 1GB) and **max record count** (default 10 million). |
| Stdio isolation | The server writes ONLY JSON-RPC messages to stdout. All logging goes to stderr. |

---

## 8. Implementation Plan

### Phase 1: Foundation
1. Create `biors-mcp-server` crate in `packages/rust/biors-mcp-server/`
2. Add to workspace `Cargo.toml`
3. Set up `rmcp` dependency
4. Implement stdio transport bootstrap
5. Define `ToolManifest` and response structs
6. Implement deterministic hash with tests

### Phase 2: Adapter Refactoring
1. Refactor `biors` crate handlers to expose `compute_*` functions
2. Implement `cli_adapter.rs` in `biors-mcp-server`

### Phase 3: Tool Implementation
1. Implement `#[tool]` handlers for all 16 tools
2. Generate JSON Schemas for each tool's `inputSchema` and `outputSchema`

### Phase 4: Integration & Testing
1. Integration tests using `rmcp` client transport
2. Deterministic hash tests
3. Security tests (path traversal, destructive ops)
4. Error mapping tests

### Phase 5: Documentation & Release
1. Write `docs/mcp-server.md`
2. Update `README.md`
3. Add CI job
4. Version bump to `0.46.0`

---

## 9. Summary of Changes to Existing Crates

| Crate | Change | Breaking? |
|-------|--------|-----------|
| `biors` | Refactor `handlers.rs` to expose `compute_*` functions alongside existing `run_*` wrappers | No |
| `biors` | Make `output::write_success_to` public (or add `serialize_success`) | No |
| `biors-core` | None | No |
| Workspace `Cargo.toml` | Add `biors-mcp-server` to `members` | No |

use biors_core::error::Diagnostic;
use biors_core::sequence::{SequenceKind, SequenceValidationIssue};
use std::fs;

mod common;

#[test]
fn python_api_docs_do_not_use_runtime_bridge_ready_alone() {
    let repo = common::repo_root();
    let python_api = fs::read_to_string(repo.join("docs/python-api.md")).expect("read Python API");

    assert!(
        python_api.contains("bridge[\"contract_ready\"]")
            && python_api.contains("bridge[\"artifact_checked\"]")
            && python_api.contains("bridge[\"execution_ready\"]"),
        "Python API runtime bridge example must inspect contract and execution readiness fields"
    );
    assert!(
        !python_api.contains("print(bridge[\"ready\"])"),
        "Python API runtime bridge example must not use ready alone as execution readiness"
    );
    assert!(
        !python_api.contains("bridge[\"compatible\"]"),
        "Python API runtime bridge example must not document a non-existent compatible field"
    );
}

#[test]
fn reliability_docs_match_json_parse_error_contract() {
    let repo = common::repo_root();
    let cli_contract =
        fs::read_to_string(repo.join("docs/cli-contract.md")).expect("read CLI contract");

    assert!(
        cli_contract.contains("--json") && cli_contract.contains("cli.invalid_arguments"),
        "CLI contract must document JSON-mode CLI argument parse failures"
    );

    assert!(
        cli_contract.contains("location: null"),
        "CLI contract must document parse-error envelopes without source locations"
    );
}

#[test]
fn public_docs_preserve_local_no_secret_no_upload_defaults() {
    let repo = common::repo_root();
    let public_docs = [
        ("README", "README.md"),
        ("quickstart", "docs/quickstart.md"),
        ("researcher workflows", "docs/researcher-workflows.md"),
        ("MCP README", "crates/biors-mcp-server/README.md"),
        ("service interface", "docs/service-interface.md"),
        ("WASM API", "docs/wasm-api.md"),
    ];

    let mut combined = String::new();
    for (name, path) in public_docs {
        let contents =
            fs::read_to_string(repo.join(path)).unwrap_or_else(|_| panic!("read {name}"));
        let normalized = normalize_whitespace(&contents);
        assert!(
            contains_any(
                &normalized,
                &[
                    "no upload",
                    "not upload",
                    "does not upload",
                    "without uploading",
                    "no input uploads",
                    "no biological data upload",
                ],
            ),
            "{name} must state that promoted workflows do not upload biological data"
        );
        combined.push_str(&normalized);
        combined.push('\n');
    }

    assert_contains_all(
        &combined,
        &[
            "No API keys, tokens, secrets, credentials, or network access are required",
            "no telemetry",
            "no external model calls",
        ],
        "public docs must preserve local-only defaults:",
    );

    for forbidden in [
        "set OPENAI_API_KEY",
        "set ANTHROPIC_API_KEY",
        "requires telemetry",
        "uploads biological data",
        "calls cloud models by default",
        "persistent hosted workspace",
    ] {
        assert!(
            !combined.contains(forbidden),
            "public docs must not contain promoted remote/default claim: {forbidden}"
        );
    }
}

#[test]
fn public_docs_use_researcher_agent_tool_layer_framing() {
    let repo = common::repo_root();
    let framed_docs = [
        ("README", "README.md"),
        ("quickstart", "docs/quickstart.md"),
        ("CLI contract", "docs/cli-contract.md"),
        ("package format", "docs/package-format.md"),
        ("service interface", "docs/service-interface.md"),
        ("Python API", "docs/python-api.md"),
        ("WASM API", "docs/wasm-api.md"),
        ("Candle backend", "docs/candle-backend.md"),
        ("MCP README", "crates/biors-mcp-server/README.md"),
    ];

    let mut combined = String::new();
    for (name, path) in framed_docs {
        let contents =
            fs::read_to_string(repo.join(path)).unwrap_or_else(|_| panic!("read {name}"));
        let normalized = normalize_whitespace(&contents);
        assert!(
            contains_any(
                &normalized,
                &[
                    "AI-ready biological data I/O, validation, and tokenization engine",
                    "agent-callable",
                    "researcher-callable",
                ],
            ),
            "{name} must use the researcher/agent engine framing"
        );
        combined.push_str(&normalized);
        combined.push('\n');
    }

    assert_contains_all(
        &combined,
        &[
            "researchers",
            "research agents",
            "CLI",
            "MCP",
            "model-ready",
            "package",
            "reproducible JSON",
        ],
        "public docs missing AI-ready product framing:",
    );

    for forbidden in [
        "bio-rs is an autonomous research agent",
        "provides autonomous research planning",
        "hosted workspace by default",
        "browser model execution",
        "cloud model execution",
        "full DNA/RNA package conversion",
        "pretrained inference service",
    ] {
        assert!(
            !combined.contains(forbidden),
            "public docs must not overclaim 1.0 scope: {forbidden}"
        );
    }
}

#[test]
fn sequence_issue_codes_match_docs_schemas_wasm_and_diagnostic_contracts() {
    let repo = common::repo_root();
    let expected_codes = ["ambiguous_symbol", "invalid_symbol"];
    let issue = SequenceValidationIssue::invalid('U', 5, SequenceKind::Dna);
    let payload = serde_json::to_value(&issue).expect("serialize sequence issue");

    assert_eq!(issue.code(), "invalid_symbol");
    assert_eq!(payload["code"], "invalid_symbol");

    let surfaces = [
        (
            "FASTA validation schema",
            fs::read_to_string(repo.join("schemas/fasta-validation-output.v0.json"))
                .expect("read FASTA validation schema"),
        ),
        (
            "WASM TypeScript declarations",
            fs::read_to_string(repo.join("crates/biors-wasm/src/types.rs"))
                .expect("read WASM TypeScript declarations"),
        ),
        (
            "CLI contract docs",
            fs::read_to_string(repo.join("docs/cli-contract.md")).expect("read CLI contract"),
        ),
        (
            "error code registry",
            fs::read_to_string(repo.join("docs/error-codes.md")).expect("read error codes"),
        ),
        (
            "Rust API docs",
            fs::read_to_string(repo.join("docs/rust-api.md")).expect("read Rust API docs"),
        ),
    ];

    for (name, contents) in surfaces {
        for code in expected_codes {
            assert!(contents.contains(code), "{name} must document {code}");
        }
        assert!(
            !contents.contains("sequence.invalid_symbol")
                && !contents.contains("sequence.ambiguous_symbol"),
            "{name} must not expose legacy namespaced sequence issue codes"
        );
    }
}

fn assert_contains_all(contents: &str, required: &[&str], context: &str) {
    for value in required {
        assert!(contents.contains(value), "{context} {value}");
    }
}

fn contains_any(contents: &str, candidates: &[&str]) -> bool {
    candidates
        .iter()
        .any(|candidate| contents.contains(candidate))
}

fn normalize_whitespace(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

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

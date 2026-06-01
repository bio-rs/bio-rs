use biors_core::error::Diagnostic;
use biors_core::sequence::{SequenceKind, SequenceValidationIssue};
use biors_core::service;
use serde_json::Value;
use std::fs;
use std::path::Path;

mod common;

#[test]
fn release_readiness_documentation_surfaces_are_present_and_linked() {
    let repo = common::repo_root();
    let required = [
        "CITATION.cff",
        "docs/quickstart.md",
        "docs/demo.md",
        "docs/install.md",
        "docs/cli-contract.md",
        "docs/backend-architecture.md",
        "docs/candle-backend.md",
        "docs/error-codes.md",
        "docs/reliability.md",
        "docs/python-interop.md",
        "docs/wasm-readiness.md",
        "docs/public-contract-1.0-candidates.md",
        "docs/versioning.md",
        "docs/final-release-checklist.md",
    ];

    for path in required {
        assert!(
            repo.join(path).exists(),
            "missing release-readiness doc: {path}"
        );
    }

    let readme = fs::read_to_string(repo.join("README.md")).expect("read README");
    for link in [
        "docs/quickstart.md",
        "docs/demo.md",
        "docs/install.md",
        "docs/cli-contract.md",
        "docs/backend-architecture.md",
        "docs/candle-backend.md",
        "docs/error-codes.md",
        "docs/reliability.md",
        "docs/python-interop.md",
        "docs/wasm-readiness.md",
        "docs/public-contract-1.0-candidates.md",
        "docs/versioning.md",
        "docs/final-release-checklist.md",
        "CITATION.cff",
    ] {
        assert!(readme.contains(link), "README does not link {link}");
    }

    let quickstart = fs::read_to_string(repo.join("docs/quickstart.md")).expect("read quickstart");
    let demo = fs::read_to_string(repo.join("docs/demo.md")).expect("read demo");
    let cli_contract =
        fs::read_to_string(repo.join("docs/cli-contract.md")).expect("read CLI contract");
    for (name, contents) in [
        ("quickstart", quickstart.as_str()),
        ("demo", demo.as_str()),
        ("CLI contract", cli_contract.as_str()),
    ] {
        assert!(
            contents.contains("biors --version"),
            "{name} does not document version verification"
        );
    }

    assert!(
        readme.contains("## Quickstart"),
        "README does not expose quickstart copy"
    );
    assert!(
        quickstart.contains("First 60 Seconds"),
        "quickstart does not expose first-impression commands"
    );
}

#[test]
fn readme_schema_inventory_lists_all_schema_files() {
    let repo = common::repo_root();
    let readme = fs::read_to_string(repo.join("README.md")).expect("read README");
    let schemas_dir = repo.join("schemas");

    for entry in fs::read_dir(&schemas_dir).expect("read schemas directory") {
        let entry = entry.expect("read schema entry");
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("json") {
            continue;
        }

        let file_name = path
            .file_name()
            .and_then(|name| name.to_str())
            .expect("schema file name");
        assert!(
            readme.contains(file_name),
            "README schema inventory is missing schemas/{file_name}"
        );
    }
}

#[test]
fn citation_version_matches_workspace_package_version() {
    let repo = common::repo_root();
    let workspace_version = workspace_package_version(&repo);

    let citation = fs::read_to_string(repo.join("CITATION.cff")).expect("read citation metadata");
    let citation_version = citation
        .lines()
        .find_map(|line| line.strip_prefix("version: "))
        .map(|value| value.trim_matches('"'))
        .expect("citation version");

    assert_eq!(
        citation_version, workspace_version,
        "CITATION.cff version must match the workspace package version"
    );
}

#[test]
fn example_package_metadata_versions_match_workspace_package_version() {
    let repo = common::repo_root();
    let workspace_version = workspace_package_version(&repo);
    let manifest_path = repo.join("examples/protein-package/manifest.json");
    let manifest: Value = serde_json::from_str(
        &fs::read_to_string(&manifest_path).expect("read example package manifest"),
    )
    .expect("parse example package manifest");
    let citation = fs::read_to_string(repo.join("examples/protein-package/docs/CITATION.cff"))
        .expect("read example package citation");
    let package_format =
        fs::read_to_string(repo.join("docs/package-format.md")).expect("read package format doc");
    let checklist = fs::read_to_string(repo.join("docs/final-release-checklist.md"))
        .expect("read final release checklist");

    assert_eq!(
        manifest["metadata"]["citation"]["preferred_citation"],
        format!("bio-rs protein package fixture, version {workspace_version}")
    );
    assert!(
        citation.contains(&format!("version: \"{workspace_version}\"")),
        "example package citation version must match workspace package version"
    );

    for (name, contents) in [
        ("example package manifest", manifest.to_string()),
        ("example package citation", citation),
        ("package format doc", package_format),
    ] {
        assert!(
            !contents.contains("0.31.0"),
            "{name} still contains stale package fixture version text"
        );
    }
    assert!(
        checklist.contains("Example Metadata Version Audit"),
        "final release checklist must include an example metadata version audit"
    );
}

#[test]
fn stale_benchmark_artifact_is_labeled_historical_in_readme() {
    let repo = common::repo_root();
    let workspace_version = workspace_package_version(&repo);
    let readme = fs::read_to_string(repo.join("README.md")).expect("read README");
    let benchmark: Value = serde_json::from_str(
        &fs::read_to_string(repo.join("benchmarks/fasta_vs_biopython.json"))
            .expect("read benchmark artifact"),
    )
    .expect("parse benchmark artifact");

    let benchmark_version = benchmark["environment"]["biors_core"]
        .as_str()
        .expect("benchmark biors-core version");

    if benchmark_version != workspace_version {
        assert!(
            readme.contains("Historical FASTA benchmark reference"),
            "stale benchmark artifacts must be visibly labeled historical"
        );
        assert!(
            readme.contains("not current-version performance evidence"),
            "README must not present stale benchmark numbers as current release evidence"
        );
        assert!(
            !readme.contains(&format!(
                "The `{workspace_version}` patch keeps those numeric claims pinned"
            )),
            "README must not tie stale numeric benchmark claims to the current version"
        );
    }
}

#[test]
fn python_api_docs_use_runtime_bridge_ready_field() {
    let repo = common::repo_root();
    let python_api = fs::read_to_string(repo.join("docs/python-api.md")).expect("read Python API");

    assert!(
        python_api.contains("print(bridge[\"ready\"])"),
        "Python API runtime bridge example must use the schema-backed ready field"
    );
    assert!(
        !python_api.contains("bridge[\"compatible\"]"),
        "Python API runtime bridge example must not document a non-existent compatible field"
    );
}

#[test]
fn reliability_docs_match_json_parse_error_contract() {
    let repo = common::repo_root();
    let reliability =
        fs::read_to_string(repo.join("docs/reliability.md")).expect("read reliability docs");
    let cli_contract =
        fs::read_to_string(repo.join("docs/cli-contract.md")).expect("read CLI contract");

    for (name, contents) in [
        ("reliability docs", reliability.as_str()),
        ("CLI contract", cli_contract.as_str()),
    ] {
        assert!(
            contents.contains("--json") && contents.contains("cli.invalid_arguments"),
            "{name} must document JSON-mode CLI argument parse failures"
        );
    }

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
            fs::read_to_string(repo.join("packages/rust/biors-wasm/src/types.rs"))
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

#[test]
fn final_release_checklist_covers_required_gates() {
    let repo = common::repo_root();
    let checklist =
        fs::read_to_string(repo.join("docs/final-release-checklist.md")).expect("read checklist");
    let script =
        fs::read_to_string(repo.join("scripts/check-final-release.sh")).expect("read final script");

    for expected in [
        "Full End-To-End Workflow Validation",
        "Public Contract Freeze",
        "Dependency Policy",
        "Public Security Surface",
        "Breaking Change Cleanup",
        "Benchmark Artifact Coverage",
        "Release Artifact Contents",
        "Registry Preflight",
        "Version Tag",
        "Binary Release Test",
        "Install Flow Final Test",
        "GitHub Release Dry Run",
        "Public Demo Dry Run",
        "Final Release Checklist",
    ] {
        assert!(
            checklist.contains(expected),
            "final checklist missing {expected}"
        );
    }

    for expected in [
        "docs/pre-release-audit-main-2026-05-30.md",
        "scripts/check-dependency-policy.py",
        "Cargo.lock",
        "scripts/check-benchmark-docs.sh",
        "scripts/check-release-artifact-contents.py",
        "scripts/check-registry-versions.py",
        "LICENSE-APACHE",
        "LICENSE-MIT",
        ".github/workflows/benchmarks.yml",
        "cargo test --workspace --benches --all-features",
        "SECURITY.md",
        "WASM/npm package",
        "MCP tools",
        "external-process backend contracts",
        "optional Candle model artifact loading",
        "external services by",
    ] {
        assert!(
            checklist.contains(expected),
            "final checklist missing release gate detail {expected}"
        );
    }

    assert!(
        !checklist.contains("No known breaking cleanup is deferred"),
        "final checklist must not claim there is no deferred cleanup while an audit queue exists"
    );

    for expected in [
        "scripts/check.sh",
        "scripts/check-security-audit.sh",
        "cargo build --locked --release -p biors",
        "BIORS_BIN=target/release/biors sh scripts/launch-demo.sh",
        "scripts/check-install-smoke.sh",
        "scripts/check-package-artifacts.sh",
        "python3 scripts/check-release-workflow.py",
    ] {
        assert!(
            script.contains(expected),
            "final release script missing {expected}"
        );
    }
}

#[test]
fn security_policy_covers_promoted_public_surfaces() {
    let repo = common::repo_root();
    let security = fs::read_to_string(repo.join("SECURITY.md")).expect("read security policy");

    for expected in [
        "biors-core",
        "biors-backend-candle",
        "biors-mcp-server",
        "biors-python",
        "biors-wasm",
        "package conversion",
        "cache cleanup",
        "MCP tool inputs",
        "WASM/npm package APIs",
        "external-process backend contracts",
        "Candle model artifact loading",
        "local filesystem safety",
        "should not upload biological data",
    ] {
        assert!(
            security.contains(expected),
            "SECURITY.md missing promoted security surface detail: {expected}"
        );
    }
}

#[test]
fn benchmark_workflow_runs_smoke_and_scheduled_criterion_suite() {
    let repo = common::repo_root();
    let workflow = fs::read_to_string(repo.join(".github/workflows/benchmarks.yml"))
        .expect("read benchmark workflow");

    for expected in [
        "pull_request:",
        "workflow_dispatch:",
        "schedule:",
        "permissions:",
        "contents: read",
        "scripts/check-benchmark-docs.sh",
        "cargo test --workspace --benches --all-features",
        "if: github.event_name == 'workflow_dispatch' || github.event_name == 'schedule'",
        "cargo bench -p biors-core --bench fasta_workloads",
        "cargo bench -p biors-backend-candle --bench candle_linear_probe",
        "cargo bench -p biors-mcp-server --bench mcp_request_overhead",
    ] {
        assert!(
            workflow.contains(expected),
            "benchmark workflow missing {expected}"
        );
    }
}

#[test]
fn local_check_scripts_use_rendered_benchmark_docs_gate() {
    let repo = common::repo_root();
    let check = fs::read_to_string(repo.join("scripts/check.sh")).expect("read check.sh");
    let check_fast =
        fs::read_to_string(repo.join("scripts/check-fast.sh")).expect("read check-fast.sh");

    for (name, script) in [("check.sh", check), ("check-fast.sh", check_fast)] {
        assert!(
            script.contains("scripts/check-benchmark-docs.sh"),
            "{name} must run the rendered benchmark docs gate"
        );
        assert!(
            !script.contains("python3 scripts/check-benchmark-artifact.py"),
            "{name} must not use the artifact-only benchmark check as its top-level benchmark gate"
        );
    }
}

#[test]
fn github_templates_cover_promoted_release_surfaces() {
    let repo = common::repo_root();
    let pr_template = fs::read_to_string(repo.join(".github/pull_request_template.md"))
        .expect("read PR template");
    let benchmark_template =
        fs::read_to_string(repo.join(".github/ISSUE_TEMPLATE/benchmark_performance_idea.md"))
            .expect("read benchmark issue template");

    for expected in [
        "scripts/test-python-wheel.py",
        "wasm-pack test --node packages/rust/biors-wasm",
        "MCP integration tests",
        "Package artifact changes",
        "Schema parity",
        "Dependency/advisory/license audit",
        "Benchmark harness smoke",
    ] {
        assert!(
            pr_template.contains(expected),
            "PR template missing promoted surface check: {expected}"
        );
    }

    for expected in [
        "model-input construction",
        "workflow or pipeline orchestration",
        "dataset inspect",
        "package validation or verification",
        "Python binding",
        "WASM/JavaScript binding",
        "MCP or service contract",
        "optional Candle backend",
        "binding or request overhead",
        "Surface and non-claim boundaries",
    ] {
        assert!(
            benchmark_template.contains(expected),
            "benchmark issue template missing promoted surface: {expected}"
        );
    }
}

#[test]
fn contributing_docs_cover_promoted_surface_checks() {
    let repo = common::repo_root();
    let contributing = fs::read_to_string(repo.join("CONTRIBUTING.md")).expect("read contributing");

    for expected in [
        "Surface-specific checks",
        "Python bindings",
        "WASM/npm bindings",
        "MCP service",
        "Package/release artifacts",
        "Dependencies/security",
        "scripts/check-package-artifacts.sh",
        "scripts/check-security-audit.sh",
        "docs/final-release-checklist.md",
    ] {
        assert!(
            contributing.contains(expected),
            "CONTRIBUTING.md missing promoted surface guidance: {expected}"
        );
    }
}

#[test]
fn phase7_status_separates_implementation_from_release_readiness() {
    let repo = common::repo_root();
    let status = fs::read_to_string(repo.join("docs/phase7-status.md")).expect("read status doc");

    for expected in [
        "release-readiness is tracked separately",
        "implementation status",
        "needs contract hardening",
        "does not claim every",
        "binding and integration surface is fully researcher-grade",
        "docs/pre-release-audit-main-2026-05-30.md",
        "Do not describe Python, WASM, or MCP as fully researcher-grade",
    ] {
        assert!(
            status.contains(expected),
            "phase7 status missing release-readiness caveat: {expected}"
        );
    }

    for overstated in [
        "Implemented and release-workflow published",
        "Implemented and npm-published",
    ] {
        assert!(
            !status.contains(overstated),
            "phase7 status still overstates binding readiness: {overstated}"
        );
    }
}

#[test]
fn public_contract_candidates_separate_stable_bindings_and_experimental_runtime() {
    let repo = common::repo_root();
    let candidates = fs::read_to_string(repo.join("docs/public-contract-1.0-candidates.md"))
        .expect("read public contract candidates");

    for expected in [
        "Stable-Candidate Core Contracts",
        "CLI And JSON Schemas",
        "Binding Contracts",
        "Experimental Runtime And Integration Contracts",
        "Python package: `packages/rust/biors-python`",
        "WASM/npm package: `packages/rust/biors-wasm`",
        "MCP server: `packages/rust/biors-mcp-server`",
        "Transport-agnostic service contract",
        "SERVICE_INTERFACE_SCHEMA_VERSION",
        "ServiceInterfaceDocument",
        "RuntimeServiceSeparation",
        "OpenApiDirection",
        "ServiceRoute",
        "current_service_interface_document",
        "service_interface_document",
        "service_routes",
        "schemas/service-interface-output.v0.json",
        "internal scanner modules and low-level byte parsing helpers",
    ] {
        assert!(
            candidates.contains(expected),
            "public contract candidates missing scoped section: {expected}"
        );
    }

    let service_document = service::current_service_interface_document();
    for route in service_document.routes {
        for schema in [&route.request_schema, &route.response_schema] {
            assert!(
                candidates.contains(&format!("schemas/{schema}")),
                "public contract candidates missing service route schema {schema}"
            );
        }
    }

    let stable_section = candidates
        .split("## Experimental Runtime And Integration Contracts")
        .next()
        .expect("stable candidate section");
    for experimental in [
        "ExternalProcessBackend",
        "ExternalProcessConfig",
        "CandleBackend",
    ] {
        assert!(
            !stable_section.contains(experimental),
            "experimental runtime API listed as stable candidate: {experimental}"
        );
    }
}

#[test]
fn service_interface_docs_list_request_response_examples() {
    let repo = common::repo_root();
    let service_doc =
        fs::read_to_string(repo.join("docs/service-interface.md")).expect("read service docs");

    for expected in [
        "Request And Response Schemas",
        "sequence.validate",
        "sequence.inspect",
        "sequence.tokenize",
        "model_input.build",
        "package.inspect",
        "package.validate",
        "package.bridge.plan",
        "package.compatibility.compare",
        "fasta-validation-output.v0.json",
        "model-input-output.v0.json",
        "package-compatibility-output.v0.json",
    ] {
        assert!(
            service_doc.contains(expected),
            "service interface docs missing request/response example detail: {expected}"
        );
    }
}

fn workspace_package_version(repo: &Path) -> String {
    let workspace_manifest =
        fs::read_to_string(repo.join("Cargo.toml")).expect("read workspace manifest");
    let manifest: toml::Table = workspace_manifest
        .parse()
        .expect("parse workspace manifest");
    manifest
        .get("workspace")
        .and_then(toml::Value::as_table)
        .and_then(|workspace| workspace.get("package"))
        .and_then(toml::Value::as_table)
        .and_then(|package| package.get("version"))
        .and_then(toml::Value::as_str)
        .expect("workspace package version")
        .to_string()
}

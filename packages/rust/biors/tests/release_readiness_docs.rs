use biors_core::error::Diagnostic;
use biors_core::sequence::{SequenceKind, SequenceValidationIssue};
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
        "docs/install.md",
        "docs/cli-contract.md",
        "docs/candle-backend.md",
        "docs/error-codes.md",
        "docs/package-conversion.md",
        "docs/package-format.md",
        "docs/pipeline-config.md",
        "docs/python-api.md",
        "docs/rust-api.md",
        "docs/sequence-kind-support.md",
        "docs/service-interface.md",
        "docs/versioning.md",
        "docs/wasm-api.md",
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
        "docs/install.md",
        "docs/cli-contract.md",
        "docs/candle-backend.md",
        "docs/error-codes.md",
        "docs/package-conversion.md",
        "docs/package-format.md",
        "docs/pipeline-config.md",
        "docs/python-api.md",
        "docs/rust-api.md",
        "docs/sequence-kind-support.md",
        "docs/service-interface.md",
        "docs/versioning.md",
        "docs/wasm-api.md",
        "CITATION.cff",
    ] {
        assert!(readme.contains(link), "README does not link {link}");
    }

    let quickstart = fs::read_to_string(repo.join("docs/quickstart.md")).expect("read quickstart");
    let cli_contract =
        fs::read_to_string(repo.join("docs/cli-contract.md")).expect("read CLI contract");
    for (name, contents) in [
        ("quickstart", quickstart.as_str()),
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
    let release_script =
        fs::read_to_string(repo.join("scripts/check-final-release.sh")).expect("read final script");

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
        release_script.contains("scripts/check-package-artifacts.sh"),
        "final release gate must verify package artifacts after metadata version changes"
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
    let script =
        fs::read_to_string(repo.join("scripts/check-final-release.sh")).expect("read final script");
    let full_check = fs::read_to_string(repo.join("scripts/check.sh")).expect("read full check");
    let package_artifacts = fs::read_to_string(repo.join("scripts/check-package-artifacts.sh"))
        .expect("read package artifact check");
    let workflow_check = fs::read_to_string(repo.join("scripts/check-release-workflow.py"))
        .expect("read release workflow check");
    let release_workflow = fs::read_to_string(repo.join(".github/workflows/release.yml"))
        .expect("read release workflow");
    let attributes = fs::read_to_string(repo.join(".gitattributes")).expect("read attributes");

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
    assert!(
        full_check.contains("scripts/check-benchmark-docs.sh"),
        "full release check missing scripts/check-benchmark-docs.sh"
    );
    assert!(
        package_artifacts.contains("scripts/check-release-artifact-contents.py"),
        "package artifact gate must verify artifact contents"
    );
    assert!(
        script.contains("scripts/check.sh")
            && full_check.contains("scripts/check-dependency-policy.py"),
        "final release gate must transitively run the dependency policy check"
    );

    for expected in ["id-token: write", "npm publish", "--provenance"] {
        assert!(
            release_workflow.contains(expected),
            "release workflow missing trusted publishing invariant {expected}"
        );
    }

    for expected in [
        "--provenance",
        "npm publish",
        "scripts/check-release-artifact-contents.py",
        "scripts/check-registry-versions.py",
        "scripts/print-release-tool-versions.sh",
        "NPM_TOKEN",
        "NODE_AUTH_TOKEN",
    ] {
        assert!(
            workflow_check.contains(expected),
            "release workflow check missing trusted publishing invariant {expected}"
        );
    }

    assert!(
        attributes.contains("examples/protein-package/** text eol=lf"),
        "checksum fixture files must keep stable LF line endings on Windows"
    );
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
        "cargo bench -p biors-core --bench package_validation",
        "cargo bench -p biors-core --bench workflow_workloads",
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
fn published_rust_crates_include_discovery_metadata() {
    let repo = common::repo_root();

    for manifest_path in [
        "packages/rust/biors/Cargo.toml",
        "packages/rust/biors-core/Cargo.toml",
        "packages/rust/biors-backend-candle/Cargo.toml",
        "packages/rust/biors-mcp-server/Cargo.toml",
    ] {
        let manifest = fs::read_to_string(repo.join(manifest_path))
            .unwrap_or_else(|_| panic!("read {manifest_path}"));
        let manifest: toml::Table = manifest
            .parse()
            .unwrap_or_else(|_| panic!("parse {manifest_path}"));
        let package = manifest
            .get("package")
            .and_then(toml::Value::as_table)
            .unwrap_or_else(|| panic!("{manifest_path} missing [package] table"));
        let name = package
            .get("name")
            .and_then(toml::Value::as_str)
            .unwrap_or(manifest_path);

        let readme = package
            .get("readme")
            .and_then(toml::Value::as_str)
            .unwrap_or_default();
        assert!(
            !readme.is_empty(),
            "{name} must declare a readme before crates.io publishing"
        );

        let keywords = package
            .get("keywords")
            .and_then(toml::Value::as_array)
            .unwrap_or_else(|| panic!("{name} must declare crates.io keywords"));
        assert!(
            !keywords.is_empty() && keywords.len() <= 5,
            "{name} must declare 1-5 crates.io keywords"
        );
        assert!(
            keywords.iter().all(|keyword| keyword
                .as_str()
                .is_some_and(|keyword| !keyword.is_empty() && keyword.len() <= 20)),
            "{name} keywords must be non-empty and within crates.io length limits"
        );

        let categories = package
            .get("categories")
            .and_then(toml::Value::as_array)
            .unwrap_or_else(|| panic!("{name} must declare crates.io categories"));
        assert!(
            !categories.is_empty() && categories.len() <= 5,
            "{name} must declare 1-5 crates.io categories"
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
        "fixed-length model-input construction",
        "no-padding model-input construction",
        "workflow or pipeline orchestration",
        "pipeline config execution",
        "dataset inspect",
        "package validation or verification",
        "package artifact validation",
        "package fixture verification",
        "Python binding",
        "WASM/JavaScript binding",
        "MCP or service contract",
        "optional Candle backend",
        "binding or request overhead",
        "binding round-trip overhead",
        "Benchmark purpose",
        "release claim",
        "regression guard",
        "smoke benchmark",
        "exploratory measurement",
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
        "scripts/check-final-release.sh",
    ] {
        assert!(
            contributing.contains(expected),
            "CONTRIBUTING.md missing promoted surface guidance: {expected}"
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

#[test]
fn sequence_kind_support_matrix_covers_promoted_surfaces() {
    let repo = common::repo_root();
    let matrix =
        fs::read_to_string(repo.join("docs/sequence-kind-support.md")).expect("read matrix");
    let readme = fs::read_to_string(repo.join("README.md")).expect("read README");
    let service_tokenize_schema =
        fs::read_to_string(repo.join("schemas/service-sequence-tokenize-request.v0.json"))
            .expect("read service tokenize schema");
    let service_model_input_schema =
        fs::read_to_string(repo.join("schemas/service-model-input-request.v0.json"))
            .expect("read service model-input schema");
    let pipeline_config_schema = fs::read_to_string(repo.join("schemas/pipeline-config.v0.json"))
        .expect("read pipeline config schema");
    let python_api = fs::read_to_string(repo.join("docs/python-api.md")).expect("read Python API");
    let python_stub =
        fs::read_to_string(repo.join("packages/rust/biors-python/python/biors/__init__.pyi"))
            .expect("read Python stub");

    assert!(
        readme.contains("docs/sequence-kind-support.md"),
        "README must link the sequence-kind support matrix before broad DNA/RNA claims"
    );

    for expected in [
        "CLI `fasta validate` / `seq validate`",
        "CLI `batch validate`",
        "CLI `tokenize`",
        "CLI `model-input`",
        "CLI `workflow`",
        "CLI `pipeline --config`",
        "Python bindings",
        "WASM / JavaScript bindings",
        "MCP server",
        "Service contract schemas",
        "Package manifest validation",
        "Package conversion from Python/HF projects",
        "Benchmarks",
        "project-conversion limitations",
        "validate_fasta_input_with_kind",
    ] {
        assert!(
            matrix.contains(expected),
            "support matrix missing promoted surface or limitation: {expected}"
        );
    }

    for profile in [
        "protein-20",
        "protein-20-special",
        "dna-iupac",
        "dna-iupac-special",
        "rna-iupac",
        "rna-iupac-special",
    ] {
        assert!(
            matrix.contains(profile),
            "support matrix missing tokenizer profile: {profile}"
        );
        assert!(
            service_tokenize_schema.contains(profile),
            "service tokenize schema missing tokenizer profile: {profile}"
        );
        assert!(
            service_model_input_schema.contains(profile),
            "service model-input schema missing tokenizer profile: {profile}"
        );
        assert!(
            pipeline_config_schema.contains(profile),
            "pipeline config schema missing tokenizer profile: {profile}"
        );
    }

    for kind in ["protein", "dna", "rna"] {
        assert!(
            pipeline_config_schema.contains(kind),
            "pipeline config schema missing sequence kind: {kind}"
        );
    }

    for surface in [python_api.as_str(), python_stub.as_str()] {
        assert!(
            surface.contains("validate_fasta_input_with_kind"),
            "Python kind-aware validation helper must be documented and typed"
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

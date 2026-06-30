use std::fs;
use std::path::Path;

mod common;

#[test]
fn final_release_checklist_covers_required_gates() {
    let repo = common::repo_root();
    let script = read_repo_file(&repo, "scripts/check-final-release.sh");
    let full_check = read_repo_file(&repo, "scripts/check.sh");
    let package_artifacts = read_repo_file(&repo, "scripts/check-package-artifacts.sh");
    let workflow_check = [
        "scripts/check-release-workflow.py",
        "scripts/release/workflow_jobs.py",
        "scripts/release/workflow_text_markers.py",
    ]
    .into_iter()
    .map(|path| read_repo_file(&repo, path))
    .collect::<Vec<_>>()
    .join("\n");
    let release_workflow = read_repo_file(&repo, ".github/workflows/release.yml");
    let attributes = read_repo_file(&repo, ".gitattributes");

    assert_contains_all(
        &script,
        &[
            "scripts/check.sh",
            "scripts/check-security-audit.sh",
            "cargo build --locked --release -p biors",
            "BIORS_BIN=target/release/biors sh scripts/launch-demo.sh",
            "scripts/check-install-smoke.sh",
            "scripts/check-package-artifacts.sh",
            "scripts/check-local-artifact-qa.sh --no-publish",
            "python3 scripts/check-release-workflow.py",
        ],
        "final release script missing",
    );
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

    assert_contains_all(
        &release_workflow,
        &["id-token: write", "npm publish", "--provenance"],
        "release workflow missing trusted publishing invariant",
    );

    assert_contains_all(
        &workflow_check,
        &[
            "--provenance",
            "npm publish",
            "scripts/check-release-artifact-contents.py",
            "scripts/check-registry-versions.py",
            "scripts/print-release-tool-versions.sh",
            "NPM_TOKEN",
            "NODE_AUTH_TOKEN",
        ],
        "release workflow check missing trusted publishing invariant",
    );

    assert!(
        attributes.contains("testdata/protein-package/** text eol=lf"),
        "checksum fixture files must keep stable LF line endings on Windows"
    );
}

#[test]
fn repository_layout_uses_current_public_surface_names() {
    let repo = common::repo_root();

    for expected in [
        "crates",
        "contracts",
        "deploy/service",
        "testdata/sequences",
        "testdata/model-input-contract",
        "testdata/pipeline",
        "testdata/protein-package",
    ] {
        assert!(
            repo.join(expected).exists(),
            "repository layout missing {expected}"
        );
    }

    for obsolete in ["packages/rust", "examples", "fixtures/support"] {
        assert!(
            !repo.join(obsolete).exists(),
            "obsolete repository surface still exists: {obsolete}"
        );
    }
}

#[test]
fn security_policy_covers_promoted_public_surfaces() {
    let repo = common::repo_root();
    let security = read_repo_file(&repo, "SECURITY.md");

    assert_contains_all(
        &security,
        &[
            "biors-core",
            "biors-backend-candle",
            "biors-mcp-server",
            "biors-python",
            "biors-wasm",
            "package conversion",
            "MCP tool inputs",
            "WASM/npm package APIs",
            "external-process backend contracts",
            "Candle model artifact loading",
            "local filesystem safety",
            "should not upload biological data",
        ],
        "SECURITY.md missing promoted security surface detail:",
    );
}

#[test]
fn benchmark_workflow_runs_smoke_and_scheduled_criterion_suite() {
    let repo = common::repo_root();
    let workflow = read_repo_file(&repo, ".github/workflows/benchmarks.yml");

    assert_contains_all(
        &workflow,
        &[
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
        ],
        "benchmark workflow missing",
    );
}

#[test]
fn published_rust_crates_include_discovery_metadata() {
    let repo = common::repo_root();

    for manifest_path in [
        "crates/biors/Cargo.toml",
        "crates/biors-core/Cargo.toml",
        "crates/biors-backend-candle/Cargo.toml",
        "crates/biors-mcp-server/Cargo.toml",
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
fn local_check_scripts_use_benchmark_artifact_gate() {
    let repo = common::repo_root();
    let check = read_repo_file(&repo, "scripts/check.sh");
    let check_fast = read_repo_file(&repo, "scripts/check-fast.sh");
    let benchmark_gate = read_repo_file(&repo, "scripts/check-benchmark-docs.sh");

    for (name, script) in [("check.sh", check), ("check-fast.sh", check_fast)] {
        assert!(
            script.contains("scripts/check-benchmark-docs.sh"),
            "{name} must run the benchmark artifact gate"
        );
    }

    for artifact_check in [
        "check-cli-benchmark-artifact.py",
        "check-python-benchmark-artifact.py",
        "check-wasm-benchmark-artifact.py",
        "check-backend-benchmark-artifact.py",
        "check-mcp-benchmark-artifact.py",
    ] {
        assert!(
            benchmark_gate.contains(artifact_check),
            "benchmark artifact gate must validate {artifact_check}"
        );
    }

    assert!(
        !benchmark_gate.contains("fasta_vs_biopython"),
        "benchmark artifact gate must not depend on removed historical FASTA benchmark artifacts"
    );
}

#[test]
fn local_check_scripts_keep_preview_candle_out_of_workspace_metadata_gates() {
    let repo = common::repo_root();
    let check = read_repo_file(&repo, "scripts/check.sh");
    let check_fast = read_repo_file(&repo, "scripts/check-fast.sh");

    for (name, script) in [("check.sh", &check), ("check-fast.sh", &check_fast)] {
        assert!(
            script.contains("--exclude biors-backend-candle"),
            "{name} must exclude the preview Candle backend from workspace metadata gates"
        );
        assert!(
            script.contains("CARGO_BUILD_JOBS:=1")
                && script.contains("CARGO_INCREMENTAL:=0")
                && script.contains("CARGO_PROFILE_DEV_DEBUG:=0"),
            "{name} must pin cargo gate settings that avoid local metadata hangs"
        );
    }

    assert!(
        check.contains(
            "CARGO_BUILD_JOBS=1 cargo test --locked -p biors-backend-candle --test candle_backend"
        ),
        "check.sh must keep a runtime smoke test for the preview Candle backend"
    );
}

fn read_repo_file(repo: &Path, path: &str) -> String {
    fs::read_to_string(repo.join(path)).unwrap_or_else(|_| panic!("read {path}"))
}

fn assert_contains_all(haystack: &str, expected: &[&str], failure_prefix: &str) {
    for expected in expected {
        assert!(haystack.contains(expected), "{failure_prefix} {expected}");
    }
}

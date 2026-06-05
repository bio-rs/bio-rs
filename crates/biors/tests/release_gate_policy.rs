use std::fs;

mod common;

#[test]
fn final_release_checklist_covers_required_gates() {
    let repo = common::repo_root();
    let script =
        fs::read_to_string(repo.join("scripts/check-final-release.sh")).expect("read final script");
    let full_check = fs::read_to_string(repo.join("scripts/check.sh")).expect("read full check");
    let package_artifacts = fs::read_to_string(repo.join("scripts/check-package-artifacts.sh"))
        .expect("read package artifact check");
    let workflow_check = [
        "scripts/check-release-workflow.py",
        "scripts/release_workflow_jobs.py",
        "scripts/release_workflow_text_markers.py",
    ]
    .into_iter()
    .map(|path| fs::read_to_string(repo.join(path)).expect("read release workflow check"))
    .collect::<Vec<_>>()
    .join("\n");
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
        "integrations/python",
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

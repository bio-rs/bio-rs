use std::fs;

mod common;

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

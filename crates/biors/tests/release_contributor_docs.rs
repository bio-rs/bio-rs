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
        "wasm-pack test --node crates/biors-wasm",
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
fn security_policy_is_ready_for_1_0_support_line() {
    let repo = common::repo_root();
    let security = fs::read_to_string(repo.join("SECURITY.md")).expect("read security policy");

    for expected in [
        "Until `1.0.0` is published",
        "After `1.0.0` is published",
        "latest published `1.x` release",
        "current `main` branch",
        "latest published `0.x` release",
        "transitional",
        "unless explicitly extended",
    ] {
        assert!(
            contains_normalized(&security, expected),
            "SECURITY.md missing 1.0 support policy: {expected}"
        );
    }
}

#[test]
fn versioning_docs_separate_crate_1_0_from_schema_lifecycle() {
    let repo = common::repo_root();
    let versioning =
        fs::read_to_string(repo.join("docs/versioning.md")).expect("read versioning policy");

    for expected in [
        "crate SemVer 1.0",
        "schema lifecycle",
        "JSON schema names do not need to be renamed for crate 1.0",
        "product workflow stability",
        "biors.package.v0",
        "biors.package.v1",
    ] {
        assert!(
            contains_normalized(&versioning, expected),
            "versioning policy missing 1.0 schema lifecycle distinction: {expected}"
        );
    }
}

#[test]
fn release_version_prep_script_is_not_patch_release_only() {
    let repo = common::repo_root();
    let script = fs::read_to_string(repo.join("scripts/prepare-release-version.py"))
        .expect("read release prep script");

    assert!(
        script.contains("Prepare a bio-rs release version"),
        "release prep script must describe general release version prep"
    );
    assert!(
        script.contains("for example 1.0.0"),
        "release prep help should include a 1.0.0 example"
    );
    assert!(
        !script.contains("patch release version"),
        "release prep script must not describe itself as patch-release-only"
    );
}

fn contains_normalized(haystack: &str, needle: &str) -> bool {
    let normalized_haystack = haystack.split_whitespace().collect::<Vec<_>>().join(" ");
    let normalized_needle = needle.split_whitespace().collect::<Vec<_>>().join(" ");
    normalized_haystack.contains(&normalized_needle)
}

use std::fs;
use std::path::Path;

mod common;

#[test]
fn local_artifact_qa_script_exposes_safe_no_publish_mode() {
    let repo = common::repo_root();
    let script = read_repo_file(&repo, "scripts/check-local-artifact-qa.sh");

    assert_contains_all(
        &script,
        &[
            "--no-publish",
            "check_release_binary_cli",
            "check_mcp_stdio",
            "check_python_wheel",
            "check_wasm_npm_package",
            "check_local_service",
            "check_package_workflow",
        ],
        "local artifact QA script missing:",
    );

    for forbidden in [
        "cargo publish",
        "npm publish",
        "twine upload",
        "gh release create",
    ] {
        assert!(
            !script.contains(forbidden),
            "local artifact QA script must not run {forbidden}"
        );
    }
}

fn read_repo_file(repo: &Path, path: &str) -> String {
    fs::read_to_string(repo.join(path)).unwrap_or_else(|_| panic!("read {path}"))
}

fn assert_contains_all(haystack: &str, expected: &[&str], failure_prefix: &str) {
    for expected in expected {
        assert!(haystack.contains(expected), "{failure_prefix} {expected}");
    }
}

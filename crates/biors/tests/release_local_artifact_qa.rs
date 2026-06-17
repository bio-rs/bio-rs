use std::fs;
use std::path::Path;

mod common;

#[test]
fn release_qa_doc_covers_all_local_tool_layer_surfaces() {
    let repo = common::repo_root();
    let doc = read_repo_file(&repo, "docs/release-qa.md");

    assert_contains_all(
        &doc,
        &[
            "## No-Publish Local QA",
            "Release binary CLI workflows",
            "MCP stdio tool smoke",
            "Python wheel install/import/package API smoke",
            "WASM/npm build/import smoke",
            "Local service release-binary smoke",
            "Package validate/verify/bridge smoke",
            "## Post-Publish Approval Gate",
            "explicit approval",
        ],
        "release QA doc missing:",
    );
}

#[test]
fn no_publish_release_qa_section_has_no_publish_upload_or_tag_commands() {
    let repo = common::repo_root();
    let doc = read_repo_file(&repo, "docs/release-qa.md");
    let no_publish = section_between(
        &doc,
        "## No-Publish Local QA",
        "## Post-Publish Approval Gate",
    );

    for forbidden in [
        "cargo publish",
        "npm publish",
        "twine upload",
        "maturin upload",
        "gh release create",
        "git tag",
        "git push --tags",
        "actions/upload-artifact",
    ] {
        assert!(
            !no_publish.contains(forbidden),
            "no-publish QA section must not contain {forbidden}"
        );
    }

    let post_publish = section_after(&doc, "## Post-Publish Approval Gate");
    assert_contains_all(
        post_publish,
        &["cargo publish", "npm publish", "gh release create"],
        "post-publish gate must isolate publish commands:",
    );
}

#[test]
fn local_artifact_qa_script_exposes_safe_no_publish_mode() {
    let repo = common::repo_root();
    let script = read_repo_file(&repo, "scripts/check-local-artifact-qa.sh");

    assert_contains_all(
        &script,
        &[
            "--no-publish",
            "--check-doc-safety",
            "check_doc_safety",
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

fn section_between<'a>(contents: &'a str, start: &str, end: &str) -> &'a str {
    let start_index = contents
        .find(start)
        .unwrap_or_else(|| panic!("missing {start}"));
    let end_index = contents[start_index..]
        .find(end)
        .map(|index| start_index + index)
        .unwrap_or_else(|| panic!("missing {end}"));
    &contents[start_index..end_index]
}

fn section_after<'a>(contents: &'a str, start: &str) -> &'a str {
    let start_index = contents
        .find(start)
        .unwrap_or_else(|| panic!("missing {start}"));
    &contents[start_index..]
}

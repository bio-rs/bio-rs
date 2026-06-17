use std::fs;

mod common;

#[test]
fn package_artifact_workflow_docs_name_required_safety_boundaries() {
    let repo = common::repo_root();
    let researcher_doc =
        fs::read_to_string(repo.join("docs/researcher-workflows.md")).expect("workflow doc");
    let package_doc = fs::read_to_string(repo.join("docs/package-format.md")).expect("package doc");
    let docs = format!("{researcher_doc}\n{package_doc}");

    for required in [
        "package validate",
        "package verify",
        "package bridge",
        "Absolute paths",
        "`..` traversal",
        "checksum mismatch",
        "unknown schema version",
        "fixture observation",
        "contract_ready",
        "artifact_checked",
        "execution_ready",
    ] {
        assert!(
            docs.contains(required),
            "package workflow docs missing {required}"
        );
    }
}

#[test]
fn package_artifact_safety_tests_cover_required_failures() {
    let repo = common::repo_root();
    let core_paths =
        fs::read_to_string(repo.join("crates/biors-core/tests/package_artifact_paths.rs"))
            .expect("core path tests");
    let core_checksums =
        fs::read_to_string(repo.join("crates/biors-core/tests/package_artifact_checksums.rs"))
            .expect("core checksum tests");
    let cli_manifest =
        fs::read_to_string(repo.join("crates/biors/tests/cli_package_validate_manifest.rs"))
            .expect("CLI manifest tests");
    let cli_schema =
        fs::read_to_string(repo.join("crates/biors/tests/cli_package_validate_schema.rs"))
            .expect("CLI schema tests");
    let cli_verify =
        fs::read_to_string(repo.join("crates/biors/tests/cli_package_verify_fixture_results.rs"))
            .expect("CLI fixture verification tests");
    let bridge_rejections = fs::read_to_string(
        repo.join("crates/biors-core/tests/package_runtime_bridge_rejections.rs"),
    )
    .expect("runtime bridge rejection tests");
    let cli_bridge = fs::read_to_string(repo.join("crates/biors/tests/cli_package_bridge.rs"))
        .expect("CLI bridge tests");
    let tests = format!(
        "{core_paths}\n{core_checksums}\n{cli_manifest}\n{cli_schema}\n{cli_verify}\n{bridge_rejections}\n{cli_bridge}"
    );

    for required in [
        "rejects_absolute_artifact_paths",
        "rejects_asset_paths_outside_package_root",
        "rejects_checksum_mismatch_against_real_artifact",
        "package_validate_rejects_unknown_manifest_schema_version",
        "package_verify_reports_content_mismatch_code",
        "runtime_bridge_blocks_external_process_manifest_backend",
        "contract_ready",
        "artifact_checked",
        "execution_ready",
    ] {
        assert!(
            tests.contains(required),
            "package safety tests missing {required}"
        );
    }
}

use biors_core::package::{validate_package_manifest_artifacts, PackageValidationIssueCode};
use std::fs;

mod common;

#[test]
fn rejects_directory_artifact_path_without_checksum() {
    let base = common::temp_package_dir("directory-artifact");
    fs::create_dir_all(base.join("models")).expect("create models dir");
    fs::create_dir_all(base.join("fixtures")).expect("create fixtures dir");
    fs::write(base.join("fixtures/tiny.fasta"), b">seq1\nACDEFG\n").expect("write input");
    fs::write(base.join("fixtures/tiny.output.json"), b"{\"ok\":true}\n").expect("write output");

    let mut manifest = common::valid_manifest();
    manifest.model.path = "models".to_string();

    let report = validate_package_manifest_artifacts(&manifest, &base);

    assert!(!report.valid);
    assert!(report.structured_issues.iter().any(|issue| {
        issue.code == PackageValidationIssueCode::AssetReadFailed
            && issue.field == "model"
            && issue.message.contains("asset path is not a file")
    }));
}

#[test]
fn rejects_missing_manifest_relative_artifact() {
    let mut manifest = common::example_manifest();
    manifest.vocab.as_mut().expect("vocab").path = "vocabs/missing.json".to_string();

    let report = validate_package_manifest_artifacts(&manifest, &common::example_base_dir());

    assert!(!report.valid);
    assert!(report
        .issues
        .iter()
        .any(|issue| issue.contains("failed to read asset 'vocabs/missing.json'")));
}

#[test]
fn rejects_asset_paths_outside_package_root() {
    let mut manifest = common::example_manifest();
    manifest.fixtures[0].input = "../outside.fasta".to_string();

    let report = validate_package_manifest_artifacts(&manifest, &common::example_base_dir());

    assert!(!report.valid);
    assert!(report
        .issues
        .iter()
        .any(|issue| issue.contains("must stay inside the package root")));
    assert_eq!(
        report.structured_issues[0].code,
        PackageValidationIssueCode::InvalidAssetPath
    );
}

#[cfg(unix)]
#[test]
fn rejects_symlinked_artifact_that_escapes_package_root() {
    use std::os::unix::fs::symlink;

    let base = package_with_fixture_files("symlink-artifact-escape");
    let outside = common::temp_package_dir("symlink-artifact-escape-outside");
    fs::write(outside.join("model.onnx"), b"external model").expect("write outside model");
    symlink(
        outside.join("model.onnx"),
        base.join("models/protein-seed.onnx"),
    )
    .expect("symlink outside model");

    let report = validate_package_manifest_artifacts(&common::valid_manifest(), &base);

    assert!(!report.valid);
    assert!(report.structured_issues.iter().any(|issue| {
        issue.code == PackageValidationIssueCode::InvalidAssetPath
            && issue.field == "model"
            && issue.message.contains("must stay inside the package root")
    }));
}

#[cfg(unix)]
#[test]
fn rejects_nested_symlinked_artifact_that_escapes_package_root() {
    use std::os::unix::fs::symlink;

    let base = package_with_fixture_files("nested-symlink-artifact-escape");
    fs::remove_dir_all(base.join("models")).expect("remove models dir");
    let outside = common::temp_package_dir("nested-symlink-artifact-escape-outside");
    fs::write(outside.join("protein-seed.onnx"), b"external model").expect("write outside model");
    symlink(&outside, base.join("models")).expect("symlink outside models dir");

    let report = validate_package_manifest_artifacts(&common::valid_manifest(), &base);

    assert!(!report.valid);
    assert!(report.structured_issues.iter().any(|issue| {
        issue.code == PackageValidationIssueCode::InvalidAssetPath
            && issue.field == "model"
            && issue.message.contains("must stay inside the package root")
    }));
}

#[cfg(unix)]
#[test]
fn accepts_symlinked_artifact_that_stays_inside_package_root() {
    use std::os::unix::fs::symlink;

    let base = package_with_fixture_files("internal-symlink-artifact");
    fs::create_dir_all(base.join("artifacts")).expect("create artifact dir");
    fs::write(base.join("artifacts/model.onnx"), b"internal model").expect("write internal model");
    symlink(
        base.join("artifacts/model.onnx"),
        base.join("models/protein-seed.onnx"),
    )
    .expect("symlink internal model");

    let report = validate_package_manifest_artifacts(&common::valid_manifest(), &base);

    assert!(report.valid, "{:?}", report.structured_issues);
}

#[cfg(unix)]
fn package_with_fixture_files(name: &str) -> std::path::PathBuf {
    let base = common::temp_package_dir(name);
    fs::create_dir_all(base.join("models")).expect("create models dir");
    fs::create_dir_all(base.join("fixtures")).expect("create fixtures dir");
    fs::write(base.join("fixtures/tiny.fasta"), b">seq1\nACDEFG\n").expect("write input");
    fs::write(base.join("fixtures/tiny.output.json"), b"{\"ok\":true}\n").expect("write output");
    base
}

use biors_core::hash::{sha256_bytes_digest, sha256_canonical_json_digest};
use biors_core::package::{validate_package_manifest_artifacts, PackageValidationIssueCode};
use std::fs;

mod common;

#[test]
fn rejects_v1_assets_outside_declared_package_layout() {
    let mut manifest = common::example_manifest();
    manifest.model.path = "artifacts/protein-seed.onnx".to_string();

    let report = validate_package_manifest_artifacts(&manifest, &common::example_base_dir());

    assert!(!report.valid);
    assert!(report.structured_issues.iter().any(|issue| {
        issue.code == PackageValidationIssueCode::LayoutMismatch
            && issue.field == "model.path"
            && issue.message.contains("models")
    }));
}

#[test]
fn rejects_pipeline_config_outside_declared_pipeline_layout() {
    let mut manifest = common::example_manifest();
    manifest.preprocessing[0]
        .config
        .as_mut()
        .expect("pipeline config")
        .path = "configs/protein.toml".to_string();

    let report = validate_package_manifest_artifacts(&manifest, &common::example_base_dir());

    assert!(!report.valid);
    assert!(report.structured_issues.iter().any(|issue| {
        issue.code == PackageValidationIssueCode::LayoutMismatch
            && issue.field == "preprocessing[0].config.path"
            && issue.message.contains("pipelines")
    }));
}

#[test]
fn rejects_pipeline_config_checksum_mismatch() {
    let mut manifest = common::example_manifest();
    manifest.preprocessing[0]
        .config
        .as_mut()
        .expect("pipeline config")
        .checksum =
        Some("sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb".to_string());

    let report = validate_package_manifest_artifacts(&manifest, &common::example_base_dir());

    assert!(!report.valid);
    assert!(report.structured_issues.iter().any(|issue| {
        issue.code == PackageValidationIssueCode::ChecksumMismatch
            && issue.field == "preprocessing[0].config.checksum"
    }));
}

#[test]
fn rejects_invalid_checksum_format() {
    let mut manifest = common::valid_manifest();
    manifest.model.checksum = Some("draft-model-checksum".to_string());

    let report = validate_package_manifest_artifacts(&manifest, std::path::Path::new("."));

    assert!(!report.valid);
    assert!(report
        .issues
        .iter()
        .any(|issue| issue.contains("model.checksum")));
    assert_eq!(
        report.structured_issues[0].code,
        PackageValidationIssueCode::InvalidChecksumFormat
    );
    assert_eq!(report.structured_issues[0].field, "model.checksum");
}

#[test]
fn rejects_uppercase_checksum_hex_to_match_schema() {
    let mut manifest = common::valid_manifest();
    manifest.model.checksum =
        Some("sha256:AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA".to_string());

    let report = validate_package_manifest_artifacts(&manifest, std::path::Path::new("."));

    assert!(!report.valid);
    assert!(report.structured_issues.iter().any(|issue| {
        issue.code == PackageValidationIssueCode::InvalidChecksumFormat
            && issue.field == "model.checksum"
    }));
}

#[test]
fn rejects_checksum_mismatch_against_real_artifact() {
    let mut manifest = common::example_manifest();
    manifest.model.checksum =
        Some("sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_string());

    let report = validate_package_manifest_artifacts(&manifest, &common::example_base_dir());

    assert!(!report.valid);
    assert!(report
        .issues
        .iter()
        .any(|issue| issue.contains("model.checksum mismatch")));
    assert!(report.structured_issues.iter().any(|issue| {
        issue.code == PackageValidationIssueCode::ChecksumMismatch
            && issue.field == "model.checksum"
            && issue.message.contains("computed")
    }));
}

#[test]
fn package_artifact_validation_uses_raw_file_sha256_for_json_artifacts() {
    let base = common::temp_package_dir("raw-json-checksum");
    fs::create_dir_all(base.join("models")).expect("create models dir");
    fs::create_dir_all(base.join("fixtures")).expect("create fixtures dir");
    fs::write(base.join("models/protein-seed.onnx"), b"{\n  \"a\": 1\n}\n").expect("write model");
    fs::write(base.join("fixtures/tiny.fasta"), b">seq1\nACDEFG\n").expect("write input");
    fs::write(base.join("fixtures/tiny.output.json"), b"{\"ok\":true}\n").expect("write output");

    let mut manifest = common::valid_manifest();
    let model_bytes = fs::read(base.join("models/protein-seed.onnx")).expect("read model");
    manifest.model.checksum = Some(sha256_bytes_digest(&model_bytes));

    let report = validate_package_manifest_artifacts(&manifest, &base);
    assert!(report.valid, "{:?}", report.structured_issues);

    manifest.model.checksum = Some(sha256_canonical_json_digest(&model_bytes));
    let report = validate_package_manifest_artifacts(&manifest, &base);
    assert!(!report.valid);
    assert!(report.structured_issues.iter().any(|issue| {
        issue.code == PackageValidationIssueCode::ChecksumMismatch
            && issue.field == "model.checksum"
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

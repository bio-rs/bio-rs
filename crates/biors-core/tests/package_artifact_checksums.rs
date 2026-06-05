use biors_core::hash::{sha256_bytes_digest, sha256_canonical_json_digest};
use biors_core::package::{validate_package_manifest_artifacts, PackageValidationIssueCode};
use std::fs;

mod common;

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
fn accepts_large_checksum_only_model_artifact() {
    let base = common::temp_package_dir("large-model-checksum");
    fs::create_dir_all(base.join("models")).expect("create models dir");
    fs::create_dir_all(base.join("fixtures")).expect("create fixtures dir");
    fs::write(base.join("fixtures/tiny.fasta"), b">seq1\nACDEFG\n").expect("write input");
    fs::write(base.join("fixtures/tiny.output.json"), b"{\"ok\":true}\n").expect("write output");

    let model_bytes = vec![b'M'; 2 * 1024 * 1024];
    fs::write(base.join("models/protein-seed.onnx"), &model_bytes).expect("write model");

    let mut manifest = common::valid_manifest();
    manifest.model.checksum = Some(sha256_bytes_digest(&model_bytes));

    let report = validate_package_manifest_artifacts(&manifest, &base);

    assert!(report.valid, "{:?}", report.structured_issues);
}

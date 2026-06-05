use biors_core::package::{
    validate_package_manifest_artifacts,
    validate_package_manifest_artifacts_with_pipeline_config_validator, PackageValidationIssueCode,
    ReferencedConfigError,
};

mod common;

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
fn reports_invalid_referenced_pipeline_config_content() {
    let report = validate_package_manifest_artifacts_with_pipeline_config_validator(
        &common::example_manifest(),
        &common::example_base_dir(),
        Some(&|_| {
            Err(ReferencedConfigError::new(
                "pipeline.invalid_config",
                "export.max_length must be greater than zero",
                Some("export.max_length".to_string()),
            ))
        }),
    );

    assert!(!report.valid);
    assert!(report.structured_issues.iter().any(|issue| {
        issue.code == PackageValidationIssueCode::InvalidPipelineConfig
            && issue.field == "preprocessing[0].config"
            && issue.message.contains("pipelines/protein.toml")
            && issue.message.contains("export.max_length")
    }));
}

#[test]
fn skips_referenced_pipeline_config_content_when_checksum_mismatches() {
    let mut manifest = common::example_manifest();
    manifest.preprocessing[0]
        .config
        .as_mut()
        .expect("pipeline config")
        .checksum =
        Some("sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb".to_string());

    let report = validate_package_manifest_artifacts_with_pipeline_config_validator(
        &manifest,
        &common::example_base_dir(),
        Some(&|_| {
            Err(ReferencedConfigError::new(
                "pipeline.invalid_config",
                "should not run",
                None,
            ))
        }),
    );

    assert!(!report.valid);
    assert!(report.structured_issues.iter().any(|issue| {
        issue.code == PackageValidationIssueCode::ChecksumMismatch
            && issue.field == "preprocessing[0].config.checksum"
    }));
    assert!(!report
        .structured_issues
        .iter()
        .any(|issue| issue.code == PackageValidationIssueCode::InvalidPipelineConfig));
}

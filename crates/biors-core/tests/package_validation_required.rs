use biors_core::package::{
    validate_package_manifest, ModelArtifactMetadata, PackageValidationIssueCode,
};

mod common;

#[test]
fn validate_package_manifest_accepts_minimal_valid_manifest() {
    let manifest = common::valid_manifest();
    let report = validate_package_manifest(&manifest);
    assert!(report.valid);
    assert!(report.issues.is_empty());
    assert!(report.structured_issues.is_empty());
}

#[test]
fn validate_package_manifest_rejects_empty_model_metadata_name() {
    let mut manifest = common::valid_manifest();
    manifest.model.metadata = Some(ModelArtifactMetadata {
        name: " ".into(),
        version: None,
        architecture: None,
        task: None,
        source: None,
        description: None,
    });

    let report = validate_package_manifest(&manifest);

    assert!(!report.valid);
    assert!(report.structured_issues.iter().any(|issue| {
        issue.code == PackageValidationIssueCode::RequiredField
            && issue.field == "model.metadata.name"
    }));
}

#[test]
fn validate_package_manifest_rejects_missing_name() {
    let mut manifest = common::valid_manifest();
    manifest.name = "".into();
    let report = validate_package_manifest(&manifest);
    assert!(!report.valid);
    assert_eq!(report.structured_issues.len(), 1);
    assert_eq!(
        report.structured_issues[0].code,
        PackageValidationIssueCode::RequiredField
    );
    assert_eq!(report.structured_issues[0].field, "name");
}

#[test]
fn validate_package_manifest_rejects_missing_model_path() {
    let mut manifest = common::valid_manifest();
    manifest.model.path = "   ".into();
    let report = validate_package_manifest(&manifest);
    assert!(!report.valid);
    let model_issue = report
        .structured_issues
        .iter()
        .find(|i| i.field == "model.path")
        .expect("model.path issue expected");
    assert_eq!(model_issue.code, PackageValidationIssueCode::RequiredField);
}

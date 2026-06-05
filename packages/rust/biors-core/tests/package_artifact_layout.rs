use biors_core::package::{validate_package_manifest_artifacts, PackageValidationIssueCode};

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

use biors_core::package::{
    inspect_package_manifest, validate_package_manifest, validate_package_manifest_artifacts,
    ModelFormat, PackageManifest, PackageValidationIssueCode, RuntimeBackend,
    RuntimeTargetPlatform, SchemaVersion,
};
use std::path::Path;

mod common;

#[test]
fn inspects_portable_package_manifest() {
    let manifest = common::valid_manifest();
    let summary = inspect_package_manifest(&manifest);

    assert_eq!(summary.schema_version, SchemaVersion::BiorsPackageV0);
    assert_eq!(summary.name, "protein-seed");
    assert_eq!(summary.model_format, ModelFormat::Onnx);
    assert_eq!(summary.runtime_backend, RuntimeBackend::OnnxWebgpu);
    assert_eq!(
        summary.runtime_target,
        RuntimeTargetPlatform::BrowserWasmWebgpu
    );
    assert_eq!(summary.preprocessing_steps, 1);
    assert_eq!(summary.postprocessing_steps, 1);
    assert_eq!(summary.fixtures, 1);
}

#[test]
fn validates_v1_package_layout_and_metadata_contract() {
    let manifest = common::example_manifest();
    let summary = inspect_package_manifest(&manifest);

    assert_eq!(summary.schema_version, SchemaVersion::BiorsPackageV1);

    let layout = summary.package_layout.as_ref().expect("package layout");
    let metadata = summary.metadata.as_ref().expect("metadata");

    assert_eq!(layout.models, "models");
    assert_eq!(metadata.license, "CC0-1.0");
    assert_eq!(metadata.model_card, "docs/model-card.md");
    assert_eq!(layout.pipelines.as_deref(), Some("pipelines"));
    assert_eq!(
        summary.layout.pipeline_configs,
        vec!["pipelines/protein.toml"]
    );
    assert_eq!(
        manifest.preprocessing[0]
            .config
            .as_ref()
            .expect("pipeline config")
            .schema_version
            .to_string(),
        "biors.pipeline.v0"
    );

    let report = validate_package_manifest_artifacts(&manifest, &common::example_base_dir());

    assert!(report.valid, "{:?}", report.issues);
}

#[test]
fn rejects_v1_manifest_without_required_research_metadata() {
    let mut manifest = common::example_manifest();
    manifest.metadata = None;

    let report = validate_package_manifest(&manifest);

    assert!(!report.valid);
    assert!(report.structured_issues.iter().any(|issue| {
        issue.code == PackageValidationIssueCode::RequiredField && issue.field == "metadata"
    }));
}

#[test]
fn validates_required_package_manifest_fields() {
    let mut manifest = common::valid_manifest();
    manifest.name.clear();
    manifest.model.path.clear();
    manifest.fixtures[0].expected_output.clear();

    let report = validate_package_manifest(&manifest);

    assert!(!report.valid);
    assert_eq!(
        report.issues,
        vec![
            "name is required",
            "model.path is required",
            "fixtures[0].expected_output is required",
        ]
    );
}

#[test]
fn manifest_validation_fixture_covers_multiple_structured_failures() {
    let fixture =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/manifest/invalid_manifest.json");

    assert!(
        fixture.exists(),
        "missing manifest fixture: {}",
        fixture.display()
    );

    let manifest: PackageManifest =
        serde_json::from_str(&std::fs::read_to_string(fixture).expect("read manifest fixture"))
            .expect("fixture remains structurally deserializable");
    let report = validate_package_manifest(&manifest);
    let codes: Vec<_> = report
        .structured_issues
        .iter()
        .map(|issue| issue.code)
        .collect();

    assert!(!report.valid);
    assert!(codes.contains(&PackageValidationIssueCode::RequiredField));
    assert!(codes.contains(&PackageValidationIssueCode::MissingFixture));
    assert!(codes.contains(&PackageValidationIssueCode::InvalidShape));
}

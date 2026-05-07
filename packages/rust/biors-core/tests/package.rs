use biors_core::package::{
    inspect_package_manifest, plan_runtime_bridge, validate_package_manifest, ModelFormat,
    PackageManifest, PackageValidationIssueCode, RuntimeBackend, RuntimeTargetPlatform,
    SchemaVersion,
};
use std::path::Path;

fn valid_manifest() -> PackageManifest {
    serde_json::from_str(
        r#"{
          "schema_version": "biors.package.v0",
          "name": "protein-seed",
          "model": {
            "format": "onnx",
            "path": "models/protein-seed.onnx"
          },
          "preprocessing": [
            {
              "name": "protein_fasta_tokenize",
              "implementation": "biors-core",
              "contract": "protein-20"
            }
          ],
          "postprocessing": [
            {
              "name": "classification_scores",
              "implementation": "python-baseline",
              "contract": "float32-vector"
            }
          ],
          "runtime": {
            "backend": "onnx-webgpu",
            "target": "browser-wasm-webgpu"
          },
          "fixtures": [
            {
              "name": "tiny-protein",
              "input": "fixtures/tiny.fasta",
              "expected_output": "fixtures/tiny.output.json"
            }
          ]
        }"#,
    )
    .expect("valid manifest JSON")
}

fn example_manifest() -> PackageManifest {
    serde_json::from_str(
        &std::fs::read_to_string(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../../examples/protein-package/manifest.json"),
        )
        .expect("read example manifest"),
    )
    .expect("parse example manifest")
}

fn example_base_dir() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../examples/protein-package")
}

#[test]
fn inspects_portable_package_manifest() {
    let manifest = valid_manifest();
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
    let manifest = example_manifest();
    let summary = inspect_package_manifest(&manifest);

    assert_eq!(summary.schema_version, SchemaVersion::BiorsPackageV1);
    assert_eq!(
        summary
            .package_layout
            .as_ref()
            .expect("package layout")
            .models,
        "models"
    );
    assert_eq!(
        summary.metadata.as_ref().expect("metadata").license,
        "CC0-1.0"
    );
    assert_eq!(
        summary.metadata.as_ref().expect("metadata").model_card,
        "docs/model-card.md"
    );
    assert_eq!(
        summary
            .package_layout
            .as_ref()
            .expect("package layout")
            .pipelines
            .as_deref(),
        Some("pipelines")
    );
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

    let report =
        biors_core::package::validate_package_manifest_artifacts(&manifest, &example_base_dir());

    assert!(report.valid, "{:?}", report.issues);
}

#[test]
fn rejects_v1_manifest_without_required_research_metadata() {
    let mut manifest = example_manifest();
    manifest.metadata = None;

    let report = validate_package_manifest(&manifest);

    assert!(!report.valid);
    assert!(report.structured_issues.iter().any(|issue| {
        issue.code == PackageValidationIssueCode::RequiredField && issue.field == "metadata"
    }));
}

#[test]
fn rejects_v1_assets_outside_declared_package_layout() {
    let mut manifest = example_manifest();
    manifest.model.path = "artifacts/protein-seed.onnx".to_string();

    let report =
        biors_core::package::validate_package_manifest_artifacts(&manifest, &example_base_dir());

    assert!(!report.valid);
    assert!(report.structured_issues.iter().any(|issue| {
        issue.code == PackageValidationIssueCode::LayoutMismatch
            && issue.field == "model.path"
            && issue.message.contains("models")
    }));
}

#[test]
fn rejects_pipeline_config_outside_declared_pipeline_layout() {
    let mut manifest = example_manifest();
    manifest.preprocessing[0]
        .config
        .as_mut()
        .expect("pipeline config")
        .path = "configs/protein.toml".to_string();

    let report =
        biors_core::package::validate_package_manifest_artifacts(&manifest, &example_base_dir());

    assert!(!report.valid);
    assert!(report.structured_issues.iter().any(|issue| {
        issue.code == PackageValidationIssueCode::LayoutMismatch
            && issue.field == "preprocessing[0].config.path"
            && issue.message.contains("pipelines")
    }));
}

#[test]
fn rejects_pipeline_config_checksum_mismatch() {
    let mut manifest = example_manifest();
    manifest.preprocessing[0]
        .config
        .as_mut()
        .expect("pipeline config")
        .checksum =
        Some("sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb".to_string());

    let report =
        biors_core::package::validate_package_manifest_artifacts(&manifest, &example_base_dir());

    assert!(!report.valid);
    assert!(report.structured_issues.iter().any(|issue| {
        issue.code == PackageValidationIssueCode::ChecksumMismatch
            && issue.field == "preprocessing[0].config.checksum"
    }));
}

#[test]
fn validates_required_package_manifest_fields() {
    let mut manifest = valid_manifest();
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
fn rejects_invalid_checksum_format() {
    let mut manifest = valid_manifest();
    manifest.model.checksum = Some("draft-model-checksum".to_string());

    let report = biors_core::package::validate_package_manifest_artifacts(
        &manifest,
        std::path::Path::new("."),
    );

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
fn rejects_checksum_mismatch_against_real_artifact() {
    let mut manifest = example_manifest();
    manifest.model.checksum =
        Some("sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_string());

    let report =
        biors_core::package::validate_package_manifest_artifacts(&manifest, &example_base_dir());

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
fn rejects_missing_manifest_relative_artifact() {
    let mut manifest = example_manifest();
    manifest.vocab.as_mut().expect("vocab").path = "vocabs/missing.json".to_string();

    let report =
        biors_core::package::validate_package_manifest_artifacts(&manifest, &example_base_dir());

    assert!(!report.valid);
    assert!(report
        .issues
        .iter()
        .any(|issue| issue.contains("failed to read asset 'vocabs/missing.json'")));
}

#[test]
fn rejects_asset_paths_outside_package_root() {
    let mut manifest = example_manifest();
    manifest.fixtures[0].input = "../outside.fasta".to_string();

    let report =
        biors_core::package::validate_package_manifest_artifacts(&manifest, &example_base_dir());

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

#[test]
fn plans_supported_onnx_webgpu_runtime_bridge() {
    let manifest = valid_manifest();
    let report = plan_runtime_bridge(&manifest);

    assert!(report.ready);
    assert_eq!(report.backend, RuntimeBackend::OnnxWebgpu);
    assert_eq!(report.target, RuntimeTargetPlatform::BrowserWasmWebgpu);
    assert_eq!(report.execution_provider, "webgpu");
    assert!(report.blocking_issues.is_empty());
}

#[test]
fn rejects_unsupported_runtime_values_at_deserialization_time() {
    let error = serde_json::from_str::<PackageManifest>(
        r#"{
          "schema_version": "biors.package.v0",
          "name": "protein-seed",
          "model": { "format": "onnx", "path": "models/protein-seed.onnx" },
          "preprocessing": [],
          "postprocessing": [],
          "runtime": {
            "backend": "python",
            "target": "cpython-server"
          },
          "fixtures": [
            {
              "name": "tiny-protein",
              "input": "fixtures/tiny.fasta",
              "expected_output": "fixtures/tiny.output.json"
            }
          ]
        }"#,
    )
    .expect_err("unsupported runtime values should be rejected");

    assert!(error.to_string().contains("unknown variant"));
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

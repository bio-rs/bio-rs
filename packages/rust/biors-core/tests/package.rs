use biors_core::{
    inspect_package_manifest, plan_runtime_bridge, validate_package_manifest,
    validate_package_relative_path, ModelFormat, PackageManifest, RuntimeBackend,
    RuntimeTargetPlatform, SchemaVersion,
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

    let report =
        biors_core::validate_package_manifest_artifacts(&manifest, std::path::Path::new("."));

    assert!(!report.valid);
    assert!(report
        .issues
        .iter()
        .any(|issue| issue.contains("model.checksum")));
}

#[test]
fn rejects_checksum_mismatch_against_real_artifact() {
    let mut manifest = example_manifest();
    manifest.model.checksum =
        Some("sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_string());

    let report = biors_core::validate_package_manifest_artifacts(&manifest, &example_base_dir());

    assert!(!report.valid);
    assert!(report
        .issues
        .iter()
        .any(|issue| issue.contains("model.checksum mismatch")));
}

#[test]
fn rejects_missing_manifest_relative_artifact() {
    let mut manifest = example_manifest();
    manifest.vocab.as_mut().expect("vocab").path = "vocabs/missing.json".to_string();

    let report = biors_core::validate_package_manifest_artifacts(&manifest, &example_base_dir());

    assert!(!report.valid);
    assert!(report
        .issues
        .iter()
        .any(|issue| issue.contains("failed to read asset 'vocabs/missing.json'")));
}

#[test]
fn rejects_asset_paths_outside_package_root() {
    assert!(validate_package_relative_path("fixtures/tiny.fasta").is_ok());
    assert!(validate_package_relative_path("../outside.fasta").is_err());
    assert!(validate_package_relative_path("/tmp/outside.fasta").is_err());

    let mut manifest = example_manifest();
    manifest.fixtures[0].input = "../outside.fasta".to_string();

    let report = biors_core::validate_package_manifest_artifacts(&manifest, &example_base_dir());

    assert!(!report.valid);
    assert!(report
        .issues
        .iter()
        .any(|issue| issue.contains("must stay inside the package root")));
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

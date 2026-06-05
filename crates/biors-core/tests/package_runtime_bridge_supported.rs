use biors_core::package::{
    inspect_package_manifest, plan_runtime_bridge, ModelFormat, PackageManifest, RuntimeBackend,
    RuntimeTargetPlatform,
};

mod common;

#[test]
fn inspects_model_artifact_metadata_for_runtime_planning() {
    let manifest: PackageManifest = serde_json::from_str(
        r#"{
          "schema_version": "biors.package.v1",
          "name": "protein-seed",
          "package_layout": {
            "manifest": "manifest.json",
            "models": "models",
            "fixtures": "fixtures",
            "docs": "docs"
          },
          "metadata": {
            "license": { "expression": "CC0-1.0" },
            "citation": { "preferred_citation": "bio-rs package fixture" },
            "model_card": {
              "path": "docs/model-card.md",
              "summary": "Tiny deterministic package fixture.",
              "intended_use": ["Validate package tooling"],
              "limitations": ["Not suitable for scientific inference"]
            }
          },
          "model": {
            "format": "onnx",
            "path": "models/protein-seed.onnx",
            "metadata": {
              "name": "protein-seed-linear-probe",
              "version": "fixture-0",
              "architecture": "linear-probe",
              "task": "classification",
              "source": "local-fixture",
              "description": "Deterministic package fixture model"
            }
          },
          "preprocessing": [],
          "postprocessing": [],
          "runtime": {
            "backend": "onnx-webgpu",
            "target": "browser-wasm-webgpu",
            "version": "onnx-webgpu.v0"
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
    .expect("manifest with model metadata");

    let summary = inspect_package_manifest(&manifest);
    let metadata = summary.model_metadata.as_ref().expect("model metadata");
    assert_eq!(metadata.name, "protein-seed-linear-probe");
    assert_eq!(metadata.version.as_deref(), Some("fixture-0"));
    assert_eq!(metadata.architecture.as_deref(), Some("linear-probe"));
    assert_eq!(metadata.task.as_deref(), Some("classification"));

    let report = plan_runtime_bridge(&manifest);
    assert!(report.ready, "{:?}", report.blocking_issues);
    assert!(report.contract_ready, "{:?}", report.blocking_issues);
    assert!(!report.artifact_checked);
    assert!(!report.execution_ready);
    assert!(report
        .readiness_notes
        .iter()
        .any(|note| note.contains("not format-validated")));
    assert_eq!(report.model_format, ModelFormat::Onnx);
    assert_eq!(
        report
            .model_metadata
            .as_ref()
            .expect("bridge metadata")
            .name,
        "protein-seed-linear-probe"
    );
    assert!(report.compatibility_checks.iter().any(|check| {
        check.code == "runtime_model_pair"
            && check.passed
            && check
                .message
                .contains("onnx-webgpu/browser-wasm-webgpu supports onnx")
    }));
}

#[test]
fn plans_supported_onnx_webgpu_runtime_bridge() {
    let manifest = common::valid_manifest();
    let report = plan_runtime_bridge(&manifest);

    assert!(report.ready);
    assert!(report.contract_ready);
    assert!(!report.artifact_checked);
    assert!(!report.execution_ready);
    assert!(report
        .readiness_notes
        .iter()
        .any(|note| note.contains("ready/contract_ready only indicate")));
    assert_eq!(report.backend, RuntimeBackend::OnnxWebgpu);
    assert_eq!(report.target, RuntimeTargetPlatform::BrowserWasmWebgpu);
    assert_eq!(report.model_format, ModelFormat::Onnx);
    assert_eq!(report.backend_config.backend_id, "protein-seed:onnx-webgpu");
    assert_eq!(report.backend_config.provider, "webgpu");
    assert_eq!(
        report.backend_config.version.as_deref(),
        Some("onnx-webgpu.v0")
    );
    assert_eq!(
        report.backend_config.model_artifact.as_deref(),
        Some("models/protein-seed.onnx")
    );
    assert_eq!(report.execution_provider, "webgpu");
    assert!(report
        .compatibility_checks
        .iter()
        .any(|check| { check.code == "runtime_model_pair" && check.passed }));
    assert!(report.blocking_issues.is_empty());
}

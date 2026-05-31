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

#[test]
fn runtime_bridge_rejects_incompatible_model_backend_pair() {
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
              "name": "protein-seed-linear-probe"
            }
          },
          "preprocessing": [],
          "postprocessing": [],
          "runtime": {
            "backend": "candle",
            "target": "local-cpu",
            "version": "candle.v0"
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
    .expect("manifest with incompatible runtime pair");

    let report = plan_runtime_bridge(&manifest);

    assert!(!report.ready);
    assert_eq!(report.backend, RuntimeBackend::Candle);
    assert_eq!(report.target, RuntimeTargetPlatform::LocalCpu);
    assert!(report.blocking_issues.iter().any(|issue| {
        issue.contains("model format 'onnx'")
            && issue.contains("backend 'candle'")
            && issue.contains("target 'local-cpu'")
    }));
    assert!(report
        .compatibility_checks
        .iter()
        .any(|check| { check.code == "runtime_model_pair" && !check.passed }));
}

#[test]
fn runtime_bridge_blocks_external_process_manifest_backend() {
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
              "name": "protein-seed-linear-probe"
            }
          },
          "preprocessing": [],
          "postprocessing": [],
          "runtime": {
            "backend": "external-process",
            "target": "local-cpu",
            "version": "external-process.v0"
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
    .expect("manifest with external-process runtime");

    let report = plan_runtime_bridge(&manifest);

    assert!(!report.ready);
    assert_eq!(report.backend, RuntimeBackend::ExternalProcess);
    assert_eq!(report.target, RuntimeTargetPlatform::LocalCpu);
    assert_eq!(report.execution_provider, "external-process");
    assert!(report.blocking_issues.iter().any(|issue| {
        issue.contains("external-process")
            && issue.contains("not supported by the public package manifest contract")
    }));
    assert!(report
        .compatibility_checks
        .iter()
        .any(|check| { check.code == "runtime_model_pair" && check.passed }));
}

#[test]
fn runtime_bridge_reports_backend_capabilities() {
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
            "format": "safetensors",
            "path": "models/protein-seed.safetensors",
            "metadata": {
              "name": "protein-seed-linear-probe"
            }
          },
          "preprocessing": [],
          "postprocessing": [],
          "runtime": {
            "backend": "candle",
            "target": "local-cpu",
            "version": "candle.v0"
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
    .expect("manifest with candle runtime");

    let report = plan_runtime_bridge(&manifest);

    assert!(report.ready);
    let caps = report
        .backend_capabilities
        .as_ref()
        .expect("backend capabilities present");
    assert!(caps.deterministic);
    assert!(caps.supports_batch);
    assert!(!caps.supports_streaming);
    assert!(caps
        .supported_inputs
        .contains(&"biors.model-input.v0+json".to_string()));
    assert!(caps
        .supported_outputs
        .contains(&"biors.candle.linear-probe.v0+json".to_string()));
    assert!(report
        .compatibility_checks
        .iter()
        .any(|check| check.code == "backend_capabilities" && check.passed));
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

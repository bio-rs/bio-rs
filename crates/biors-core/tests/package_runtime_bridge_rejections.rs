use biors_core::package::{
    plan_runtime_bridge, PackageManifest, RuntimeBackend, RuntimeTargetPlatform,
};

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
    assert!(!report.contract_ready);
    assert!(!report.artifact_checked);
    assert!(!report.execution_ready);
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
    assert!(!report.contract_ready);
    assert!(!report.artifact_checked);
    assert!(!report.execution_ready);
    assert_eq!(report.backend, RuntimeBackend::ExternalProcess);
    assert_eq!(report.target, RuntimeTargetPlatform::LocalCpu);
    assert_eq!(report.execution_provider, "external-process");
    assert!(report.blocking_issues.iter().any(|issue| {
        issue.contains("external-process")
            && issue.contains("not supported by the public package manifest contract")
    }));
    let runtime_check = report
        .compatibility_checks
        .iter()
        .find(|check| check.code == "runtime_model_pair")
        .expect("runtime-model compatibility check");
    assert!(!runtime_check.passed);
    assert!(runtime_check.message.contains("external-process"));
    assert!(runtime_check
        .message
        .contains("not supported by the public package manifest contract"));
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

use biors_core::package::{plan_runtime_bridge, PackageManifest};

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

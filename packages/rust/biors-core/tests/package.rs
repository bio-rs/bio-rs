use biors_core::{
    inspect_package_manifest, plan_runtime_bridge, validate_package_manifest, PackageManifest,
};

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

#[test]
fn inspects_portable_package_manifest() {
    let manifest = valid_manifest();
    let summary = inspect_package_manifest(&manifest);

    assert_eq!(summary.schema_version, "biors.package.v0");
    assert_eq!(summary.name, "protein-seed");
    assert_eq!(summary.model_format, "onnx");
    assert_eq!(summary.runtime_backend, "onnx-webgpu");
    assert_eq!(summary.runtime_target, "browser-wasm-webgpu");
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
fn plans_supported_onnx_webgpu_runtime_bridge() {
    let manifest = valid_manifest();
    let report = plan_runtime_bridge(&manifest);

    assert!(report.ready);
    assert_eq!(report.backend, "onnx-webgpu");
    assert_eq!(report.target, "browser-wasm-webgpu");
    assert_eq!(report.execution_provider, "webgpu");
    assert!(report.blocking_issues.is_empty());
}

#[test]
fn classifies_unsupported_runtime_bridge() {
    let mut manifest = valid_manifest();
    manifest.runtime.backend = "python".to_string();
    manifest.runtime.target = "cpython-server".to_string();

    let report = plan_runtime_bridge(&manifest);

    assert!(!report.ready);
    assert_eq!(
        report.blocking_issues,
        vec![
            "runtime.backend 'python' is not supported by the portable bridge",
            "runtime.target 'cpython-server' is not supported by the portable bridge",
        ]
    );
}

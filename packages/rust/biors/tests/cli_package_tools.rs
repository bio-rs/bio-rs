use serde_json::Value;
use std::path::Path;

mod common;
use common::{ChildInputExt, TempDir};

const V0_MANIFEST: &str = r#"{
  "schema_version": "biors.package.v0",
  "name": "protein-seed",
  "model": {
    "format": "onnx",
    "path": "models/protein-seed.onnx"
  },
  "preprocessing": [],
  "postprocessing": [],
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
}"#;

#[test]
fn package_migrate_outputs_v0_to_v1_plan() {
    let output = common::spawn_biors(&["package", "migrate", "-", "--to", "biors.package.v1"])
        .tap_stdin(V0_MANIFEST);

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stderr.is_empty());

    let value: Value = serde_json::from_slice(&output.stdout).expect("valid JSON output");

    assert_eq!(value["data"]["package"], "protein-seed");
    assert_eq!(value["data"]["from"], "biors.package.v0");
    assert_eq!(value["data"]["to"], "biors.package.v1");
    assert_eq!(value["data"]["compatibility"], "migration_required");
    assert_eq!(value["data"]["automatic"], false);
    assert!(value["data"]["required_steps"]
        .as_array()
        .expect("required steps")
        .iter()
        .any(|step| step.as_str().expect("step").contains("metadata.license")));
}

#[test]
fn package_compatibility_reports_schema_migration() {
    let temp = TempDir::new("package-compatibility");
    let v0 = temp.write("manifest.v0.json", V0_MANIFEST);
    let v1 = example_manifest();

    let output = common::run_biors_paths(&["package", "compatibility"], &[&v0, &v1]);
    let value: Value = serde_json::from_slice(&output.stdout).expect("valid JSON output");

    assert_eq!(value["data"]["left_package"], "protein-seed");
    assert_eq!(value["data"]["right_package"], "protein-seed");
    assert_eq!(value["data"]["left_schema_version"], "biors.package.v0");
    assert_eq!(value["data"]["right_schema_version"], "biors.package.v1");
    assert_eq!(value["data"]["compatibility"], "migration_required");
    assert_eq!(value["data"]["schema_compatible"], true);
    assert_eq!(value["data"]["migration_required"], true);
    assert_eq!(value["data"]["same_package_name"], true);
}

#[test]
fn package_diff_reports_manifest_content_changes() {
    let temp = TempDir::new("package-diff");
    let v0 = temp.write("manifest.v0.json", V0_MANIFEST);
    let v1 = example_manifest();

    let output = common::run_biors_paths(&["package", "diff"], &[&v0, &v1]);
    let value: Value = serde_json::from_slice(&output.stdout).expect("valid JSON output");

    assert_eq!(value["data"]["left_schema_version"], "biors.package.v0");
    assert_eq!(value["data"]["right_schema_version"], "biors.package.v1");
    assert_eq!(value["data"]["compatibility"], "migration_required");
    assert_eq!(value["data"]["same_package_name"], true);
    assert_eq!(value["data"]["diff"]["matches"], false);
    assert!(value["data"]["diff"]["content_diff"].is_object());
}

fn example_manifest() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../examples/protein-package/manifest.json")
}

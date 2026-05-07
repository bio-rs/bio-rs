use jsonschema::JSONSchema;
use serde_json::Value;
use std::fs;
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
fn cli_outputs_match_package_tooling_schemas() {
    let temp = TempDir::new("package-tooling-schema");
    let v0 = temp.write("manifest.v0.json", V0_MANIFEST);
    let v1 = repo_root().join("examples/protein-package/manifest.json");

    let migration = common::spawn_biors(&["package", "migrate", "-", "--to", "biors.package.v1"])
        .tap_stdin(V0_MANIFEST)
        .stdout;
    assert_payload_matches_schema(&migration, "schemas/package-migration-output.v0.json");

    let conversion = common::spawn_biors(&[
        "package",
        "convert",
        "-",
        "--to",
        "biors.package.v1",
        "--license",
        "CC0-1.0",
        "--citation",
        "bio-rs converted fixture",
        "--model-card",
        "docs/model-card.md",
        "--model-card-summary",
        "Converted package fixture for schema tests.",
        "--intended-use",
        "Schema validation",
        "--limitation",
        "Not for inference",
    ])
    .tap_stdin(V0_MANIFEST)
    .stdout;
    assert_payload_matches_schema(&conversion, "schemas/package-conversion-output.v0.json");

    let compatibility = common::run_biors_paths(&["package", "compatibility"], &[&v0, &v1]).stdout;
    assert_payload_matches_schema(
        &compatibility,
        "schemas/package-compatibility-output.v0.json",
    );

    let diff = common::run_biors_paths(&["package", "diff"], &[&v0, &v1]).stdout;
    assert_payload_matches_schema(&diff, "schemas/package-diff-output.v0.json");
}

fn assert_payload_matches_schema(output: &[u8], schema_path: &str) {
    let envelope: Value = serde_json::from_slice(output).expect("valid CLI JSON");
    assert_json_value_matches_schema(&envelope, "schemas/cli-success.v0.json");
    assert_json_value_matches_schema(&envelope["data"], schema_path);
}

fn assert_json_value_matches_schema(value: &Value, schema_path: &str) {
    let schema: Value = serde_json::from_str(
        &fs::read_to_string(repo_root().join(schema_path)).expect("read payload schema"),
    )
    .expect("schema JSON");
    let compiled = JSONSchema::compile(&schema).expect("compile schema");
    let validation = compiled.validate(value);
    if let Err(errors) = validation {
        let messages: Vec<_> = errors.map(|error| error.to_string()).collect();
        panic!("JSON did not match schema {schema_path}: {messages:?}");
    }
}

fn repo_root() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..")
}

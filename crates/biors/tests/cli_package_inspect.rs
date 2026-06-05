use serde_json::Value;
use std::process::Command;

mod common;
mod package_support;

#[test]
fn package_inspect_outputs_manifest_summary() {
    let output = Command::new(env!("CARGO_BIN_EXE_biors"))
        .arg("package")
        .arg("inspect")
        .arg(package_support::example_manifest_path())
        .output()
        .expect("run biors package inspect");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let value: Value = serde_json::from_slice(&output.stdout).expect("valid JSON output");

    assert_eq!(value["data"]["schema_version"], "biors.package.v1");
    assert_eq!(value["data"]["name"], "protein-seed");
    assert_eq!(value["data"]["package_layout"]["models"], "models");
    assert_eq!(value["data"]["package_layout"]["pipelines"], "pipelines");
    assert_eq!(value["data"]["package_layout"]["docs"], "docs");
    assert_eq!(value["data"]["metadata"]["license"], "CC0-1.0");
    assert_eq!(
        value["data"]["metadata"]["model_card"],
        "docs/model-card.md"
    );
    assert_eq!(value["data"]["model_format"], "onnx");
    assert_eq!(value["data"]["has_model_checksum"], true);
    assert_eq!(value["data"]["runtime_backend"], "onnx-webgpu");
    assert_eq!(value["data"]["runtime_target"], "browser-wasm-webgpu");
    assert_eq!(value["data"]["fixtures"], 1);
    assert_eq!(value["data"]["layout"]["model"], "models/protein-seed.onnx");
    assert_eq!(
        value["data"]["layout"]["tokenizer"],
        "tokenizers/protein-20.json"
    );
    assert_eq!(value["data"]["layout"]["vocab"], "vocabs/protein-20.json");
    assert_eq!(
        value["data"]["layout"]["fixture_inputs"][0],
        "fixtures/tiny.fasta"
    );
    assert_eq!(
        value["data"]["layout"]["fixture_outputs"][0],
        "fixtures/tiny.output.json"
    );
    assert_eq!(
        value["data"]["layout"]["pipeline_configs"][0],
        "pipelines/protein.toml"
    );
}

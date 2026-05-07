use serde_json::Value;
use std::process::Command;

mod common;
use common::{ChildInputExt, TempDir};

const V0_MANIFEST: &str = r#"{
  "schema_version": "biors.package.v0",
  "name": "protein-seed",
  "model": {
    "format": "onnx",
    "path": "models/protein-seed.onnx",
    "checksum": "sha256:2c1da72b15fab35bd6f1bb62f5037b936e26e6413a220fa9afe5a64bce0df68d"
  },
  "tokenizer": {
    "name": "protein-20",
    "path": "tokenizers/protein-20.json",
    "contract_version": "protein-20.v0"
  },
  "vocab": {
    "name": "protein-20",
    "path": "vocabs/protein-20.json",
    "contract_version": "protein-20.v0"
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
fn package_convert_writes_v1_manifest_with_author_metadata() {
    let temp = TempDir::new("package-convert");
    let input = temp.write("manifest.v0.json", V0_MANIFEST);
    let output_path = temp.path().join("manifest.json");

    let output = Command::new(env!("CARGO_BIN_EXE_biors"))
        .arg("package")
        .arg("convert")
        .arg(input)
        .arg("--output")
        .arg(&output_path)
        .args(conversion_metadata_args())
        .output()
        .expect("run biors package convert");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stderr.is_empty());

    let value: Value = serde_json::from_slice(&output.stdout).expect("valid JSON output");
    assert_eq!(value["data"]["report"]["package"], "protein-seed");
    assert_eq!(value["data"]["report"]["from"], "biors.package.v0");
    assert_eq!(value["data"]["report"]["to"], "biors.package.v1");
    assert_eq!(value["data"]["report"]["converted"], true);
    assert_eq!(value["data"]["report"]["metadata_supplied"], true);
    assert!(value["data"]["report"]["manifest_sha256"]
        .as_str()
        .expect("manifest hash")
        .starts_with("sha256:"));

    let manifest = &value["data"]["manifest"];
    assert_eq!(manifest["schema_version"], "biors.package.v1");
    assert_eq!(manifest["package_layout"]["manifest"], "manifest.json");
    assert_eq!(manifest["package_layout"]["models"], "models");
    assert_eq!(manifest["package_layout"]["tokenizers"], "tokenizers");
    assert_eq!(manifest["package_layout"]["vocabs"], "vocabs");
    assert_eq!(manifest["package_layout"]["fixtures"], "fixtures");
    assert_eq!(manifest["package_layout"]["docs"], "docs");
    assert_eq!(manifest["metadata"]["license"]["expression"], "CC0-1.0");
    assert_eq!(
        manifest["metadata"]["citation"]["preferred_citation"],
        "bio-rs converted fixture"
    );
    assert_eq!(
        manifest["metadata"]["model_card"]["path"],
        "docs/model-card.md"
    );
    assert_eq!(
        manifest["metadata"]["model_card"]["intended_use"][0],
        "CLI conversion test"
    );
    assert_eq!(
        manifest["metadata"]["model_card"]["limitations"][0],
        "Not for inference"
    );

    let written: Value = serde_json::from_slice(
        &std::fs::read(output_path).expect("read converted manifest from output path"),
    )
    .expect("written manifest JSON");
    assert_eq!(written, *manifest);
}

#[test]
fn package_convert_reports_missing_v1_metadata() {
    let output = common::spawn_biors(&[
        "--json",
        "package",
        "convert",
        "-",
        "--to",
        "biors.package.v1",
    ])
    .tap_stdin(V0_MANIFEST);

    assert_eq!(output.status.code(), Some(2));
    assert!(output.stderr.is_empty());

    let value: Value = serde_json::from_slice(&output.stdout).expect("valid JSON error");
    assert_eq!(
        value["error"]["code"],
        "package.conversion_missing_metadata"
    );
}

#[test]
fn package_convert_project_creates_valid_package_skeleton() {
    let temp = TempDir::new("package-convert-project");
    let project = temp.path().join("python-project");
    std::fs::create_dir_all(&project).expect("create python project");
    std::fs::write(project.join("model.onnx"), b"placeholder onnx").expect("write model");
    std::fs::write(
        project.join("tokenizer_config.json"),
        r#"{
  "tokenizer_class": "BertTokenizer",
  "do_lower_case": false,
  "cls_token": "[CLS]",
  "sep_token": "[SEP]",
  "pad_token": "[PAD]",
  "unk_token": "[UNK]",
  "mask_token": "[MASK]"
}"#,
    )
    .expect("write tokenizer config");
    let fixture_input = temp.write("tiny.fasta", ">tiny\nACDE\n");
    let fixture_output = temp.write("tiny.output.json", r#"{"label":"fixture","score":1.0}"#);
    let output_dir = temp.path().join("package");

    let output = Command::new(env!("CARGO_BIN_EXE_biors"))
        .arg("package")
        .arg("convert-project")
        .arg(&project)
        .arg("--output")
        .arg(&output_dir)
        .arg("--name")
        .arg("protein-project")
        .args(skeleton_metadata_args())
        .arg("--fixture-input")
        .arg(&fixture_input)
        .arg("--fixture-output")
        .arg(&fixture_output)
        .output()
        .expect("run biors package convert-project");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stderr.is_empty());

    let value: Value = serde_json::from_slice(&output.stdout).expect("valid JSON output");
    assert_eq!(value["data"]["package"], "protein-project");
    assert!(value["data"]["manifest_sha256"]
        .as_str()
        .expect("manifest hash")
        .starts_with("sha256:"));

    let manifest_path = output_dir.join("manifest.json");
    let manifest: Value =
        serde_json::from_slice(&std::fs::read(&manifest_path).expect("read generated manifest"))
            .expect("manifest JSON");
    assert_eq!(manifest["schema_version"], "biors.package.v1");
    assert_eq!(manifest["tokenizer"]["name"], "protein-20-special");
    assert_eq!(
        manifest["preprocessing"][0]["config"]["path"],
        "pipelines/protein.toml"
    );

    let validate = Command::new(env!("CARGO_BIN_EXE_biors"))
        .arg("package")
        .arg("validate")
        .arg(&manifest_path)
        .output()
        .expect("validate generated package");
    assert!(
        validate.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&validate.stderr)
    );
}

fn conversion_metadata_args() -> [&'static str; 16] {
    [
        "--license",
        "CC0-1.0",
        "--citation",
        "bio-rs converted fixture",
        "--model-card",
        "docs/model-card.md",
        "--model-card-summary",
        "Converted package fixture for CLI tests.",
        "--intended-use",
        "CLI conversion test",
        "--limitation",
        "Not for inference",
        "--license-file",
        "docs/LICENSE.txt",
        "--citation-file",
        "docs/CITATION.cff",
    ]
}

fn skeleton_metadata_args() -> [&'static str; 10] {
    [
        "--license",
        "CC0-1.0",
        "--citation",
        "bio-rs converted fixture",
        "--model-card-summary",
        "Converted package fixture for CLI tests.",
        "--intended-use",
        "CLI conversion test",
        "--limitation",
        "Not for inference",
    ]
}

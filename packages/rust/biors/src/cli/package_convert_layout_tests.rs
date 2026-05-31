use super::*;
use crate::cli::package_args::PackageSchemaArg;
use std::path::PathBuf;

const V0_MANIFEST: &str = r#"{
  "schema_version": "biors.package.v0",
  "name": "protein-seed",
  "model": {
    "format": "onnx",
    "path": "models/protein-seed.onnx"
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
fn conversion_input_infers_layout_dirs_from_manifest_and_metadata() {
    let input = build_conversion_input(&default_args(), &manifest()).expect("conversion input");

    assert_eq!(input.package_layout.manifest, "manifest.v1.json");
    assert_eq!(input.package_layout.models, "models");
    assert_eq!(
        input.package_layout.tokenizers.as_deref(),
        Some("tokenizers")
    );
    assert_eq!(input.package_layout.vocabs.as_deref(), Some("vocabs"));
    assert_eq!(input.package_layout.fixtures, "fixtures");
    assert_eq!(input.package_layout.docs, "docs");
    assert_eq!(input.metadata.license.expression, "CC0-1.0");
}

#[test]
fn conversion_input_rejects_layout_override_outside_declared_paths() {
    let mut args = default_args();
    args.models_dir = Some("external-models".to_string());

    let error = build_conversion_input(&args, &manifest()).expect_err("layout conflict");
    assert!(format!("{error}").contains("all paths for --models-dir"));
}

#[test]
fn conversion_input_rejects_absolute_layout_override() {
    let mut args = default_args();
    args.docs_dir = Some("/tmp/docs".to_string());

    let error = build_conversion_input(&args, &manifest()).expect_err("layout conflict");
    assert!(format!("{error}").contains("must be a non-empty package-relative path"));
}

fn manifest() -> PackageManifest {
    serde_json::from_str(V0_MANIFEST).expect("manifest")
}

fn default_args() -> PackageConvertArgs {
    PackageConvertArgs {
        path: PathBuf::from("manifest.v0.json"),
        to: PackageSchemaArg::BiorsPackageV1,
        output: Some(PathBuf::from("manifest.v1.json")),
        license: Some("CC0-1.0".to_string()),
        citation: Some("fixture citation".to_string()),
        doi: None,
        model_card: Some("docs/model-card.md".to_string()),
        model_card_summary: Some("Fixture model card".to_string()),
        intended_use: vec!["testing".to_string()],
        limitations: vec!["not for inference".to_string()],
        license_file: Some("docs/LICENSE.txt".to_string()),
        citation_file: Some("docs/CITATION.cff".to_string()),
        models_dir: None,
        tokenizers_dir: None,
        vocabs_dir: None,
        pipelines_dir: None,
        fixtures_dir: None,
        observed_dir: None,
        docs_dir: None,
    }
}

#![allow(dead_code)]

use biors_core::package::PackageManifest;
use std::fs;
use std::path::{Path, PathBuf};

pub fn valid_manifest() -> PackageManifest {
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

pub fn example_manifest() -> PackageManifest {
    serde_json::from_str(
        &fs::read_to_string(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../testdata/protein-package/manifest.json"),
        )
        .expect("read example manifest"),
    )
    .expect("parse example manifest")
}

pub fn example_base_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../testdata/protein-package")
}

pub fn temp_package_dir(name: &str) -> PathBuf {
    let path = std::env::temp_dir().join(format!(
        "biors-{name}-{}-{}",
        std::process::id(),
        std::thread::current().name().unwrap_or("test")
    ));
    let _ = fs::remove_dir_all(&path);
    fs::create_dir_all(&path).expect("create temp package dir");
    path
}

use serde_json::Value;
use std::process::Command;

mod common;
mod package_support;
use common::TempDir;

#[test]
fn package_init_writes_non_misleading_metadata_files() {
    let temp = TempDir::new("package-init-metadata-files");
    let model = temp.write("model.onnx", "model");
    let fixture_input = temp.write("tiny.fasta", ">tiny\nACDE\n");
    let fixture_output = temp.write("tiny.output.json", r#"{"label":"fixture","score":1.0}"#);
    let output_dir = temp.path().join("package");

    let output = Command::new(env!("CARGO_BIN_EXE_biors"))
        .arg("package")
        .arg("init")
        .arg(&output_dir)
        .arg("--name")
        .arg("protein-init")
        .arg("--model")
        .arg(&model)
        .arg("--fixture-input")
        .arg(&fixture_input)
        .arg("--fixture-output")
        .arg(&fixture_output)
        .arg("--license")
        .arg("MIT")
        .arg("--citation")
        .arg("Smith et al. 2026")
        .arg("--doi")
        .arg("10.1234/example")
        .arg("--model-card-summary")
        .arg("Converted package fixture for CLI tests.")
        .arg("--intended-use")
        .arg("CLI conversion test")
        .arg("--limitation")
        .arg("Not for inference")
        .output()
        .expect("run biors package init");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stderr.is_empty());

    let manifest: Value = serde_json::from_slice(
        &std::fs::read(output_dir.join("manifest.json")).expect("read generated manifest"),
    )
    .expect("manifest JSON");
    assert_eq!(
        manifest["metadata"]["license"]["file"]["path"],
        "docs/LICENSE-SPDX.txt"
    );
    assert_eq!(
        manifest["metadata"]["citation"]["file"]["path"],
        "docs/CITATION.txt"
    );
    assert_eq!(
        std::fs::read_to_string(output_dir.join("docs/LICENSE-SPDX.txt")).expect("read license"),
        "SPDX-License-Identifier: MIT\n"
    );
    assert_eq!(
        std::fs::read_to_string(output_dir.join("docs/CITATION.txt")).expect("read citation"),
        "Smith et al. 2026\nDOI: 10.1234/example\n"
    );
    assert!(!output_dir.join("docs/CITATION.cff").exists());
}

#[test]
fn package_init_pipeline_matches_supplied_tokenizer_profile() {
    let temp = TempDir::new("package-init-dna-profile");
    let model = temp.write("model.onnx", "model");
    let tokenizer_config = temp.write(
        "dna-tokenizer.json",
        r#"{"profile":"dna-iupac","add_special_tokens":false}"#,
    );
    let fixture_input = temp.write("tiny.fasta", ">dna\nACGT\n");
    let fixture_output = temp.write("tiny.output.json", r#"{"label":"fixture","score":1.0}"#);
    let output_dir = temp.path().join("package");

    let output = Command::new(env!("CARGO_BIN_EXE_biors"))
        .arg("package")
        .arg("init")
        .arg(&output_dir)
        .arg("--name")
        .arg("dna-init")
        .arg("--model")
        .arg(&model)
        .arg("--tokenizer-config")
        .arg(&tokenizer_config)
        .arg("--fixture-input")
        .arg(&fixture_input)
        .arg("--fixture-output")
        .arg(&fixture_output)
        .args(package_support::skeleton_metadata_args())
        .output()
        .expect("run biors package init");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stderr.is_empty());

    let manifest: Value = serde_json::from_slice(
        &std::fs::read(output_dir.join("manifest.json")).expect("read generated manifest"),
    )
    .expect("manifest JSON");
    assert_eq!(
        manifest["preprocessing"][0]["config"]["path"],
        "pipelines/dna.toml"
    );
    assert_eq!(manifest["preprocessing"][0]["name"], "dna_fasta_tokenize");
    assert_eq!(manifest["preprocessing"][0]["contract"], "dna-iupac");
    let pipeline_config =
        std::fs::read_to_string(output_dir.join("pipelines/dna.toml")).expect("read pipeline");
    assert!(pipeline_config.contains(r#"kind = "dna""#));
    assert!(pipeline_config.contains(r#"profile = "dna-iupac""#));
}

#[test]
fn package_init_pipeline_matches_supplied_rna_tokenizer_profile() {
    let temp = TempDir::new("package-init-rna-profile");
    let model = temp.write("model.onnx", "model");
    let tokenizer_config = temp.write(
        "rna-tokenizer.json",
        r#"{"profile":"rna-iupac","add_special_tokens":false}"#,
    );
    let fixture_input = temp.write("tiny.fasta", ">rna\nACGU\n");
    let fixture_output = temp.write("tiny.output.json", r#"{"label":"fixture","score":1.0}"#);
    let output_dir = temp.path().join("package");

    let output = Command::new(env!("CARGO_BIN_EXE_biors"))
        .arg("package")
        .arg("init")
        .arg(&output_dir)
        .arg("--name")
        .arg("rna-init")
        .arg("--model")
        .arg(&model)
        .arg("--tokenizer-config")
        .arg(&tokenizer_config)
        .arg("--fixture-input")
        .arg(&fixture_input)
        .arg("--fixture-output")
        .arg(&fixture_output)
        .args(package_support::skeleton_metadata_args())
        .output()
        .expect("run biors package init");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stderr.is_empty());

    let manifest: Value = serde_json::from_slice(
        &std::fs::read(output_dir.join("manifest.json")).expect("read generated manifest"),
    )
    .expect("manifest JSON");
    assert_eq!(
        manifest["preprocessing"][0]["config"]["path"],
        "pipelines/rna.toml"
    );
    assert_eq!(manifest["preprocessing"][0]["name"], "rna_fasta_tokenize");
    assert_eq!(manifest["preprocessing"][0]["contract"], "rna-iupac");
    let pipeline_config =
        std::fs::read_to_string(output_dir.join("pipelines/rna.toml")).expect("read pipeline");
    assert!(pipeline_config.contains(r#"kind = "rna""#));
    assert!(pipeline_config.contains(r#"profile = "rna-iupac""#));
}

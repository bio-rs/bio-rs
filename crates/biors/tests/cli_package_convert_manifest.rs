use serde_json::Value;
use std::process::Command;

mod common;
mod package_support;
use common::{ChildInputExt, TempDir};

#[test]
fn package_convert_writes_v1_manifest_with_author_metadata() {
    let temp = TempDir::new("package-convert");
    let input = temp.write("manifest.v0.json", package_support::V0_MANIFEST);
    let output_path = temp.path().join("manifest.json");

    let output = Command::new(env!("CARGO_BIN_EXE_biors"))
        .arg("package")
        .arg("convert")
        .arg(input)
        .arg("--output")
        .arg(&output_path)
        .args(package_support::conversion_metadata_args())
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
    .tap_stdin(package_support::V0_MANIFEST);

    assert_eq!(output.status.code(), Some(2));
    assert!(output.stderr.is_empty());

    let value: Value = serde_json::from_slice(&output.stdout).expect("valid JSON error");
    assert_eq!(
        value["error"]["code"],
        "package.conversion_missing_metadata"
    );
}

use serde_json::Value;
use std::fs;
use std::process::Command;

use super::metadata_args::skeleton_metadata_args;
use crate::common;

pub fn run_package_init_with_model(model_name: &str) -> Value {
    let temp = common::TempDir::new("package-init-model-format");
    let model = temp.write(model_name, "model");
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
        .args(skeleton_metadata_args())
        .output()
        .expect("run biors package init");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stderr.is_empty());

    serde_json::from_slice(
        &fs::read(output_dir.join("manifest.json")).expect("read generated manifest"),
    )
    .expect("generated manifest JSON")
}

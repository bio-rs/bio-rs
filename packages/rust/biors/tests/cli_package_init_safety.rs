use serde_json::Value;
use std::process::Command;

mod common;
mod package_support;
use common::TempDir;

#[test]
fn package_init_rejects_unknown_model_extension() {
    let temp = TempDir::new("package-init-unknown-model");
    let model = temp.write("model.bin", "unknown model");
    let fixture_input = temp.write("tiny.fasta", ">tiny\nACDE\n");
    let fixture_output = temp.write("tiny.output.json", r#"{"label":"fixture","score":1.0}"#);
    let output_dir = temp.path().join("package");

    let output = Command::new(env!("CARGO_BIN_EXE_biors"))
        .arg("--json")
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
        .args(package_support::skeleton_metadata_args())
        .output()
        .expect("run biors package init");

    assert_eq!(output.status.code(), Some(2));
    assert!(output.stderr.is_empty());
    let value: Value = serde_json::from_slice(&output.stdout).expect("valid JSON error");
    assert_eq!(
        value["error"]["code"],
        "package.init_unsupported_model_format"
    );
    assert!(value["error"]["location"]
        .as_str()
        .expect("error location")
        .ends_with("model.bin"));
    assert!(
        !output_dir.join("manifest.json").exists(),
        "unsupported model format must fail before writing manifest"
    );
    assert!(
        !output_dir.exists(),
        "unsupported model format must fail before creating package files"
    );
}

#[test]
fn package_init_rejects_existing_generated_targets_without_force() {
    for collision_rel in [
        "models/model.onnx",
        "fixtures/tiny.fasta",
        "fixtures/tiny.output.json",
        "tokenizers/protein-20.json",
        "pipelines/protein.toml",
        "docs/LICENSE-SPDX.txt",
        "docs/CITATION.txt",
        "docs/model-card.md",
    ] {
        let temp = TempDir::new("package-init-collision");
        let model = temp.write("model.onnx", "new model");
        let fixture_input = temp.write("tiny.fasta", ">tiny\nACDE\n");
        let fixture_output = temp.write("tiny.output.json", r#"{"label":"fixture","score":1.0}"#);
        let output_dir = temp.path().join("package");
        let collision_path = output_dir.join(collision_rel);
        std::fs::create_dir_all(collision_path.parent().expect("collision parent"))
            .expect("create collision parent");
        std::fs::write(&collision_path, "existing").expect("write collision file");

        let output = Command::new(env!("CARGO_BIN_EXE_biors"))
            .arg("--json")
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
            .args(package_support::skeleton_metadata_args())
            .output()
            .expect("run biors package init");

        assert_eq!(output.status.code(), Some(2), "{collision_rel}");
        assert!(output.stderr.is_empty(), "{collision_rel}");
        let value: Value = serde_json::from_slice(&output.stdout).expect("valid JSON error");
        assert_eq!(value["error"]["code"], "package.init_exists");
        assert!(value["error"]["location"]
            .as_str()
            .expect("collision location")
            .contains(collision_rel));
        assert_eq!(
            std::fs::read_to_string(&collision_path).expect("read collision file"),
            "existing"
        );
        assert!(
            !output_dir.join("manifest.json").exists(),
            "package init should fail before writing manifest for {collision_rel}"
        );
    }
}

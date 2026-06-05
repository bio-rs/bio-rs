use serde_json::Value;
use std::process::Command;

mod common;
mod package_support;
use common::TempDir;

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
        .args(package_support::skeleton_metadata_args())
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
    let pipeline_config =
        std::fs::read_to_string(output_dir.join("pipelines/protein.toml")).expect("read pipeline");
    assert!(pipeline_config.contains(r#"profile = "protein-20-special""#));

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

#[test]
fn package_convert_project_skips_generated_model_directories() {
    let temp = TempDir::new("package-convert-project-skip-generated");
    let project = temp.path().join("python-project");
    std::fs::create_dir_all(project.join(".venv/cache")).expect("create cache");
    std::fs::create_dir_all(project.join("export")).expect("create export");
    std::fs::write(project.join(".venv/cache/cached.onnx"), b"cached").expect("write cached model");
    std::fs::write(project.join("export/real.onnx"), b"real").expect("write real model");
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
        .args(package_support::skeleton_metadata_args())
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
    assert_eq!(
        std::fs::read(output_dir.join("models/real.onnx")).expect("read packaged model"),
        b"real"
    );
    assert!(
        !output_dir.join("models/cached.onnx").exists(),
        "generated/cache model must not be packaged by default"
    );
}

#[test]
fn package_convert_project_skips_generated_tokenizer_config_directories() {
    let temp = TempDir::new("package-convert-project-skip-tokenizer-generated");
    let project = temp.path().join("python-project");
    std::fs::create_dir_all(project.join(".venv/cache")).expect("create cache");
    std::fs::create_dir_all(project.join("export")).expect("create export");
    let model = project.join("export/real.onnx");
    std::fs::write(&model, b"real").expect("write real model");
    std::fs::write(
        project.join(".venv/cache/tokenizer_config.json"),
        r#"{"profile":"protein-20","add_special_tokens":false}"#,
    )
    .expect("write cached tokenizer config");
    std::fs::write(
        project.join("export/tokenizer_config.json"),
        r#"{"profile":"protein-20-special","add_special_tokens":true}"#,
    )
    .expect("write exported tokenizer config");
    let fixture_input = temp.write("tiny.fasta", ">tiny\nACDE\n");
    let fixture_output = temp.write("tiny.output.json", r#"{"label":"fixture","score":1.0}"#);
    let output_dir = temp.path().join("package");

    let output = Command::new(env!("CARGO_BIN_EXE_biors"))
        .arg("package")
        .arg("convert-project")
        .arg(&project)
        .arg("--model")
        .arg(&model)
        .arg("--output")
        .arg(&output_dir)
        .arg("--name")
        .arg("protein-project")
        .args(package_support::skeleton_metadata_args())
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
    let manifest: Value = serde_json::from_slice(
        &std::fs::read(output_dir.join("manifest.json")).expect("read generated manifest"),
    )
    .expect("manifest JSON");
    assert_eq!(manifest["tokenizer"]["name"], "protein-20-special");
    assert!(output_dir
        .join("tokenizers/protein-20-special.json")
        .exists());
    assert!(!output_dir.join("tokenizers/protein-20.json").exists());
}

#[test]
fn package_convert_project_accepts_explicit_tokenizer_config_override() {
    let temp = TempDir::new("package-convert-project-tokenizer-override");
    let project = temp.path().join("python-project");
    std::fs::create_dir_all(project.join("export-a")).expect("create export a");
    std::fs::create_dir_all(project.join("export-b")).expect("create export b");
    let model = project.join("model.onnx");
    std::fs::write(&model, b"model").expect("write model");
    std::fs::write(
        project.join("export-a/tokenizer_config.json"),
        r#"{"profile":"protein-20","add_special_tokens":false}"#,
    )
    .expect("write tokenizer config a");
    let intended_tokenizer = project.join("export-b/tokenizer_config.json");
    std::fs::write(
        &intended_tokenizer,
        r#"{"profile":"protein-20-special","add_special_tokens":true}"#,
    )
    .expect("write tokenizer config b");
    let fixture_input = temp.write("tiny.fasta", ">tiny\nACDE\n");
    let fixture_output = temp.write("tiny.output.json", r#"{"label":"fixture","score":1.0}"#);
    let output_dir = temp.path().join("package");

    let output = Command::new(env!("CARGO_BIN_EXE_biors"))
        .arg("package")
        .arg("convert-project")
        .arg(&project)
        .arg("--model")
        .arg(&model)
        .arg("--tokenizer-config")
        .arg(&intended_tokenizer)
        .arg("--output")
        .arg(&output_dir)
        .arg("--name")
        .arg("protein-project")
        .args(package_support::skeleton_metadata_args())
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
    let manifest: Value = serde_json::from_slice(
        &std::fs::read(output_dir.join("manifest.json")).expect("read generated manifest"),
    )
    .expect("manifest JSON");
    assert_eq!(manifest["tokenizer"]["name"], "protein-20-special");
    assert!(output_dir
        .join("tokenizers/protein-20-special.json")
        .exists());
}

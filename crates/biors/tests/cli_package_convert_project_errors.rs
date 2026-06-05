use serde_json::Value;
use std::process::Command;

mod common;
mod package_support;
use common::TempDir;

#[test]
fn package_convert_project_rejects_ambiguous_model_candidates() {
    let temp = TempDir::new("package-convert-project-ambiguous");
    let project = temp.path().join("python-project");
    std::fs::create_dir_all(project.join("exports")).expect("create exports");
    std::fs::write(project.join("exports/a.onnx"), b"a").expect("write model a");
    std::fs::write(project.join("exports/b.onnx"), b"b").expect("write model b");
    let fixture_input = temp.write("tiny.fasta", ">tiny\nACDE\n");
    let fixture_output = temp.write("tiny.output.json", r#"{"label":"fixture","score":1.0}"#);

    let output = Command::new(env!("CARGO_BIN_EXE_biors"))
        .arg("--json")
        .arg("package")
        .arg("convert-project")
        .arg(&project)
        .arg("--output")
        .arg(temp.path().join("package"))
        .arg("--name")
        .arg("protein-project")
        .args(package_support::skeleton_metadata_args())
        .arg("--fixture-input")
        .arg(&fixture_input)
        .arg("--fixture-output")
        .arg(&fixture_output)
        .output()
        .expect("run biors package convert-project");

    assert_eq!(output.status.code(), Some(2));
    assert!(output.stderr.is_empty());
    let value: Value = serde_json::from_slice(&output.stdout).expect("valid JSON error");
    assert_eq!(value["error"]["code"], "package.project_model_ambiguous");
    let candidates = value["error"]["details"]["candidates"]
        .as_array()
        .expect("candidate list");
    assert_eq!(candidates.len(), 2);
    assert!(candidates[0]
        .as_str()
        .expect("candidate")
        .ends_with("a.onnx"));
    assert!(candidates[1]
        .as_str()
        .expect("candidate")
        .ends_with("b.onnx"));
}

#[test]
fn package_convert_project_rejects_ambiguous_tokenizer_config_candidates() {
    let temp = TempDir::new("package-convert-project-ambiguous-tokenizer");
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
    std::fs::write(
        project.join("export-b/tokenizer_config.json"),
        r#"{"profile":"protein-20-special","add_special_tokens":true}"#,
    )
    .expect("write tokenizer config b");
    let fixture_input = temp.write("tiny.fasta", ">tiny\nACDE\n");
    let fixture_output = temp.write("tiny.output.json", r#"{"label":"fixture","score":1.0}"#);

    let output = Command::new(env!("CARGO_BIN_EXE_biors"))
        .arg("--json")
        .arg("package")
        .arg("convert-project")
        .arg(&project)
        .arg("--model")
        .arg(&model)
        .arg("--output")
        .arg(temp.path().join("package"))
        .arg("--name")
        .arg("protein-project")
        .args(package_support::skeleton_metadata_args())
        .arg("--fixture-input")
        .arg(&fixture_input)
        .arg("--fixture-output")
        .arg(&fixture_output)
        .output()
        .expect("run biors package convert-project");

    assert_eq!(output.status.code(), Some(2));
    assert!(output.stderr.is_empty());
    let value: Value = serde_json::from_slice(&output.stdout).expect("valid JSON error");
    assert_eq!(
        value["error"]["code"],
        "package.project_tokenizer_config_ambiguous"
    );
    let candidates = value["error"]["details"]["candidates"]
        .as_array()
        .expect("candidate list");
    assert_eq!(candidates.len(), 2);
    assert!(candidates[0]
        .as_str()
        .expect("candidate")
        .ends_with("export-a/tokenizer_config.json"));
    assert!(candidates[1]
        .as_str()
        .expect("candidate")
        .ends_with("export-b/tokenizer_config.json"));
}

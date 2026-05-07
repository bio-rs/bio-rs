use serde_json::Value;
use std::fs;
use std::process::Command;

mod common;
use common::TempDir;

#[test]
fn dataset_inspect_resolves_files_directories_and_globs() {
    let temp = TempDir::new("biors-dataset-inspect");
    let root = temp.write("root.fasta", ">root\nACGN\n");
    fs::create_dir_all(temp.path().join("nested")).expect("create nested dir");
    let nested = temp.path().join("nested/protein.faa");
    fs::write(&nested, ">protein\nMEEPQSDPSV\n").expect("write nested FASTA");
    fs::write(temp.path().join("notes.txt"), "not fasta\n").expect("write notes");

    let glob = temp.path().join("*.fasta").display().to_string();
    let output = Command::new(env!("CARGO_BIN_EXE_biors"))
        .arg("dataset")
        .arg("inspect")
        .arg(temp.path())
        .arg(glob)
        .output()
        .expect("run biors dataset inspect");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stderr.is_empty());

    let value: Value = serde_json::from_slice(&output.stdout).expect("valid JSON output");
    assert_eq!(value["data"]["provided_inputs"], 2);
    assert_eq!(value["data"]["files"], 2);
    assert_eq!(
        value["data"]["total_bytes"].as_u64().expect("total bytes"),
        fs::metadata(&root).expect("root metadata").len()
            + fs::metadata(&nested).expect("nested metadata").len()
    );
    let resolved_paths: Vec<_> = value["data"]["resolved_files"]
        .as_array()
        .expect("resolved files")
        .iter()
        .map(|file| file["path"].as_str().expect("path").to_string())
        .collect();
    assert!(resolved_paths.contains(&root.display().to_string()));
    assert!(resolved_paths.contains(&nested.display().to_string()));
}

#[test]
fn dataset_inspect_reports_empty_input_sets() {
    let temp = TempDir::new("biors-dataset-empty");
    fs::write(temp.path().join("notes.txt"), "not fasta\n").expect("write notes");

    let output = Command::new(env!("CARGO_BIN_EXE_biors"))
        .arg("--json")
        .arg("dataset")
        .arg("inspect")
        .arg(temp.path())
        .output()
        .expect("run empty dataset inspect");

    assert_eq!(output.status.code(), Some(2));
    assert!(output.stderr.is_empty());

    let value: Value = serde_json::from_slice(&output.stdout).expect("valid JSON error");
    assert_eq!(value["error"]["code"], "dataset.no_inputs");
}

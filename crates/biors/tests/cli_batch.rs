use serde_json::Value;
use std::fs;
use std::process::Command;

mod common;
use common::TempDir;

#[test]
fn batch_validate_accepts_multiple_input_files() {
    let temp = TempDir::new("biors-batch-files");
    let dna = temp.write("dna.fasta", ">dna\nACGN\n");
    let protein = temp.write("protein.fasta", ">protein\nMEEPQSDPSV\n");

    let output = Command::new(env!("CARGO_BIN_EXE_biors"))
        .arg("batch")
        .arg("validate")
        .arg("--kind")
        .arg("auto")
        .arg(&dna)
        .arg(&protein)
        .output()
        .expect("run biors batch validate");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stderr.is_empty());

    let value: Value = serde_json::from_slice(&output.stdout).expect("valid JSON output");
    assert_eq!(value["data"]["inputs"], 2);
    assert_eq!(value["data"]["summary"]["files"], 2);
    assert_eq!(value["data"]["summary"]["records"], 2);
    assert_eq!(value["data"]["summary"]["kind_counts"]["dna"], 1);
    assert_eq!(value["data"]["summary"]["kind_counts"]["protein"], 1);
    assert_eq!(value["data"]["files"][0]["records"], 1);
    assert_eq!(value["data"]["files"][0]["path"], dna.display().to_string());
    assert!(value["data"]["files"][0]["input_hash"]
        .as_str()
        .expect("input hash")
        .starts_with("fnv1a64:"));
}

#[test]
fn batch_validate_expands_directories_and_quoted_globs() {
    let temp = TempDir::new("biors-batch-globs");
    temp.write("a.fasta", ">a\nACGN\n");
    temp.write("b.fasta", ">b\nACGU\n");
    temp.write("notes.txt", "not fasta\n");

    let dir_output = Command::new(env!("CARGO_BIN_EXE_biors"))
        .arg("batch")
        .arg("validate")
        .arg("--kind")
        .arg("auto")
        .arg(temp.path())
        .output()
        .expect("run biors batch validate dir");

    assert!(dir_output.status.success());
    let dir_value: Value = serde_json::from_slice(&dir_output.stdout).expect("valid JSON output");
    assert_eq!(dir_value["data"]["summary"]["files"], 2);
    assert_eq!(dir_value["data"]["summary"]["records"], 2);

    let glob = temp.path().join("*.fasta").display().to_string();
    let glob_output = Command::new(env!("CARGO_BIN_EXE_biors"))
        .arg("batch")
        .arg("validate")
        .arg("--kind")
        .arg("auto")
        .arg(glob)
        .output()
        .expect("run biors batch validate glob");

    assert!(glob_output.status.success());
    let glob_value: Value = serde_json::from_slice(&glob_output.stdout).expect("valid JSON output");
    assert_eq!(glob_value["data"]["summary"], dir_value["data"]["summary"]);
    assert_eq!(
        glob_value["data"]["files"][0]["path"],
        dir_value["data"]["files"][0]["path"]
    );
}

#[test]
fn batch_validate_recurses_directories_and_reports_empty_globs() {
    let temp = TempDir::new("biors-batch-recursive");
    temp.write("root.fasta", ">root\nACGN\n");
    fs::create_dir_all(temp.path().join("nested/deeper")).expect("create nested dir");
    fs::write(
        temp.path().join("nested/deeper/protein.faa"),
        ">protein\nMEEPQSDPSV\n",
    )
    .expect("write nested FASTA");
    fs::write(temp.path().join("nested/notes.txt"), "not fasta\n").expect("write notes");

    let dir_output = Command::new(env!("CARGO_BIN_EXE_biors"))
        .arg("batch")
        .arg("validate")
        .arg("--kind")
        .arg("auto")
        .arg(temp.path())
        .output()
        .expect("run recursive biors batch validate");

    assert!(
        dir_output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&dir_output.stderr)
    );
    let dir_value: Value = serde_json::from_slice(&dir_output.stdout).expect("valid JSON output");
    assert_eq!(dir_value["data"]["summary"]["files"], 2);
    assert_eq!(dir_value["data"]["summary"]["records"], 2);

    let missing_glob = temp.path().join("*.missing").display().to_string();
    let glob_output = Command::new(env!("CARGO_BIN_EXE_biors"))
        .arg("--json")
        .arg("batch")
        .arg("validate")
        .arg("--kind")
        .arg("auto")
        .arg(missing_glob)
        .output()
        .expect("run empty-glob biors batch validate");

    assert_eq!(glob_output.status.code(), Some(2));
    assert!(glob_output.stderr.is_empty());
    let error: Value = serde_json::from_slice(&glob_output.stdout).expect("valid JSON error");
    assert_eq!(error["error"]["code"], "batch.no_inputs");
}

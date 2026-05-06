use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

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

struct TempDir {
    path: PathBuf,
}

impl TempDir {
    fn new(name: &str) -> Self {
        let path = std::env::temp_dir().join(format!(
            "{name}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system time")
                .as_nanos()
        ));
        fs::create_dir_all(&path).expect("create temp dir");
        Self { path }
    }

    fn path(&self) -> &Path {
        &self.path
    }

    fn write(&self, name: &str, contents: &str) -> PathBuf {
        let path = self.path.join(name);
        fs::write(&path, contents).expect("write temp FASTA");
        path
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

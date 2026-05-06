use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

mod common;
use common::ChildInputExt;

#[test]
fn diff_reports_canonical_json_matches_and_mismatches() {
    let temp = TempDir::new("biors-diff");
    let expected = temp.write("expected.json", r#"{"tokens":[1,2],"id":"seq1"}"#);
    let reordered = temp.write("reordered.json", r#"{"id":"seq1","tokens":[1,2]}"#);
    let mismatch = temp.write("mismatch.json", r#"{"id":"seq1","tokens":[1,3]}"#);

    let matching = run_biors(&["diff"], &[&expected, &reordered]);
    assert_eq!(matching["data"]["matches"], true);
    assert!(matching["data"]["expected_sha256"]
        .as_str()
        .expect("expected hash")
        .starts_with("sha256:"));
    assert_eq!(matching["data"]["content_diff"], Value::Null);

    let different = run_biors(&["diff"], &[&expected, &mismatch]);
    assert_eq!(different["data"]["matches"], false);
    assert_ne!(
        different["data"]["expected_sha256"],
        different["data"]["observed_sha256"]
    );
    assert_eq!(
        different["data"]["content_diff"]["expected_path"],
        expected.display().to_string()
    );
    assert!(
        different["data"]["content_diff"]["first_difference"]["byte_offset"]
            .as_u64()
            .is_some()
    );
}

#[test]
fn pipeline_outputs_validate_tokenize_export_chain_without_config() {
    let output = Command::new(env!("CARGO_BIN_EXE_biors"))
        .arg("pipeline")
        .arg("--max-length")
        .arg("6")
        .arg("-")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn biors pipeline")
        .tap_stdin(">seq1\nACDE\n");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stderr.is_empty());
    let value: Value = serde_json::from_slice(&output.stdout).expect("valid JSON output");
    assert_eq!(value["data"]["pipeline"], "validate_tokenize_export.v0");
    assert_eq!(value["data"]["ready"], true);
    assert_eq!(value["data"]["steps"][0]["name"], "validate");
    assert_eq!(value["data"]["steps"][1]["name"], "tokenize");
    assert_eq!(value["data"]["steps"][2]["name"], "export");
    assert_eq!(value["data"]["steps"][2]["status"], "passed");
    assert_eq!(
        value["data"]["workflow"]["model_input"]["records"][0]["input_ids"],
        serde_json::json!([0, 1, 2, 3, 0, 0])
    );
}

#[test]
fn debug_outputs_step_by_step_tokens_model_input_and_error_visualization() {
    let output = Command::new(env!("CARGO_BIN_EXE_biors"))
        .arg("debug")
        .arg("--max-length")
        .arg("6")
        .arg("-")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn biors debug")
        .tap_stdin(">bad\nAX*\n");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stderr.is_empty());
    let value: Value = serde_json::from_slice(&output.stdout).expect("valid JSON output");
    let record = &value["data"]["records"][0];

    assert_eq!(value["data"]["view"], "sequence_debug.v0");
    assert_eq!(record["id"], "bad");
    assert_eq!(record["normalized_sequence"], "AX*");
    assert_eq!(record["token_map"][0]["status"], "standard");
    assert_eq!(record["token_map"][1]["status"], "warning");
    assert_eq!(record["token_map"][2]["status"], "error");
    assert_eq!(record["model_input"], Value::Null);
    assert!(record["error_visualization"]["markers"]
        .as_str()
        .expect("markers")
        .contains('E'));
}

fn run_biors(args: &[&str], paths: &[&Path]) -> Value {
    let output = Command::new(env!("CARGO_BIN_EXE_biors"))
        .args(args)
        .args(paths)
        .output()
        .expect("run biors command");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).expect("valid JSON")
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

    fn write(&self, name: &str, contents: &str) -> PathBuf {
        let path = self.path.join(name);
        fs::write(&path, contents).expect("write temp file");
        path
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

#![allow(dead_code)]

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use jsonschema::JSONSchema;
use serde_json::Value;

pub struct TempDir {
    path: PathBuf,
}

impl TempDir {
    pub fn new(name: &str) -> Self {
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

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn write(&self, name: &str, contents: &str) -> PathBuf {
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

pub fn spawn_biors(args: &[&str]) -> std::process::Child {
    Command::new(env!("CARGO_BIN_EXE_biors"))
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn biors")
}

pub fn run_biors(args: &[&str]) -> std::process::Output {
    let output = Command::new(env!("CARGO_BIN_EXE_biors"))
        .args(args)
        .output()
        .expect("run biors command");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    output
}

pub fn run_biors_stdin(args: &[&str], input: &str) -> std::process::Output {
    let output = spawn_biors(args).tap_stdin(input);
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stderr.is_empty());
    output
}

pub fn run_biors_paths(args: &[&str], paths: &[&Path]) -> std::process::Output {
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
    output
}

pub fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..")
}

pub fn assert_payload_matches_schema(output: &[u8], schema_path: &str) {
    let envelope: Value = serde_json::from_slice(output).expect("valid CLI JSON");
    assert_json_value_matches_schema(&envelope, "schemas/cli-success.v0.json");
    assert_json_value_matches_schema(&envelope["data"], schema_path);
}

pub fn assert_json_matches_schema(output: &[u8], schema_path: &str) {
    let value: Value = serde_json::from_slice(output).expect("valid CLI JSON");
    assert_json_value_matches_schema(&value, schema_path);
}

pub fn assert_json_value_matches_schema(value: &Value, schema_path: &str) {
    let schema: Value = serde_json::from_str(
        &fs::read_to_string(repo_root().join(schema_path)).expect("read payload schema"),
    )
    .expect("schema JSON");
    let compiled = JSONSchema::compile(&schema).expect("compile schema");
    let validation = compiled.validate(value);
    if let Err(errors) = validation {
        let messages: Vec<_> = errors.map(|error| error.to_string()).collect();
        panic!("JSON did not match schema {schema_path}: {messages:?}");
    }
}

pub fn assert_payload_rejected_by_schema(payload: &Value, schema_path: &str) {
    let schema: Value = serde_json::from_str(
        &fs::read_to_string(repo_root().join(schema_path)).expect("read payload schema"),
    )
    .expect("schema JSON");
    let compiled = JSONSchema::compile(&schema).expect("compile schema");

    assert!(
        compiled.validate(payload).is_err(),
        "payload unexpectedly matched schema {schema_path}"
    );
}

pub fn run_biors_stdin_expect_failure(args: &[&str], input: &str) -> std::process::Output {
    let output = spawn_biors(args).tap_stdin(input);
    assert_eq!(output.status.code(), Some(2));
    assert!(output.stderr.is_empty());
    output
}

pub trait ChildInputExt {
    fn tap_stdin(self, input: &str) -> std::process::Output;
}

impl ChildInputExt for std::process::Child {
    fn tap_stdin(mut self, input: &str) -> std::process::Output {
        self.stdin
            .as_mut()
            .expect("stdin pipe")
            .write_all(input.as_bytes())
            .expect("write stdin");

        self.wait_with_output().expect("wait for biors")
    }
}

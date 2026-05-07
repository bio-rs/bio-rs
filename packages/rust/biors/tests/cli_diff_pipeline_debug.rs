use serde_json::Value;
use std::fs;
use std::path::Path;

mod common;
use common::{ChildInputExt, TempDir};

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
    let output =
        common::spawn_biors(&["pipeline", "--max-length", "6", "-"]).tap_stdin(">seq1\nACDE\n");

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
fn pipeline_runs_toml_config_with_explain_plan() {
    let config = repo_root().join("examples/pipeline/protein.toml");
    let config_arg = config.to_string_lossy();

    let value = run_biors(
        &["pipeline", "--config", &config_arg, "--explain-plan"],
        &[],
    );

    assert_eq!(value["data"]["pipeline"], "config_pipeline.v0");
    assert_eq!(value["data"]["config"]["name"], "protein-fixture-pipeline");
    assert_eq!(value["data"]["dry_run"], false);
    assert_eq!(value["data"]["explain_plan"], true);
    assert_eq!(
        value["data"]["steps"]
            .as_array()
            .expect("steps")
            .iter()
            .map(|step| step["name"].as_str().expect("step name"))
            .collect::<Vec<_>>(),
        vec!["parse", "normalize", "validate", "tokenize", "export"]
    );
    assert_eq!(
        value["data"]["plan"]["stages"][0]["operation"],
        "parse FASTA input"
    );
    assert_eq!(
        value["data"]["workflow"]["model_input"]["records"][0]["input_ids"],
        serde_json::json!([0, 1, 2, 3, 0, 0, 0, 0])
    );
}

#[test]
fn pipeline_dry_run_accepts_yaml_config_without_reading_input() {
    let temp = TempDir::new("biors-pipeline-yaml");
    let config = temp.write(
        "pipeline.yaml",
        r#"schema_version: biors.pipeline.v0
name: dry-run-only
input:
  format: fasta
  path: missing/input.fasta
normalize:
  policy: strip_ascii_whitespace_uppercase
validate:
  kind: protein
tokenize:
  profile: protein-20
export:
  format: model-input-json
  max_length: 8
"#,
    );
    let config_arg = config.to_string_lossy();

    let value = run_biors(&["pipeline", "--config", &config_arg, "--dry-run"], &[]);

    assert_eq!(value["data"]["pipeline"], "config_pipeline.v0");
    assert_eq!(value["data"]["dry_run"], true);
    assert_eq!(value["data"]["workflow"], Value::Null);
    assert!(value["data"]["steps"]
        .as_array()
        .expect("steps")
        .iter()
        .all(|step| step["status"] == "planned"));
}

#[test]
fn pipeline_runs_json_config() {
    let config = repo_root().join("examples/pipeline/protein.json");
    let config_arg = config.to_string_lossy();

    let value = run_biors(&["pipeline", "--config", &config_arg], &[]);

    assert_eq!(value["data"]["pipeline"], "config_pipeline.v0");
    assert_eq!(
        value["data"]["config"]["schema_version"],
        "biors.pipeline.v0"
    );
    assert_eq!(value["data"]["ready"], true);
    assert!(value["data"]["plan"].is_null());
}

#[test]
fn pipeline_writes_lockfile_with_package_provenance() {
    let temp = TempDir::new("biors-pipeline-lock");
    let lockfile = temp.path().join("pipeline.lock");
    let repo = repo_root();
    let config = repo.join("examples/protein-package/pipelines/protein.toml");
    let package = repo.join("examples/protein-package/manifest.json");
    let config_arg = config.to_string_lossy();
    let package_arg = package.to_string_lossy();
    let lock_arg = lockfile.to_string_lossy();

    let value = run_biors(
        &[
            "pipeline",
            "--config",
            &config_arg,
            "--package",
            &package_arg,
            "--write-lock",
            &lock_arg,
        ],
        &[],
    );

    assert_eq!(value["data"]["pipeline"], "config_pipeline.v0");
    assert!(lockfile.exists(), "pipeline.lock was not written");

    let lock: Value = serde_json::from_str(&fs::read_to_string(&lockfile).expect("read lockfile"))
        .expect("valid lockfile JSON");

    assert_eq!(lock["schema_version"], "biors.pipeline.lock.v0");
    assert_eq!(
        lock["pipeline_config"]["schema_version"],
        "biors.pipeline.v0"
    );
    assert_eq!(lock["package"]["name"], "protein-seed");
    assert_eq!(
        lock["package"]["model_sha256"],
        "sha256:2c1da72b15fab35bd6f1bb62f5037b936e26e6413a220fa9afe5a64bce0df68d"
    );
    assert_eq!(lock["package"]["runtime_backend"], "onnx-webgpu");
    assert_eq!(lock["package"]["backend_version"], "onnx-webgpu.v0");
    assert!(lock["hashes"]["vocabulary_sha256"]
        .as_str()
        .expect("vocab hash")
        .starts_with("sha256:"));
    assert!(lock["execution"]["input_hash"]
        .as_str()
        .expect("input hash")
        .starts_with("fnv1a64:"));
    assert_eq!(
        lock["python_baseline"]["comparison"],
        "normalized_records_and_protein20_tokens"
    );
}

#[test]
fn debug_outputs_step_by_step_tokens_model_input_and_error_visualization() {
    let output = common::spawn_biors(&["debug", "--max-length", "6", "-"]).tap_stdin(">bad\nAX*\n");

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
    let output = common::run_biors_paths(args, paths);
    serde_json::from_slice(&output.stdout).expect("valid JSON")
}

fn repo_root() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..")
}

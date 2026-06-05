use serde_json::Value;
use std::process::Command;

mod cli_pipeline_support;
mod common;

use cli_pipeline_support::{repo_root, run_biors};
use common::{ChildInputExt, TempDir};

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
fn pipeline_config_rejects_yaml_by_default() {
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

    let output = Command::new(env!("CARGO_BIN_EXE_biors"))
        .arg("--json")
        .arg("pipeline")
        .arg("--config")
        .arg(config_arg.as_ref())
        .arg("--dry-run")
        .output()
        .expect("run biors pipeline");

    assert_eq!(output.status.code(), Some(2));
    assert!(output.stderr.is_empty());
    let value: Value = serde_json::from_slice(&output.stdout).expect("valid JSON error");
    assert_eq!(value["error"]["code"], "pipeline.invalid_config");
    assert!(value["error"]["message"]
        .as_str()
        .expect("error message")
        .contains("unsupported pipeline config extension: yaml"));
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
fn pipeline_config_accepts_nucleotide_profiles() {
    let temp = TempDir::new("biors-pipeline-nucleotide");
    let fasta = temp.write("dna.fasta", ">dna\nACGT\n");
    let config = temp.write(
        "dna.toml",
        &format!(
            r#"schema_version = "biors.pipeline.v0"
name = "dna-pipeline"

[input]
format = "fasta"
path = "{}"

[normalize]
policy = "strip_ascii_whitespace_uppercase"

[validate]
kind = "dna"

[tokenize]
profile = "dna-iupac"

[export]
format = "model-input-json"
max_length = 6
pad_token_id = 0
padding = "fixed_length"
"#,
            fasta.file_name().expect("fasta filename").to_string_lossy()
        ),
    );
    let config_arg = config.to_string_lossy();

    let value = run_biors(&["pipeline", "--config", &config_arg], &[]);

    assert_eq!(value["data"]["ready"], true);
    assert_eq!(value["data"]["config"]["name"], "dna-pipeline");
    assert_eq!(
        value["data"]["workflow"]["provenance"]["validation_alphabet"],
        "dna-iupac"
    );
    assert_eq!(
        value["data"]["workflow"]["provenance"]["tokenizer"]["name"],
        "dna-iupac"
    );
    assert_eq!(
        value["data"]["workflow"]["model_input"]["records"][0]["input_ids"],
        serde_json::json!([0, 1, 2, 3, 0, 0])
    );
}

#[test]
fn pipeline_config_rejects_kind_profile_mismatch() {
    let temp = TempDir::new("biors-pipeline-profile-mismatch");
    temp.write("dna.fasta", ">dna\nACGT\n");
    let config = temp.write(
        "mismatch.toml",
        r#"schema_version = "biors.pipeline.v0"
name = "mismatch"

[input]
format = "fasta"
path = "dna.fasta"

[normalize]
policy = "strip_ascii_whitespace_uppercase"

[validate]
kind = "protein"

[tokenize]
profile = "dna-iupac"

[export]
format = "model-input-json"
max_length = 6
"#,
    );

    let output = Command::new(env!("CARGO_BIN_EXE_biors"))
        .arg("--json")
        .arg("pipeline")
        .arg("--config")
        .arg(config)
        .output()
        .expect("run biors pipeline");

    assert_eq!(output.status.code(), Some(2));
    let value: Value = serde_json::from_slice(&output.stdout).expect("valid JSON error");
    assert_eq!(value["error"]["code"], "pipeline.invalid_config");
    assert_eq!(value["error"]["location"], "validate.kind");
    assert!(value["error"]["message"]
        .as_str()
        .expect("error message")
        .contains("validate.kind must be 'dna'"));
}

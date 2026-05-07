use jsonschema::JSONSchema;
use serde_json::Value;
use std::fs;
use std::path::Path;

mod common;

#[test]
fn machine_readable_schemas_are_valid_json() {
    let repo = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
    for schema in [
        "schemas/cli-success.v0.json",
        "schemas/cli-error.v0.json",
        "schemas/tokenize-output.v0.json",
        "schemas/inspect-output.v0.json",
        "schemas/model-input-output.v0.json",
        "schemas/batch-validation-output.v0.json",
        "schemas/doctor-output.v0.json",
        "schemas/output-diff.v0.json",
        "schemas/pipeline-output.v0.json",
        "schemas/pipeline-config.v0.json",
        "schemas/sequence-debug-output.v0.json",
        "schemas/tokenizer-inspect-output.v0.json",
        "schemas/sequence-workflow-output.v0.json",
        "schemas/fasta-validation-output.v0.json",
        "schemas/package-inspect-output.v0.json",
        "schemas/package-bridge-output.v0.json",
        "schemas/package-verify-output.v0.json",
        "schemas/package-manifest.v0.json",
        "schemas/package-manifest.v1.json",
        "schemas/package-validation-report.v0.json",
    ] {
        let input = fs::read_to_string(repo.join(schema)).expect("read schema");
        let value: Value = serde_json::from_str(&input).expect("schema is valid JSON");

        assert_eq!(
            value["$schema"],
            "https://json-schema.org/draft/2020-12/schema"
        );
        assert!(value["$id"].as_str().expect("schema id").contains("bio-rs"));
        assert!(matches!(
            value["type"].as_str(),
            Some("object") | Some("array")
        ));
    }
}

#[test]
fn package_manifest_example_uses_declared_schema_version() {
    let repo = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
    let manifest: Value = serde_json::from_str(
        &fs::read_to_string(repo.join("examples/protein-package/manifest.json"))
            .expect("read package manifest"),
    )
    .expect("manifest JSON");

    assert_eq!(manifest["schema_version"], "biors.package.v1");
    assert_eq!(manifest["package_layout"]["models"], "models");
    assert_eq!(manifest["package_layout"]["docs"], "docs");
    assert_eq!(manifest["metadata"]["license"]["expression"], "CC0-1.0");
    assert_eq!(
        manifest["metadata"]["model_card"]["path"],
        "docs/model-card.md"
    );
    assert!(manifest["model"]["checksum"].is_string());
    assert!(manifest["tokenizer"]["checksum"].is_string());
    assert!(manifest["vocab"]["checksum"].is_string());
    assert!(manifest["metadata"]["license"]["file"]["checksum"].is_string());
    assert!(manifest["metadata"]["citation"]["file"]["checksum"].is_string());
    assert!(manifest["metadata"]["model_card"]["checksum"].is_string());
    assert!(manifest["expected_input"]["dtype"].is_string());
    assert!(manifest["fixtures"][0]["input_hash"]
        .as_str()
        .expect("fixture input hash")
        .starts_with("sha256:"));
    assert!(manifest["fixtures"][0]["expected_output_hash"]
        .as_str()
        .expect("fixture output hash")
        .starts_with("sha256:"));
    assert_json_value_matches_schema(&manifest, "schemas/package-manifest.v1.json");
}

#[test]
fn cli_outputs_match_sequence_schemas() {
    let tokenize = common::run_biors_stdin(&["tokenize", "-"], ">seq1\nACDE\n").stdout;
    assert_payload_matches_schema(&tokenize, "schemas/tokenize-output.v0.json");

    let special_config =
        repo_root().join("examples/model-input-contract/protein-20-special.config.json");
    let special_fasta = repo_root().join("examples/model-input-contract/protein.fasta");
    let special_tokenize = common::run_biors_paths(
        &["tokenize", "--config"],
        &[&special_config, &special_fasta],
    )
    .stdout;
    assert_payload_matches_schema(&special_tokenize, "schemas/tokenize-output.v0.json");

    let inspect = common::run_biors_stdin(&["inspect", "-"], ">seq1\nACDE\n>seq2\nAX\n").stdout;
    assert_payload_matches_schema(&inspect, "schemas/inspect-output.v0.json");

    let fasta_validate =
        common::run_biors_stdin(&["fasta", "validate", "-"], ">seq1\nAX*\n").stdout;
    assert_payload_matches_schema(&fasta_validate, "schemas/fasta-validation-output.v0.json");

    let seq_validate = common::run_biors_stdin(&["seq", "validate", "-"], ">seq1\nACGN\n").stdout;
    assert_payload_matches_schema(&seq_validate, "schemas/fasta-validation-output.v0.json");

    let model_input = common::run_biors_stdin(
        &["model-input", "--max-length", "4", "-"],
        ">seq1\nACDEFG\n",
    )
    .stdout;
    assert_payload_matches_schema(&model_input, "schemas/model-input-output.v0.json");

    let workflow =
        common::run_biors_stdin(&["workflow", "--max-length", "4", "-"], ">seq1\nACDEFG\n").stdout;
    assert_payload_matches_schema(&workflow, "schemas/sequence-workflow-output.v0.json");

    let pipeline =
        common::run_biors_stdin(&["pipeline", "--max-length", "4", "-"], ">seq1\nACDE\n").stdout;
    assert_payload_matches_schema(&pipeline, "schemas/pipeline-output.v0.json");

    let pipeline_config_path = repo_root().join("examples/pipeline/protein.toml");
    let pipeline_config_arg = pipeline_config_path.to_string_lossy();
    let pipeline_config = common::run_biors_paths(
        &[
            "pipeline",
            "--config",
            &pipeline_config_arg,
            "--explain-plan",
        ],
        &[],
    )
    .stdout;
    assert_payload_matches_schema(&pipeline_config, "schemas/pipeline-output.v0.json");

    let pipeline_config_json: Value = serde_json::from_str(
        &fs::read_to_string(repo_root().join("examples/pipeline/protein.json"))
            .expect("read pipeline JSON config"),
    )
    .expect("pipeline config JSON");
    assert_json_value_matches_schema(&pipeline_config_json, "schemas/pipeline-config.v0.json");

    let debug =
        common::run_biors_stdin(&["debug", "--max-length", "4", "-"], ">seq1\nAX*\n").stdout;
    assert_payload_matches_schema(&debug, "schemas/sequence-debug-output.v0.json");
}

#[test]
fn cli_outputs_match_diff_schema() {
    let expected = repo_root().join("examples/protein-package/fixtures/tiny.output.json");
    let observed = repo_root().join("examples/protein-package/observed/tiny.reordered.json");
    let diff = common::run_biors_paths(&["diff"], &[&expected, &observed]).stdout;
    assert_payload_matches_schema(&diff, "schemas/output-diff.v0.json");
}

#[test]
fn cli_outputs_match_batch_schema() {
    let examples = repo_root().join("examples");
    let batch_validate =
        common::run_biors_paths(&["batch", "validate", "--kind", "auto"], &[&examples]).stdout;
    assert_payload_matches_schema(&batch_validate, "schemas/batch-validation-output.v0.json");
}

#[test]
fn cli_outputs_match_tooling_schemas() {
    let tokenizer_inspect = common::run_biors_paths(
        &["tokenizer", "inspect", "--profile", "protein-20-special"],
        &[],
    )
    .stdout;
    assert_payload_matches_schema(
        &tokenizer_inspect,
        "schemas/tokenizer-inspect-output.v0.json",
    );

    let doctor = common::run_biors_paths(&["doctor"], &[]).stdout;
    assert_payload_matches_schema(&doctor, "schemas/doctor-output.v0.json");
}

#[test]
fn cli_outputs_match_package_schemas() {
    let manifest = repo_root().join("examples/protein-package/manifest.json");
    let observations = repo_root().join("examples/protein-package/observations.json");

    let package_inspect = common::run_biors_paths(&["package", "inspect"], &[&manifest]).stdout;
    assert_payload_matches_schema(&package_inspect, "schemas/package-inspect-output.v0.json");

    let package_validate = common::run_biors_paths(&["package", "validate"], &[&manifest]).stdout;
    assert_payload_matches_schema(
        &package_validate,
        "schemas/package-validation-report.v0.json",
    );

    let package_bridge = common::run_biors_paths(&["package", "bridge"], &[&manifest]).stdout;
    assert_payload_matches_schema(&package_bridge, "schemas/package-bridge-output.v0.json");

    let package_verify =
        common::run_biors_paths(&["package", "verify"], &[&manifest, &observations]).stdout;
    assert_payload_matches_schema(&package_verify, "schemas/package-verify-output.v0.json");
}

#[test]
fn cli_rejections_match_schemas() {
    let zero_model_input = serde_json::json!({
        "policy": {
            "max_length": 0,
            "pad_token_id": 0,
            "padding": "fixed_length"
        },
        "records": []
    });
    assert_payload_rejected_by_schema(&zero_model_input, "schemas/model-input-output.v0.json");

    let mismatch_report = serde_json::json!({
        "package": "protein-seed",
        "fixtures": 1,
        "passed": 0,
        "failed": 1,
        "results": [
            {
                "name": "tiny-protein",
                "input_path": "fixtures/tiny.fasta",
                "expected_output_path": "fixtures/tiny.output.json",
                "observed_output_path": "observed/tiny.bad.json",
                "expected_output_hash": "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
                "observed_output_hash": "sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
                "status": "failed",
                "checksum_mismatch": true,
                "content_mismatch": true,
                "issue_code": "output_content_mismatch",
                "content_diff": {
                    "expected_path": "fixtures/tiny.output.json",
                    "observed_path": "observed/tiny.bad.json",
                    "expected_len": 32,
                    "observed_len": 28,
                    "first_difference": {
                        "byte_offset": 10,
                        "expected_byte": 34,
                        "observed_byte": 48
                    }
                },
                "issue": "output content mismatch between 'fixtures/tiny.output.json' and 'observed/tiny.bad.json'"
            }
        ]
    });
    assert_json_value_matches_schema(&mismatch_report, "schemas/package-verify-output.v0.json");
}

#[test]
fn cli_outputs_match_success_and_error_envelope_schemas() {
    let success = common::run_biors_stdin(&["tokenize", "-"], ">seq1\nACDE\n").stdout;
    assert_json_matches_schema(&success, "schemas/cli-success.v0.json");

    let error =
        common::run_biors_stdin_expect_failure(&["--json", "tokenize", "-"], "ACDE\n").stdout;
    assert_json_matches_schema(&error, "schemas/cli-error.v0.json");
}

fn assert_payload_matches_schema(output: &[u8], schema_path: &str) {
    let envelope: Value = serde_json::from_slice(output).expect("valid CLI JSON");
    assert_json_value_matches_schema(&envelope, "schemas/cli-success.v0.json");
    assert_json_value_matches_schema(&envelope["data"], schema_path);
}

fn assert_json_matches_schema(output: &[u8], schema_path: &str) {
    let value: Value = serde_json::from_slice(output).expect("valid CLI JSON");
    assert_json_value_matches_schema(&value, schema_path);
}

fn assert_json_value_matches_schema(value: &Value, schema_path: &str) {
    let repo = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
    let schema: Value = serde_json::from_str(
        &fs::read_to_string(repo.join(schema_path)).expect("read payload schema"),
    )
    .expect("schema JSON");
    let compiled = JSONSchema::compile(&schema).expect("compile schema");
    let validation = compiled.validate(value);
    if let Err(errors) = validation {
        let messages: Vec<_> = errors.map(|error| error.to_string()).collect();
        panic!("JSON did not match schema {schema_path}: {messages:?}");
    }
}

fn repo_root() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..")
}

fn assert_payload_rejected_by_schema(payload: &Value, schema_path: &str) {
    let repo = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
    let schema: Value = serde_json::from_str(
        &fs::read_to_string(repo.join(schema_path)).expect("read payload schema"),
    )
    .expect("schema JSON");
    let compiled = JSONSchema::compile(&schema).expect("compile schema");

    assert!(
        compiled.validate(payload).is_err(),
        "payload unexpectedly matched schema {schema_path}"
    );
}

use serde_json::Value;
use std::fs;

mod common;

#[test]
fn machine_readable_schemas_are_valid_json() {
    for schema in [
        "schemas/cli-success.v0.json",
        "schemas/cli-error.v0.json",
        "schemas/tokenize-output.v0.json",
        "schemas/inspect-output.v0.json",
        "schemas/model-input-output.v0.json",
        "schemas/batch-validation-output.v0.json",
        "schemas/dataset-inspect-output.v0.json",
        "schemas/cache-output.v0.json",
        "schemas/doctor-output.v0.json",
        "schemas/output-diff.v0.json",
        "schemas/pipeline-output.v0.json",
        "schemas/pipeline-config.v0.json",
        "schemas/pipeline-lock.v0.json",
        "schemas/sequence-debug-output.v0.json",
        "schemas/tokenizer-inspect-output.v0.json",
        "schemas/tokenizer-conversion-output.v0.json",
        "schemas/sequence-workflow-output.v0.json",
        "schemas/fasta-validation-output.v0.json",
        "schemas/package-inspect-output.v0.json",
        "schemas/package-bridge-output.v0.json",
        "schemas/package-verify-output.v0.json",
        "schemas/package-conversion-output.v0.json",
        "schemas/package-skeleton-output.v0.json",
        "schemas/package-migration-output.v0.json",
        "schemas/package-compatibility-output.v0.json",
        "schemas/package-diff-output.v0.json",
        "schemas/service-interface-output.v0.json",
        "schemas/package-manifest.v0.json",
        "schemas/package-manifest.v1.json",
        "schemas/package-validation-report.v0.json",
    ] {
        let input = fs::read_to_string(common::repo_root().join(schema)).expect("read schema");
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
fn invalid_payload_examples_are_rejected_by_schemas() {
    let zero_model_input = serde_json::json!({
        "policy": {
            "max_length": 0,
            "pad_token_id": 0,
            "padding": "fixed_length"
        },
        "records": []
    });
    common::assert_payload_rejected_by_schema(
        &zero_model_input,
        "schemas/model-input-output.v0.json",
    );
}

#[test]
fn cli_outputs_match_success_and_error_envelope_schemas() {
    let success = common::run_biors_stdin(&["tokenize", "-"], ">seq1\nACDE\n").stdout;
    common::assert_json_matches_schema(&success, "schemas/cli-success.v0.json");

    let error =
        common::run_biors_stdin_expect_failure(&["--json", "tokenize", "-"], "ACDE\n").stdout;
    common::assert_json_matches_schema(&error, "schemas/cli-error.v0.json");

    let parse_error = common::run_biors_stdin_expect_failure(
        &[
            "--json",
            "workflow",
            "--max-length",
            "8",
            "--padding",
            "fixed_length",
            "-",
        ],
        ">seq1\nACDE\n",
    )
    .stdout;
    common::assert_json_matches_schema(&parse_error, "schemas/cli-error.v0.json");
}

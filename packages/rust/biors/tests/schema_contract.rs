use serde_json::Value;
use std::fs;

use biors_core::service::current_service_interface_document;

mod common;

#[test]
fn machine_readable_schemas_are_valid_json() {
    let schemas_dir = common::repo_root().join("schemas");

    for entry in fs::read_dir(&schemas_dir).expect("read schemas directory") {
        let entry = entry.expect("read schema entry");
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("json") {
            continue;
        }

        let input = fs::read_to_string(&path).expect("read schema");
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
        jsonschema::JSONSchema::compile(&value).expect("compile schema");
    }
}

#[test]
fn service_contract_references_checked_in_schemas() {
    let repo = common::repo_root();
    let document = current_service_interface_document();

    for route in document.routes {
        for schema in [&route.request_schema, &route.response_schema] {
            let path = repo.join("schemas").join(schema);
            assert!(
                path.exists(),
                "service route {} references missing schema {schema}",
                route.operation_id
            );
        }
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

    let non_binary_mask = serde_json::json!({
        "policy": {
            "max_length": 2,
            "pad_token_id": 0,
            "padding": "fixed_length"
        },
        "records": [{
            "id": "seq1",
            "input_ids": [1, 0],
            "attention_mask": [1, 2],
            "truncated": false
        }]
    });
    common::assert_payload_rejected_by_schema(
        &non_binary_mask,
        "schemas/model-input-output.v0.json",
    );

    let out_of_range_token = serde_json::json!([{
        "id": "seq1",
        "length": 1,
        "alphabet": "protein-20",
        "valid": true,
        "tokens": [256],
        "warnings": [],
        "errors": []
    }]);
    common::assert_payload_rejected_by_schema(
        &out_of_range_token,
        "schemas/tokenize-output.v0.json",
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

    let package_error = common::run_biors_stdin_expect_failure(
        &["--json", "package", "validate", "-"],
        r#"{
          "schema_version": "biors.package.v0",
          "name": "",
          "model": { "format": "onnx", "path": "" },
          "preprocessing": [],
          "postprocessing": [],
          "runtime": {
            "backend": "onnx-webgpu",
            "target": "browser-wasm-webgpu"
          },
          "fixtures": []
        }"#,
    )
    .stdout;
    common::assert_json_matches_schema(&package_error, "schemas/cli-error.v0.json");
}

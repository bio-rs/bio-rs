use serde_json::Value;
use std::fs;

use biors_core::conversion::convert_fasta_records;
use biors_core::sequence::SequenceKindSelection;
use biors_core::service::{
    current_hosted_workflow_boundary, current_service_interface_document, service_openapi_document,
};
use biors_core::templates::{find_task_template, task_templates};

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
fn hosted_boundary_output_matches_checked_in_schema() {
    let boundary = current_hosted_workflow_boundary();
    let value = serde_json::to_value(boundary).expect("serialize hosted boundary");

    common::assert_json_value_matches_schema(
        &value,
        "schemas/hosted-workflow-boundary-output.v0.json",
    );
}

#[test]
fn service_openapi_output_accepts_local_and_container_bind_urls() {
    for server_url in ["http://127.0.0.1:8787", "http://0.0.0.0:8787"] {
        let openapi = service_openapi_document("0.57.0", server_url);
        common::assert_json_value_matches_schema(
            &openapi,
            "schemas/service-openapi-output.v0.json",
        );
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

    let missing_conversion_payload = serde_json::json!({
        "schema_version": "biors.conversion.v0",
        "records": 1,
        "valid_records": 1,
        "model_ready_records": 1,
        "warning_count": 0,
        "error_count": 0,
        "entities": [{
            "id": "seq1",
            "entity_type": "sequence",
            "source": { "format": "fasta" },
            "record": {
                "type": "sequence",
                "data": {}
            },
            "validation": {
                "valid": true,
                "model_ready": true,
                "warning_count": 0,
                "error_count": 0,
                "warnings": [],
                "errors": []
            }
        }]
    });
    common::assert_payload_rejected_by_schema(
        &missing_conversion_payload,
        "schemas/bio-entity-export-output.v0.json",
    );

    let incomplete_browser_tooling = serde_json::json!({
        "schema_version": "biors.browser_tooling.v0",
        "file": {
            "format": "fasta",
            "size_bytes": 11,
            "content_sha256": "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
        },
        "warnings": []
    });
    common::assert_payload_rejected_by_schema(
        &incomplete_browser_tooling,
        "schemas/browser-tooling-output.v0.json",
    );
}

#[test]
fn browser_tooling_examples_match_checked_in_schema() {
    let browser_policy = serde_json::json!({
        "schema_version": "biors.browser_tooling.v0",
        "execution_mode": "wasm_local",
        "network_access": "none",
        "uploads_input_data": false,
        "external_model_calls": false,
        "persistence": "caller_controlled",
        "max_input_bytes": 67108864,
        "warning_input_bytes": 16777216,
        "streaming": {
            "supported": false,
            "behavior": "single Uint8Array input is validated before parsing",
            "caller_guidance": "slice or reject larger files before passing them to WASM"
        },
        "supported_validation_formats": ["fasta", "fastq", "pdb", "smiles"],
        "supported_tokenization_formats": ["fasta"]
    });
    common::assert_json_value_matches_schema(
        &browser_policy,
        "schemas/browser-tooling-output.v0.json",
    );

    let browser_tokenization = serde_json::json!({
        "schema_version": "biors.browser_tooling.v0",
        "file": {
            "name": "protein.fasta",
            "format": "fasta",
            "size_bytes": 11,
            "content_sha256": "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            "input_hash": "fnv1a64:08a331cb13c7bd72"
        },
        "tokenization": {
            "inputIds": [[0, 1, 2, 3]],
            "attentionMask": [[1, 1, 1, 1]],
            "ids": ["seq1"],
            "records": [{
                "id": "seq1",
                "alphabet": "protein-20",
                "valid": true,
                "tokens": [0, 1, 2, 3],
                "length": 4,
                "warnings": [],
                "errors": []
            }]
        },
        "model_input_policy_hint": {
            "max_length_required": true,
            "supported_padding": ["fixed_length", "no_padding"],
            "note": "pass tokenization.records to buildModelInputWithPolicy"
        },
        "warnings": []
    });
    common::assert_json_value_matches_schema(
        &browser_tokenization,
        "schemas/browser-tooling-output.v0.json",
    );
}

#[test]
fn conversion_and_template_outputs_match_contract_schemas() {
    let records = biors_core::parse_fasta_records(">seq1\nACDE\n").expect("parse FASTA");
    let export = convert_fasta_records(&records, SequenceKindSelection::Auto);
    let export_value = serde_json::to_value(export).expect("serialize conversion export");
    common::assert_json_value_matches_schema(
        &export_value,
        "schemas/bio-entity-export-output.v0.json",
    );

    let catalog = serde_json::to_value(task_templates()).expect("serialize template catalog");
    common::assert_json_value_matches_schema(
        &catalog,
        "schemas/task-template-catalog-output.v0.json",
    );

    let template = find_task_template("molecule-property-prediction-v0").expect("template exists");
    let template_value = serde_json::to_value(template).expect("serialize template");
    common::assert_json_value_matches_schema(
        &template_value,
        "schemas/task-template-output.v0.json",
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

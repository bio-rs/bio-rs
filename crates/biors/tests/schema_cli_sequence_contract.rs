use serde_json::Value;
use std::fs;

mod common;

#[test]
fn cli_outputs_match_sequence_schemas() {
    let tokenize = common::run_biors_stdin(&["tokenize", "-"], ">seq1\nACDE\n").stdout;
    common::assert_payload_matches_schema(&tokenize, "schemas/tokenize-output.v0.json");

    let special_config =
        common::repo_root().join("testdata/model-input-contract/protein-20-special.config.json");
    let special_fasta = common::repo_root().join("testdata/model-input-contract/protein.fasta");
    let special_tokenize = common::run_biors_paths(
        &["tokenize", "--config"],
        &[&special_config, &special_fasta],
    )
    .stdout;
    common::assert_payload_matches_schema(&special_tokenize, "schemas/tokenize-output.v0.json");

    let inspect = common::run_biors_stdin(&["inspect", "-"], ">seq1\nACDE\n>seq2\nAX\n").stdout;
    common::assert_payload_matches_schema(&inspect, "schemas/inspect-output.v0.json");

    let fasta_validate =
        common::run_biors_stdin(&["fasta", "validate", "-"], ">seq1\nAX*\n").stdout;
    common::assert_payload_matches_schema(
        &fasta_validate,
        "schemas/fasta-validation-output.v0.json",
    );

    let seq_validate = common::run_biors_stdin(&["seq", "validate", "-"], ">seq1\nACGN\n").stdout;
    common::assert_payload_matches_schema(&seq_validate, "schemas/fasta-validation-output.v0.json");

    let model_input = common::run_biors_stdin(
        &["model-input", "--max-length", "4", "-"],
        ">seq1\nACDEFG\n",
    )
    .stdout;
    common::assert_payload_matches_schema(&model_input, "schemas/model-input-output.v0.json");

    let workflow =
        common::run_biors_stdin(&["workflow", "--max-length", "4", "-"], ">seq1\nACDEFG\n").stdout;
    common::assert_payload_matches_schema(&workflow, "schemas/sequence-workflow-output.v0.json");

    let pipeline =
        common::run_biors_stdin(&["pipeline", "--max-length", "4", "-"], ">seq1\nACDE\n").stdout;
    common::assert_payload_matches_schema(&pipeline, "schemas/pipeline-output.v0.json");

    let pipeline_config_path = common::repo_root().join("testdata/pipeline/protein.toml");
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
    common::assert_payload_matches_schema(&pipeline_config, "schemas/pipeline-output.v0.json");

    let pipeline_config_json: Value = serde_json::from_str(
        &fs::read_to_string(common::repo_root().join("testdata/pipeline/protein.json"))
            .expect("read pipeline JSON config"),
    )
    .expect("pipeline config JSON");
    common::assert_json_value_matches_schema(
        &pipeline_config_json,
        "schemas/pipeline-config.v0.json",
    );

    let pipeline_lock_json: Value = serde_json::from_str(
        &fs::read_to_string(common::repo_root().join("testdata/pipeline/pipeline.lock"))
            .expect("read pipeline lock example"),
    )
    .expect("pipeline lock JSON");
    common::assert_json_value_matches_schema(&pipeline_lock_json, "schemas/pipeline-lock.v0.json");

    let debug =
        common::run_biors_stdin(&["debug", "--max-length", "4", "-"], ">seq1\nAX*\n").stdout;
    common::assert_payload_matches_schema(&debug, "schemas/sequence-debug-output.v0.json");
}

#[test]
fn direct_core_workflow_output_matches_shared_schema() {
    let output = biors_core::workflow::prepare_protein_model_input_workflow(
        "fnv1a64:08a331cb13c7bd72".to_string(),
        &[biors_core::sequence::ProteinSequence::new_normalized(
            "seq1", "ACDE",
        )],
        biors_core::model_input::ModelInputPolicy {
            max_length: 4,
            pad_token_id: 0,
            padding: biors_core::model_input::PaddingPolicy::NoPadding,
        },
    )
    .expect("direct core workflow output");
    let value = serde_json::to_value(output).expect("workflow JSON");

    common::assert_json_value_matches_schema(&value, "schemas/sequence-workflow-output.v0.json");
}

#[test]
fn pipeline_schema_constrains_nested_workflow_payload() {
    let pipeline =
        common::run_biors_stdin(&["pipeline", "--max-length", "4", "-"], ">seq1\nACDE\n").stdout;
    let envelope: Value = serde_json::from_slice(&pipeline).expect("pipeline JSON");
    let workflow = &envelope["data"]["workflow"];
    common::assert_json_value_matches_schema(workflow, "schemas/sequence-workflow-output.v0.json");
    assert_eq!(
        workflow["provenance"]["invocation"]["command"],
        "biors pipeline"
    );

    let pipeline_config_path = common::repo_root().join("testdata/pipeline/protein.toml");
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
    let config_envelope: Value =
        serde_json::from_slice(&pipeline_config).expect("config pipeline JSON");
    let config_workflow = &config_envelope["data"]["workflow"];
    common::assert_json_value_matches_schema(
        config_workflow,
        "schemas/sequence-workflow-output.v0.json",
    );
    assert_eq!(
        config_workflow["provenance"]["invocation"]["command"],
        "biors pipeline --config"
    );

    let mut incomplete = envelope["data"].clone();
    incomplete["workflow"] = serde_json::json!({
        "workflow": "protein_model_input.v0"
    });
    common::assert_payload_rejected_by_schema(&incomplete, "schemas/pipeline-output.v0.json");
}

#[test]
fn cli_outputs_match_diff_schema() {
    let expected = common::repo_root().join("testdata/protein-package/fixtures/tiny.output.json");
    let observed =
        common::repo_root().join("testdata/protein-package/observed/tiny.reordered.json");
    let diff = common::run_biors_paths(&["diff"], &[&expected, &observed]).stdout;
    common::assert_payload_matches_schema(&diff, "schemas/output-diff.v0.json");
}

#[test]
fn cli_outputs_match_batch_schema() {
    let examples = common::repo_root().join("testdata/sequences");
    let batch_validate =
        common::run_biors_paths(&["batch", "validate", "--kind", "auto"], &[&examples]).stdout;
    common::assert_payload_matches_schema(
        &batch_validate,
        "schemas/batch-validation-output.v0.json",
    );

    let dataset_inspect = common::run_biors_paths(&["dataset", "inspect"], &[&examples]).stdout;
    common::assert_payload_matches_schema(
        &dataset_inspect,
        "schemas/dataset-inspect-output.v0.json",
    );
}

#[test]
fn cli_outputs_match_tooling_schemas() {
    let tokenizer_inspect = common::run_biors_paths(
        &["tokenizer", "inspect", "--profile", "protein-20-special"],
        &[],
    )
    .stdout;
    common::assert_payload_matches_schema(
        &tokenizer_inspect,
        "schemas/tokenizer-inspect-output.v0.json",
    );

    let doctor = common::run_biors_paths(&["doctor"], &[]).stdout;
    common::assert_payload_matches_schema(&doctor, "schemas/doctor-output.v0.json");

    let service = common::run_biors_paths(&["service", "contract"], &[]).stdout;
    common::assert_payload_matches_schema(&service, "schemas/service-interface-output.v0.json");

    let hosted_boundary = common::run_biors_paths(&["service", "hosted-boundary"], &[]).stdout;
    common::assert_payload_matches_schema(
        &hosted_boundary,
        "schemas/hosted-workflow-boundary-output.v0.json",
    );

    let temp = common::TempDir::new("schema-tooling");
    let hf_config = temp.write(
        "tokenizer_config.json",
        r#"{"tokenizer_class":"BertTokenizer","cls_token":"[CLS]","sep_token":"[SEP]"}"#,
    );
    let tokenizer_conversion =
        common::run_biors_paths(&["tokenizer", "convert-hf"], &[&hf_config]).stdout;
    common::assert_payload_matches_schema(
        &tokenizer_conversion,
        "schemas/tokenizer-conversion-output.v0.json",
    );

    let cache = common::run_biors_paths(&["cache", "inspect", "--root"], &[temp.path()]).stdout;
    common::assert_payload_matches_schema(&cache, "schemas/cache-output.v0.json");
}

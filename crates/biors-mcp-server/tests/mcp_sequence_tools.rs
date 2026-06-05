mod mcp_support;

use mcp_support::{
    assert_json_value_matches_schema, call_tool_error, call_tool_error_debug, call_tool_json,
};

#[tokio::test]
async fn test_doctor_tool() {
    let json = call_tool_json("doctor", serde_json::Map::new()).await;
    assert_eq!(json["mcp_server_ready"], true);
    assert!(json["biors_version"].as_str().unwrap().starts_with("0."));
}

#[tokio::test]
async fn test_tokenize_tool() {
    let mut args = serde_json::Map::new();
    args.insert(
        "fasta_text".to_string(),
        serde_json::Value::String(">seq1\nACDEFGHIKLMNPQRSTVWY\n".to_string()),
    );
    args.insert(
        "profile".to_string(),
        serde_json::Value::String("protein-20".to_string()),
    );
    let json = call_tool_json("tokenize", args).await;
    assert!(json.is_array());
    assert_eq!(json[0]["id"], "seq1");
}

#[tokio::test]
async fn test_tokenize_tool_accepts_nucleotide_profiles() {
    let mut args = serde_json::Map::new();
    args.insert(
        "fasta_text".to_string(),
        serde_json::Value::String(">dna\nACGTN\n".to_string()),
    );
    args.insert(
        "profile".to_string(),
        serde_json::Value::String("dna-iupac".to_string()),
    );
    let json = call_tool_json("tokenize", args).await;
    assert_eq!(json[0]["alphabet"], "dna-iupac");
    assert_eq!(json[0]["tokens"], serde_json::json!([0, 1, 2, 3, 4]));
    assert_eq!(json[0]["valid"], false);
}

#[tokio::test]
async fn test_validate_tool() {
    let mut args = serde_json::Map::new();
    args.insert(
        "fasta_text".to_string(),
        serde_json::Value::String(">seq1\nACDEFGHIKLMNPQRSTVWY\n".to_string()),
    );
    args.insert(
        "kind".to_string(),
        serde_json::Value::String("protein".to_string()),
    );
    let json = call_tool_json("validate", args).await;
    assert_eq!(json["records"], 1);
}

#[tokio::test]
async fn test_workflow_tool_matches_core_contract_defaults() {
    let mut args = serde_json::Map::new();
    args.insert(
        "fasta_text".to_string(),
        serde_json::Value::String(">seq1\nACDE\n".to_string()),
    );
    args.insert("max_length".to_string(), serde_json::json!(6));

    let json = call_tool_json("workflow", args).await;
    assert_eq!(json["workflow"], "protein_model_input.v0");
    assert_eq!(json["model_ready"], true);
    assert_eq!(
        json["provenance"]["invocation"]["command"],
        "biors-mcp workflow"
    );
    assert_eq!(
        json["provenance"]["model_input_policy"]["padding"],
        "fixed_length"
    );
    assert_eq!(json["provenance"]["model_input_policy"]["pad_token_id"], 0);
    assert!(json["provenance"]["input_hash"]
        .as_str()
        .expect("input_hash")
        .starts_with("fnv1a64:"));
    assert_eq!(
        json["model_input"]["records"][0]["input_ids"],
        serde_json::json!([0, 1, 2, 3, 0, 0])
    );
    assert_json_value_matches_schema(&json, "schemas/sequence-workflow-output.v0.json");
}

#[tokio::test]
async fn test_workflow_tool_accepts_nucleotide_profiles() {
    let mut args = serde_json::Map::new();
    args.insert(
        "fasta_text".to_string(),
        serde_json::Value::String(">dna\nACGT\n".to_string()),
    );
    args.insert(
        "kind".to_string(),
        serde_json::Value::String("dna".to_string()),
    );
    args.insert(
        "profile".to_string(),
        serde_json::Value::String("dna-iupac".to_string()),
    );
    args.insert("max_length".to_string(), serde_json::json!(6));

    let json = call_tool_json("workflow", args).await;
    assert_eq!(json["workflow"], "sequence_model_input.v0");
    assert_eq!(json["model_ready"], true);
    assert_eq!(json["provenance"]["tokenizer"]["name"], "dna-iupac");
    assert_eq!(json["provenance"]["validation_alphabet"], "dna-iupac");
    assert_eq!(
        json["model_input"]["records"][0]["input_ids"],
        serde_json::json!([0, 1, 2, 3, 0, 0])
    );
    assert_json_value_matches_schema(&json, "schemas/sequence-workflow-output.v0.json");

    let mut auto_args = serde_json::Map::new();
    auto_args.insert(
        "fasta_text".to_string(),
        serde_json::Value::String(">rna\nACGU\n".to_string()),
    );
    auto_args.insert("max_length".to_string(), serde_json::json!(4));
    let auto_json = call_tool_json("workflow", auto_args).await;
    assert_eq!(auto_json["provenance"]["tokenizer"]["name"], "rna-iupac");
}

#[tokio::test]
async fn test_workflow_tool_reports_non_model_ready_residues() {
    let mut args = serde_json::Map::new();
    args.insert(
        "fasta_text".to_string(),
        serde_json::Value::String(">seq1\nAC*X\n".to_string()),
    );
    args.insert("max_length".to_string(), serde_json::json!(8));

    let json = call_tool_json("workflow", args).await;
    assert_eq!(json["model_ready"], false);
    assert!(json["model_input"].is_null());
    assert_eq!(
        json["readiness_issues"][0]["code"],
        "sequence.not_model_ready"
    );
}

#[tokio::test]
async fn test_workflow_tool_rejects_empty_sequence_records() {
    let mut args = serde_json::Map::new();
    args.insert(
        "fasta_text".to_string(),
        serde_json::Value::String(">empty\n".to_string()),
    );
    args.insert("max_length".to_string(), serde_json::json!(8));

    let error = call_tool_error("workflow", args).await;
    assert!(error.contains("missing sequence") || error.contains("empty"));
}

#[tokio::test]
async fn test_workflow_tool_classifies_invalid_model_policy_as_invalid_params() {
    let mut args = serde_json::Map::new();
    args.insert(
        "fasta_text".to_string(),
        serde_json::Value::String(">seq1\nACDE\n".to_string()),
    );
    args.insert("max_length".to_string(), serde_json::json!(0));

    let error = call_tool_error_debug("workflow", args).await;
    assert!(error.contains("ErrorCode(-32602)"), "{error}");
    assert!(error.contains("max_length must be greater than zero"));
}

#[tokio::test]
async fn test_sequence_tools_classify_invalid_fasta_as_invalid_params() {
    for tool_name in ["tokenize", "validate", "workflow"] {
        let mut args = serde_json::Map::new();
        args.insert(
            "fasta_text".to_string(),
            serde_json::Value::String("ACDE\n".to_string()),
        );
        if tool_name == "workflow" {
            args.insert("max_length".to_string(), serde_json::json!(8));
        }

        let error = call_tool_error_debug(tool_name, args).await;
        assert!(
            error.contains("ErrorCode(-32602)"),
            "{tool_name} did not return invalid params code: {error}"
        );
        assert!(
            error.contains("fasta.missing_header"),
            "{tool_name} did not include FASTA diagnostic code: {error}"
        );
    }
}

#[tokio::test]
async fn test_validate_tool_classifies_empty_input_as_invalid_params() {
    let mut args = serde_json::Map::new();
    args.insert(
        "fasta_text".to_string(),
        serde_json::Value::String("".to_string()),
    );

    let error = call_tool_error_debug("validate", args).await;
    assert!(error.contains("ErrorCode(-32602)"));
    assert!(error.contains("fasta.empty_input"));
}

#[tokio::test]
async fn test_workflow_tool_rejects_kind_profile_mismatch_and_bad_padding() {
    let mut dna_args = serde_json::Map::new();
    dna_args.insert(
        "fasta_text".to_string(),
        serde_json::Value::String(">seq1\nACGT\n".to_string()),
    );
    dna_args.insert(
        "kind".to_string(),
        serde_json::Value::String("dna".to_string()),
    );
    dna_args.insert(
        "profile".to_string(),
        serde_json::Value::String("protein-20".to_string()),
    );
    let dna_error = call_tool_error("workflow", dna_args).await;
    assert!(dna_error.contains("workflow kind/profile mismatch"));

    let mut padding_args = serde_json::Map::new();
    padding_args.insert(
        "fasta_text".to_string(),
        serde_json::Value::String(">seq1\nACDE\n".to_string()),
    );
    padding_args.insert(
        "padding".to_string(),
        serde_json::Value::String("bad".to_string()),
    );
    let padding_error = call_tool_error("workflow", padding_args).await;
    assert!(padding_error.contains("invalid padding"));
}

#[tokio::test]
async fn test_tokenize_tool_rejects_invalid_profile() {
    let mut args = serde_json::Map::new();
    args.insert(
        "fasta_text".to_string(),
        serde_json::Value::String(">seq1\nACDE\n".to_string()),
    );
    args.insert(
        "profile".to_string(),
        serde_json::Value::String("dna".to_string()),
    );
    let error = call_tool_error("tokenize", args).await;
    assert!(error.contains("invalid profile"));
}

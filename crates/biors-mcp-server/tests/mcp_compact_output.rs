mod mcp_support;

use mcp_support::call_tool_json;

#[tokio::test]
async fn test_long_sequence_tools_default_to_compact_output_and_expand_explicitly() {
    let mut fasta = String::new();
    for index in 0..80 {
        fasta.push_str(&format!(">seq{index}\nACDEFGHIKLMNPQRSTVWY\n"));
    }

    let mut validate_args = serde_json::Map::new();
    validate_args.insert(
        "fasta_text".to_string(),
        serde_json::Value::String(fasta.clone()),
    );
    validate_args.insert(
        "kind".to_string(),
        serde_json::Value::String("protein".to_string()),
    );
    let compact_validate = call_tool_json("validate", validate_args.clone()).await;
    assert_eq!(compact_validate["schema_version"], "biors.mcp.compact.v0");
    assert_eq!(compact_validate["tool"], "validate");
    assert_eq!(compact_validate["compact"], true);
    assert_eq!(compact_validate["summary"]["records"], 80);
    assert!(compact_validate.get("sequences").is_none());

    validate_args.insert("include_records".to_string(), serde_json::Value::Bool(true));
    let full_validate = call_tool_json("validate", validate_args).await;
    assert_eq!(
        full_validate["sequences"]
            .as_array()
            .expect("sequences")
            .len(),
        80
    );

    let mut tokenize_args = serde_json::Map::new();
    tokenize_args.insert(
        "fasta_text".to_string(),
        serde_json::Value::String(fasta.clone()),
    );
    tokenize_args.insert(
        "profile".to_string(),
        serde_json::Value::String("protein-20".to_string()),
    );
    let compact_tokenize = call_tool_json("tokenize", tokenize_args.clone()).await;
    assert_eq!(compact_tokenize["schema_version"], "biors.mcp.compact.v0");
    assert_eq!(compact_tokenize["tool"], "tokenize");
    assert_eq!(compact_tokenize["compact"], true);
    assert_eq!(compact_tokenize["summary"]["records"], 80);
    assert!(compact_tokenize.get("records").is_none());

    tokenize_args.insert("include_records".to_string(), serde_json::Value::Bool(true));
    let full_tokenize = call_tool_json("tokenize", tokenize_args).await;
    assert_eq!(
        full_tokenize.as_array().expect("tokenized records").len(),
        80
    );

    let mut workflow_args = serde_json::Map::new();
    workflow_args.insert("fasta_text".to_string(), serde_json::Value::String(fasta));
    workflow_args.insert("max_length".to_string(), serde_json::json!(24));
    let compact_workflow = call_tool_json("workflow", workflow_args.clone()).await;
    assert_eq!(compact_workflow["schema_version"], "biors.mcp.compact.v0");
    assert_eq!(compact_workflow["tool"], "workflow");
    assert_eq!(compact_workflow["compact"], true);
    assert_eq!(compact_workflow["model_ready"], true);
    assert_eq!(compact_workflow["validation_summary"]["records"], 80);
    assert!(compact_workflow.get("model_input").is_none());

    workflow_args.insert("include_payload".to_string(), serde_json::Value::Bool(true));
    let full_workflow = call_tool_json("workflow", workflow_args).await;
    assert_eq!(full_workflow["workflow"], "protein_model_input.v0");
    assert_eq!(
        full_workflow["model_input"]["records"]
            .as_array()
            .unwrap()
            .len(),
        80
    );
}

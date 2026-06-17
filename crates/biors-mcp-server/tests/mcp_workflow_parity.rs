mod mcp_support;

use biors_core::model_input::{ModelInputPolicy, PaddingPolicy};
use biors_core::sequence::ProteinSequence;
use biors_core::tokenizer::{protein_tokenizer_config_for_profile, ProteinTokenizerProfile};
use biors_core::verification::stable_input_hash;
use mcp_support::call_tool_json;

#[tokio::test]
async fn test_mcp_workflow_core_parity_for_canonical_protein() {
    let fasta = ">protein_example\nACDEFGHIK\n";
    let mut args = serde_json::Map::new();
    args.insert(
        "fasta_text".to_string(),
        serde_json::Value::String(fasta.to_string()),
    );
    args.insert("max_length".to_string(), serde_json::json!(16));

    let mcp = call_tool_json("workflow", args).await;
    let records = vec![ProteinSequence::new_normalized(
        "protein_example",
        "ACDEFGHIK",
    )];
    let core = biors_core::workflow::prepare_model_input_workflow_with_config(
        stable_input_hash(fasta),
        &records,
        ModelInputPolicy {
            max_length: 16,
            pad_token_id: 0,
            padding: PaddingPolicy::FixedLength,
        },
        protein_tokenizer_config_for_profile(ProteinTokenizerProfile::Protein20),
        biors_core::workflow::SequenceWorkflowInvocation {
            command: "parity".to_string(),
            arguments: vec![],
        },
    )
    .expect("core workflow");

    assert_eq!(mcp["workflow"], core.workflow);
    assert_eq!(mcp["model_ready"], core.model_ready);
    assert_eq!(
        mcp["model_input"]["records"][0]["input_ids"],
        serde_json::to_value(&core.model_input.expect("core model input").records[0].input_ids)
            .expect("core input_ids JSON")
    );
}

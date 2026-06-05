use serde_json::Value;
use std::path::Path;

mod common;

#[test]
fn full_workflow_e2e_covers_researcher_cli_path() {
    let repo = common::repo_root();
    let fasta = repo.join("testdata/sequences/protein.fasta");
    let manifest = repo.join("testdata/protein-package/manifest.json");
    let observations = repo.join("testdata/protein-package/observations.json");

    let validation = run_biors(&["fasta", "validate"], &[&fasta]);
    assert_eq!(validation["data"]["records"], 1);
    assert_eq!(validation["data"]["error_count"], 0);

    let sequence_validation = run_biors(&["seq", "validate"], &[&fasta]);
    assert_eq!(sequence_validation["data"]["records"], 1);
    assert_eq!(sequence_validation["data"]["kind_counts"]["protein"], 1);

    let tokenized = run_biors(&["tokenize"], &[&fasta]);
    assert_eq!(tokenized["data"][0]["alphabet"], "protein-20");
    assert!(
        tokenized["data"][0]["tokens"]
            .as_array()
            .expect("tokens")
            .len()
            >= 4
    );

    let tokenizer = run_biors(
        &["tokenizer", "inspect", "--profile", "protein-20-special"],
        &[],
    );
    assert_eq!(tokenizer["data"]["profile"], "protein-20-special");
    assert_eq!(tokenizer["data"]["special_tokens"]["pad"]["token_id"], 21);

    let model_input = run_biors(&["model-input", "--max-length", "8"], &[&fasta]);
    assert_eq!(model_input["data"]["policy"]["max_length"], 8);
    assert_eq!(
        model_input["data"]["records"][0]["attention_mask"]
            .as_array()
            .expect("attention mask")
            .len(),
        8
    );

    let workflow = run_biors(&["workflow", "--max-length", "8"], &[&fasta]);
    assert_eq!(workflow["data"]["workflow"], "protein_model_input.v0");
    assert_eq!(workflow["data"]["model_ready"], true);
    assert_eq!(workflow["data"]["validation"]["records"], 1);
    assert_eq!(workflow["data"]["tokenization"]["summary"]["records"], 1);
    assert_eq!(workflow["data"]["model_input"]["policy"]["max_length"], 8);
    assert_eq!(
        workflow["data"]["provenance"]["invocation"]["command"],
        "biors workflow"
    );
    assert!(
        workflow["data"]["provenance"]["hashes"]["output_data_sha256"]
            .as_str()
            .expect("workflow output hash")
            .starts_with("sha256:")
    );

    let sequences = repo.join("testdata/sequences");
    let batch = run_biors(&["batch", "validate", "--kind", "auto"], &[&sequences]);
    assert!(batch["data"]["summary"]["files"].as_u64().expect("files") >= 3);
    assert!(
        batch["data"]["summary"]["records"]
            .as_u64()
            .expect("records")
            >= 3
    );

    let package_validation = run_biors(&["package", "validate"], &[&manifest]);
    assert_eq!(package_validation["data"]["valid"], true);

    let package_verification = run_biors(&["package", "verify"], &[&manifest, &observations]);
    assert_eq!(package_verification["data"]["failed"], 0);
}

fn run_biors(args: &[&str], paths: &[&Path]) -> Value {
    let output = common::run_biors_paths(args, paths);
    serde_json::from_slice(&output.stdout).expect("valid JSON")
}

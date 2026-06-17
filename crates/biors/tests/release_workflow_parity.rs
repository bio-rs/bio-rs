use biors_core::model_input::{ModelInputPolicy, PaddingPolicy};
use biors_core::package::{plan_runtime_bridge, PackageManifest};
use biors_core::sequence::ProteinSequence;
use biors_core::tokenizer::{protein_tokenizer_config_for_profile, ProteinTokenizerProfile};
use biors_core::verification::stable_input_hash;
use serde_json::Value;
use std::fs;

mod common;

#[test]
fn workflow_parity_doc_maps_surfaces_and_gaps() {
    let repo = common::repo_root();
    let doc =
        fs::read_to_string(repo.join("docs/1-0-workflow-parity.md")).expect("read parity doc");

    for expected in [
        "protein validate/tokenize/model-input/workflow",
        "invalid sequence",
        "package validate/bridge",
        "service batch validation",
        "CLI",
        "Rust core",
        "Python",
        "WASM",
        "MCP",
        "service",
        "not exposed",
        "unsupported on this surface",
    ] {
        assert!(doc.contains(expected), "parity doc missing {expected}");
    }
}

#[test]
fn canonical_protein_workflow_cli_core_parity() {
    let repo = common::repo_root();
    let fixture = repo.join("testdata/researcher-workflows/protein.fasta");
    let fasta = ">protein_example\nACDEFGHIK\n";
    let cli = common::run_biors_paths(&["workflow", "--max-length", "16"], &[&fixture]);
    let cli_value: Value = serde_json::from_slice(&cli.stdout).expect("CLI workflow JSON");

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

    assert_eq!(cli_value["data"]["workflow"], core.workflow);
    assert_eq!(cli_value["data"]["model_ready"], core.model_ready);
    assert_eq!(
        cli_value["data"]["model_input"]["records"][0]["input_ids"],
        serde_json::to_value(&core.model_input.expect("core model input").records[0].input_ids)
            .expect("core input_ids JSON")
    );
}

#[test]
fn invalid_sequence_cli_core_parity() {
    let fasta = ">seq1\nAC*X\n";
    let output = common::run_biors_stdin(&["workflow", "--max-length", "8", "-"], fasta);
    let cli_value: Value = serde_json::from_slice(&output.stdout).expect("CLI workflow JSON");

    let records = vec![ProteinSequence::new_normalized("seq1", "AC*X")];
    let core = biors_core::workflow::prepare_model_input_workflow_with_config(
        stable_input_hash(fasta),
        &records,
        ModelInputPolicy {
            max_length: 8,
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

    assert_eq!(cli_value["data"]["model_ready"], false);
    assert_eq!(core.model_ready, false);
    assert_eq!(
        cli_value["data"]["readiness_issues"][0]["code"],
        core.readiness_issues[0].code
    );
    assert!(cli_value["data"]["model_input"].is_null());
    assert!(core.model_input.is_none());
}

#[test]
fn package_bridge_cli_core_parity() {
    let repo = common::repo_root();
    let manifest_path = repo.join("testdata/protein-package/manifest.json");
    let cli = common::run_biors_paths(&["package", "bridge"], &[&manifest_path]);
    let cli_value: Value = serde_json::from_slice(&cli.stdout).expect("CLI bridge JSON");

    let manifest: PackageManifest =
        serde_json::from_str(&fs::read_to_string(manifest_path).expect("read package manifest"))
            .expect("manifest JSON");
    let core = plan_runtime_bridge(&manifest);

    assert_eq!(cli_value["data"]["ready"], core.ready);
    assert_eq!(cli_value["data"]["contract_ready"], core.contract_ready);
    assert_eq!(cli_value["data"]["artifact_checked"], core.artifact_checked);
    assert_eq!(cli_value["data"]["execution_ready"], core.execution_ready);
    assert_eq!(
        cli_value["data"]["execution_provider"],
        core.execution_provider
    );
}

use serde_json::Value;
use std::path::Path;
use std::process::Command;

#[test]
fn tokenizer_inspect_outputs_special_token_policy() {
    let output = Command::new(env!("CARGO_BIN_EXE_biors"))
        .arg("tokenizer")
        .arg("inspect")
        .arg("--profile")
        .arg("protein-20-special")
        .output()
        .expect("run biors tokenizer inspect");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stderr.is_empty());

    let value: Value = serde_json::from_slice(&output.stdout).expect("valid JSON output");
    assert_eq!(value["data"]["profile"], "protein-20-special");
    assert_eq!(value["data"]["special_tokens"]["pad"]["token_id"], 21);
    assert_eq!(value["data"]["special_tokens"]["cls"]["token_id"], 22);
    assert_eq!(value["data"]["special_tokens"]["sep"]["token_id"], 23);
    assert_eq!(value["data"]["special_tokens"]["mask"]["token_id"], 24);
}

#[test]
fn tokenize_accepts_tokenizer_config_file() {
    let repo = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
    let config = repo.join("examples/model-input-contract/protein-20-special.config.json");
    let fasta = repo.join("examples/model-input-contract/protein.fasta");

    let output = Command::new(env!("CARGO_BIN_EXE_biors"))
        .arg("tokenize")
        .arg("--config")
        .arg(config)
        .arg(fasta)
        .output()
        .expect("run biors tokenize with config");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let value: Value = serde_json::from_slice(&output.stdout).expect("valid JSON output");
    assert_eq!(value["data"][0]["alphabet"], "protein-20-special");
    assert_eq!(
        value["data"][0]["tokens"],
        serde_json::json!([22, 0, 1, 2, 3, 23])
    );
}

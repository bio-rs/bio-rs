use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

mod common;
use common::TempDir;

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

#[test]
fn tokenizer_convert_hf_writes_biors_tokenizer_config() {
    let temp = TempDir::new("hf-tokenizer-convert");
    let hf_config = temp.write(
        "tokenizer_config.json",
        r#"{
  "tokenizer_class": "BertTokenizer",
  "model_max_length": 1024,
  "do_lower_case": false,
  "unk_token": "[UNK]",
  "pad_token": "[PAD]",
  "cls_token": "[CLS]",
  "sep_token": "[SEP]",
  "mask_token": "[MASK]"
}"#,
    );
    let output_path = temp.path().join("biors-tokenizer.json");

    let output = Command::new(env!("CARGO_BIN_EXE_biors"))
        .arg("tokenizer")
        .arg("convert-hf")
        .arg(&hf_config)
        .arg("--output")
        .arg(&output_path)
        .output()
        .expect("run biors tokenizer convert-hf");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stderr.is_empty());

    let value: Value = serde_json::from_slice(&output.stdout).expect("valid JSON output");
    assert_eq!(
        value["data"]["source_format"],
        "huggingface.tokenizer_config"
    );
    assert_eq!(value["data"]["config"]["profile"], "protein-20-special");
    assert_eq!(value["data"]["config"]["add_special_tokens"], true);
    assert_eq!(value["data"]["conversion_status"]["status"], "preview");
    assert_eq!(value["data"]["conversion_status"]["package_ready"], false);
    assert_eq!(
        value["data"]["conversion_status"]["fixture_parity_required"],
        true
    );
    assert!(value["data"]["warnings"]
        .as_array()
        .expect("warnings")
        .iter()
        .any(|warning| warning
            .as_str()
            .expect("warning")
            .contains("validate fixture parity")));
    assert!(value["data"]["package_tokenizer_asset"].is_null());
    assert!(value["data"]["preview_tokenizer_asset"].is_object());
    assert_eq!(
        value["data"]["preview_tokenizer_asset"]["path"],
        "tokenizers/protein-20-special.json"
    );
    assert_eq!(
        value["data"]["preview_tokenizer_asset"]["checksum"],
        value["data"]["config_sha256"]
    );
    assert!(value["data"]["warnings"]
        .as_array()
        .expect("warnings")
        .iter()
        .any(|warning| warning
            .as_str()
            .expect("warning")
            .contains("outside a package tokenizers/ directory")));
    assert_eq!(
        value["data"]["output_path"],
        output_path.display().to_string()
    );

    let written: Value = serde_json::from_str(
        &fs::read_to_string(output_path).expect("read converted tokenizer config"),
    )
    .expect("written tokenizer JSON");
    assert_eq!(written["profile"], "protein-20-special");
    assert_eq!(written["add_special_tokens"], true);
}

#[test]
fn tokenizer_convert_hf_emits_package_relative_preview_path_for_tokenizers_dir() {
    let temp = TempDir::new("hf-tokenizer-package-path");
    let hf_config = temp.write(
        "tokenizer_config.json",
        r#"{
  "tokenizer_class": "BertTokenizer"
}"#,
    );
    let output_path = temp.path().join("protein-package/tokenizers/custom.json");

    let value = run_tokenizer_convert(&hf_config, &output_path);

    assert_eq!(value["data"]["config"]["profile"], "protein-20");
    assert_eq!(
        value["data"]["preview_tokenizer_asset"]["path"],
        "tokenizers/custom.json"
    );
    assert!(value["data"]["warnings"]
        .as_array()
        .expect("warnings")
        .iter()
        .all(|warning| !warning
            .as_str()
            .expect("warning")
            .contains("outside a package tokenizers/ directory")));
}

#[test]
fn tokenizer_convert_hf_preview_asset_can_be_used_in_package_manifest() {
    let temp = TempDir::new("hf-tokenizer-package-validate");
    let package_root = temp.path().join("protein-package");
    copy_dir(
        &common::repo_root().join("examples/protein-package"),
        &package_root,
    );
    let hf_config = temp.write(
        "tokenizer_config.json",
        r#"{
  "tokenizer_class": "BertTokenizer"
}"#,
    );
    let output_path = package_root.join("tokenizers/protein-20.json");

    let value = run_tokenizer_convert(&hf_config, &output_path);
    let mut manifest: Value = serde_json::from_str(
        &fs::read_to_string(package_root.join("manifest.json")).expect("read manifest"),
    )
    .expect("manifest JSON");
    manifest["tokenizer"] = value["data"]["preview_tokenizer_asset"].clone();
    fs::write(
        package_root.join("manifest.json"),
        format!(
            "{}\n",
            serde_json::to_string_pretty(&manifest).expect("serialize manifest")
        ),
    )
    .expect("write manifest");

    let validate = Command::new(env!("CARGO_BIN_EXE_biors"))
        .arg("package")
        .arg("validate")
        .arg(package_root.join("manifest.json"))
        .output()
        .expect("run package validate");

    assert!(
        validate.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&validate.stderr)
    );
}

fn run_tokenizer_convert(hf_config: &Path, output_path: &Path) -> Value {
    let output = Command::new(env!("CARGO_BIN_EXE_biors"))
        .arg("tokenizer")
        .arg("convert-hf")
        .arg(hf_config)
        .arg("--output")
        .arg(output_path)
        .output()
        .expect("run biors tokenizer convert-hf");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).expect("valid JSON output")
}

fn copy_dir(source: &Path, destination: &Path) {
    fs::create_dir_all(destination).expect("create destination directory");
    for entry in fs::read_dir(source).expect("read source directory") {
        let entry = entry.expect("read source entry");
        let source_path = entry.path();
        let destination_path: PathBuf = destination.join(entry.file_name());
        if source_path.is_dir() {
            copy_dir(&source_path, &destination_path);
        } else {
            fs::copy(&source_path, &destination_path).expect("copy file");
        }
    }
}

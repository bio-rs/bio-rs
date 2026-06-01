use serde_json::Value;
use std::fs;

mod common;

#[test]
fn package_manifest_example_uses_declared_schema_version() {
    let manifest: Value = serde_json::from_str(
        &fs::read_to_string(common::repo_root().join("examples/protein-package/manifest.json"))
            .expect("read package manifest"),
    )
    .expect("manifest JSON");

    assert_eq!(manifest["schema_version"], "biors.package.v1");
    assert_eq!(manifest["package_layout"]["models"], "models");
    assert_eq!(manifest["package_layout"]["docs"], "docs");
    assert_eq!(manifest["metadata"]["license"]["expression"], "CC0-1.0");
    assert_eq!(
        manifest["metadata"]["model_card"]["path"],
        "docs/model-card.md"
    );
    assert!(manifest["model"]["checksum"].is_string());
    assert!(manifest["tokenizer"]["checksum"].is_string());
    assert!(manifest["vocab"]["checksum"].is_string());
    assert!(manifest["metadata"]["license"]["file"]["checksum"].is_string());
    assert!(manifest["metadata"]["citation"]["file"]["checksum"].is_string());
    assert!(manifest["metadata"]["model_card"]["checksum"].is_string());
    assert!(manifest["expected_input"]["dtype"].is_string());
    assert!(manifest["fixtures"][0]["input_hash"]
        .as_str()
        .expect("fixture input hash")
        .starts_with("sha256:"));
    assert!(manifest["fixtures"][0]["expected_output_hash"]
        .as_str()
        .expect("fixture output hash")
        .starts_with("sha256:"));
    common::assert_json_value_matches_schema(&manifest, "schemas/package-manifest.v1.json");
}

#[test]
fn package_manifest_schemas_reject_empty_contract_identifiers() {
    let mut v1_manifest: Value = serde_json::from_str(
        &fs::read_to_string(common::repo_root().join("examples/protein-package/manifest.json"))
            .expect("read package manifest"),
    )
    .expect("manifest JSON");
    v1_manifest["tokenizer"]["name"] = Value::String(String::new());
    v1_manifest["tokenizer"]["contract_version"] = Value::String(String::new());
    v1_manifest["vocab"]["name"] = Value::String(String::new());
    v1_manifest["vocab"]["contract_version"] = Value::String(String::new());
    v1_manifest["preprocessing"][0]["name"] = Value::String(String::new());
    v1_manifest["preprocessing"][0]["implementation"] = Value::String(String::new());
    v1_manifest["preprocessing"][0]["contract"] = Value::String(String::new());
    v1_manifest["preprocessing"][0]["contract_version"] = Value::String(String::new());
    common::assert_payload_rejected_by_schema(&v1_manifest, "schemas/package-manifest.v1.json");

    let mut v0_manifest = v1_manifest;
    v0_manifest["schema_version"] = Value::String("biors.package.v0".into());
    if let Some(object) = v0_manifest.as_object_mut() {
        object.remove("package_layout");
        object.remove("metadata");
        if let Some(runtime) = object.get_mut("runtime").and_then(Value::as_object_mut) {
            runtime.remove("version");
        }
    }
    common::assert_payload_rejected_by_schema(&v0_manifest, "schemas/package-manifest.v0.json");
}

#[test]
fn package_manifest_schemas_reject_empty_fixture_names_and_shapes() {
    let mut v1_manifest: Value = serde_json::from_str(
        &fs::read_to_string(common::repo_root().join("examples/protein-package/manifest.json"))
            .expect("read package manifest"),
    )
    .expect("manifest JSON");
    v1_manifest["fixtures"][0]["name"] = Value::String(String::new());
    v1_manifest["expected_input"]["shape"] = Value::Array(vec![]);
    v1_manifest["expected_output"]["shape"] = Value::Array(vec![]);
    common::assert_payload_rejected_by_schema(&v1_manifest, "schemas/package-manifest.v1.json");

    let mut v0_manifest = v1_manifest;
    v0_manifest["schema_version"] = Value::String("biors.package.v0".into());
    if let Some(object) = v0_manifest.as_object_mut() {
        object.remove("package_layout");
        object.remove("metadata");
        if let Some(runtime) = object.get_mut("runtime").and_then(Value::as_object_mut) {
            runtime.remove("version");
        }
    }
    common::assert_payload_rejected_by_schema(&v0_manifest, "schemas/package-manifest.v0.json");
}

#[test]
fn cli_outputs_match_package_schemas() {
    let manifest = common::repo_root().join("examples/protein-package/manifest.json");
    let observations = common::repo_root().join("examples/protein-package/observations.json");

    let package_inspect = common::run_biors_paths(&["package", "inspect"], &[&manifest]).stdout;
    common::assert_payload_matches_schema(
        &package_inspect,
        "schemas/package-inspect-output.v0.json",
    );

    let package_validate = common::run_biors_paths(&["package", "validate"], &[&manifest]).stdout;
    common::assert_payload_matches_schema(
        &package_validate,
        "schemas/package-validation-report.v0.json",
    );

    let package_bridge = common::run_biors_paths(&["package", "bridge"], &[&manifest]).stdout;
    common::assert_payload_matches_schema(&package_bridge, "schemas/package-bridge-output.v0.json");

    let package_verify =
        common::run_biors_paths(&["package", "verify"], &[&manifest, &observations]).stdout;
    common::assert_payload_matches_schema(&package_verify, "schemas/package-verify-output.v0.json");

    let temp = common::TempDir::new("schema-package-skeleton");
    let project = temp.path().join("python-project");
    fs::create_dir_all(&project).expect("create project");
    fs::write(project.join("model.onnx"), b"onnx").expect("write model");
    fs::write(
        project.join("tokenizer_config.json"),
        r#"{"tokenizer_class":"BertTokenizer","cls_token":"[CLS]","sep_token":"[SEP]"}"#,
    )
    .expect("write tokenizer config");
    let fixture_input = temp.write("tiny.fasta", ">tiny\nACDE\n");
    let fixture_output = temp.write("tiny.output.json", r#"{"ok":true}"#);
    let output_dir = temp.path().join("package");
    let skeleton = std::process::Command::new(env!("CARGO_BIN_EXE_biors"))
        .arg("package")
        .arg("convert-project")
        .arg(&project)
        .arg("--output")
        .arg(&output_dir)
        .arg("--name")
        .arg("schema-package")
        .arg("--license")
        .arg("CC0-1.0")
        .arg("--citation")
        .arg("schema package fixture")
        .arg("--model-card-summary")
        .arg("Schema package fixture.")
        .arg("--intended-use")
        .arg("Schema validation")
        .arg("--limitation")
        .arg("Not for inference")
        .arg("--fixture-input")
        .arg(&fixture_input)
        .arg("--fixture-output")
        .arg(&fixture_output)
        .output()
        .expect("run package convert-project")
        .stdout;
    common::assert_payload_matches_schema(&skeleton, "schemas/package-skeleton-output.v0.json");
}

#[test]
fn package_rejection_examples_match_schemas() {
    let mismatch_report = serde_json::json!({
        "package": "protein-seed",
        "fixtures": 1,
        "passed": 0,
        "failed": 1,
        "observation_issues": [],
        "results": [
            {
                "name": "tiny-protein",
                "input_path": "fixtures/tiny.fasta",
                "expected_output_path": "fixtures/tiny.output.json",
                "observed_output_path": "observed/tiny.bad.json",
                "expected_output_hash": "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
                "observed_output_hash": "sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
                "status": "failed",
                "checksum_mismatch": true,
                "content_mismatch": true,
                "issue_code": "output_content_mismatch",
                "content_diff": {
                    "expected_path": "fixtures/tiny.output.json",
                    "observed_path": "observed/tiny.bad.json",
                    "expected_len": 32,
                    "observed_len": 28,
                    "first_difference": {
                        "byte_offset": 10,
                        "expected_byte": 34,
                        "observed_byte": 48
                    }
                },
                "issue": "output content mismatch between 'fixtures/tiny.output.json' and 'observed/tiny.bad.json'"
            }
        ]
    });
    common::assert_json_value_matches_schema(
        &mismatch_report,
        "schemas/package-verify-output.v0.json",
    );
}

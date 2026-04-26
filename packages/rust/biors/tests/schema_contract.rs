use jsonschema::JSONSchema;
use serde_json::Value;
use std::fs;
use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};

#[test]
fn machine_readable_schemas_are_valid_json() {
    let repo = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
    for schema in [
        "schemas/cli-success.v0.json",
        "schemas/cli-error.v0.json",
        "schemas/tokenize-output.v0.json",
        "schemas/inspect-output.v0.json",
        "schemas/model-input-output.v0.json",
        "schemas/fasta-validation-output.v0.json",
        "schemas/package-inspect-output.v0.json",
        "schemas/package-bridge-output.v0.json",
        "schemas/package-verify-output.v0.json",
        "schemas/package-manifest.v0.json",
        "schemas/package-validation-report.v0.json",
    ] {
        let input = fs::read_to_string(repo.join(schema)).expect("read schema");
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
    }
}

#[test]
fn package_manifest_example_uses_declared_schema_version() {
    let repo = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
    let manifest: Value = serde_json::from_str(
        &fs::read_to_string(repo.join("examples/protein-package/manifest.json"))
            .expect("read package manifest"),
    )
    .expect("manifest JSON");

    assert_eq!(manifest["schema_version"], "biors.package.v0");
    assert!(manifest["model"]["checksum"].is_string());
    assert!(manifest["tokenizer"]["checksum"].is_string());
    assert!(manifest["vocab"]["checksum"].is_string());
    assert!(manifest["expected_input"]["dtype"].is_string());
    assert!(manifest["fixtures"][0]["input_hash"]
        .as_str()
        .expect("fixture input hash")
        .starts_with("sha256:"));
    assert!(manifest["fixtures"][0]["expected_output_hash"]
        .as_str()
        .expect("fixture output hash")
        .starts_with("sha256:"));
}

#[test]
fn cli_outputs_match_declared_payload_schemas() {
    let tokenize = run_with_stdin(["tokenize", "-"], ">seq1\nACDE\n");
    assert_payload_matches_schema(&tokenize, "schemas/tokenize-output.v0.json");

    let inspect = run_with_stdin(["inspect", "-"], ">seq1\nACDE\n>seq2\nAX\n");
    assert_payload_matches_schema(&inspect, "schemas/inspect-output.v0.json");

    let fasta_validate = run_with_stdin(["fasta", "validate", "-"], ">seq1\nAX*\n");
    assert_payload_matches_schema(&fasta_validate, "schemas/fasta-validation-output.v0.json");

    let model_input = run_with_stdin(["model-input", "--max-length", "4", "-"], ">seq1\nACDEFG\n");
    assert_payload_matches_schema(&model_input, "schemas/model-input-output.v0.json");

    let repo = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
    let manifest = repo.join("examples/protein-package/manifest.json");
    let observations = repo.join("examples/protein-package/observations.json");

    let package_inspect = run_command(["package", "inspect"], &[manifest.as_os_str()]);
    assert_payload_matches_schema(&package_inspect, "schemas/package-inspect-output.v0.json");

    let package_bridge = run_command(["package", "bridge"], &[manifest.as_os_str()]);
    assert_payload_matches_schema(&package_bridge, "schemas/package-bridge-output.v0.json");

    let package_verify = run_command(
        ["package", "verify"],
        &[manifest.as_os_str(), observations.as_os_str()],
    );
    assert_payload_matches_schema(&package_verify, "schemas/package-verify-output.v0.json");
}

fn assert_payload_matches_schema(output: &[u8], schema_path: &str) {
    let repo = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
    let envelope: Value = serde_json::from_slice(output).expect("valid CLI JSON");
    let payload = envelope["data"].clone();
    let schema: Value = serde_json::from_str(
        &fs::read_to_string(repo.join(schema_path)).expect("read payload schema"),
    )
    .expect("schema JSON");
    let compiled = JSONSchema::compile(&schema).expect("compile schema");
    let validation = compiled.validate(&payload);
    if let Err(errors) = validation {
        let messages: Vec<_> = errors.map(|error| error.to_string()).collect();
        panic!("payload did not match schema {schema_path}: {messages:?}");
    }
}

fn run_with_stdin<const N: usize>(args: [&str; N], input: &str) -> Vec<u8> {
    let mut child = Command::new(env!("CARGO_BIN_EXE_biors"))
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn biors");

    child
        .stdin
        .as_mut()
        .expect("stdin pipe")
        .write_all(input.as_bytes())
        .expect("write stdin");

    let output = child.wait_with_output().expect("wait for biors");
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    output.stdout
}

fn run_command<const N: usize>(args: [&str; N], extra_args: &[&std::ffi::OsStr]) -> Vec<u8> {
    let output = Command::new(env!("CARGO_BIN_EXE_biors"))
        .args(args)
        .args(extra_args)
        .output()
        .expect("run biors");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    output.stdout
}

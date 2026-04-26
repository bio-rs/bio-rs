use serde_json::Value;
use std::fs;
use std::path::Path;

#[test]
fn machine_readable_schemas_are_valid_json() {
    let repo = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
    for schema in [
        "schemas/cli-success.v0.json",
        "schemas/cli-error.v0.json",
        "schemas/tokenize-output.v0.json",
        "schemas/inspect-output.v0.json",
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
    assert!(manifest["fixtures"][0]["input_hash"].is_string());
}

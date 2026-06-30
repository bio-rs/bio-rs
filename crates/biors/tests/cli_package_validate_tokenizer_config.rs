mod common;
mod package_support;

#[test]
fn package_validate_rejects_tokenizer_config_with_unknown_profile() {
    let value = package_support::validate_package_with_tokenizer_config(
        "invalid-tokenizer-profile",
        r#"{
  "profile": "bad",
  "add_special_tokens": false
}"#,
        None,
    );

    assert_eq!(value["error"]["code"], "package.invalid_tokenizer_config");
    package_support::assert_invalid_tokenizer_config_issue(&value, "unknown variant");
}

#[test]
fn package_validate_rejects_tokenizer_config_with_invalid_json_type() {
    let value = package_support::validate_package_with_tokenizer_config(
        "invalid-tokenizer-json-type",
        r#"{
  "profile": "protein-20",
  "add_special_tokens": "yes"
}"#,
        None,
    );

    assert_eq!(value["error"]["code"], "package.invalid_tokenizer_config");
    package_support::assert_invalid_tokenizer_config_issue(&value, "invalid tokenizer config JSON");
}

#[test]
fn package_validate_rejects_tokenizer_config_with_invalid_special_policy() {
    let value = package_support::validate_package_with_tokenizer_config(
        "invalid-tokenizer-special-policy",
        r#"{
  "profile": "protein-20-special",
  "add_special_tokens": false
}"#,
        Some(("protein-20-special", "protein-20-special.v0")),
    );

    assert_eq!(value["error"]["code"], "package.invalid_tokenizer_config");
    package_support::assert_invalid_tokenizer_config_issue(&value, "add_special_tokens");
}

#[test]
fn package_validate_rejects_tokenizer_manifest_profile_mismatch() {
    let value = package_support::validate_package_with_tokenizer_config(
        "invalid-tokenizer-manifest-mismatch",
        r#"{
  "profile": "protein-20-special",
  "add_special_tokens": true
}"#,
        None,
    );

    assert_eq!(value["error"]["code"], "package.invalid_tokenizer_config");
    package_support::assert_invalid_tokenizer_config_issue(&value, "tokenizer.name must match");
    package_support::assert_invalid_tokenizer_config_issue(
        &value,
        "tokenizer.contract_version must match",
    );
}

#[test]
fn package_validate_accepts_nucleotide_tokenizer_config() {
    let output = package_support::package_validate_with_tokenizer_config(
        "valid-dna-tokenizer-config",
        r#"{
  "profile": "dna-iupac",
  "add_special_tokens": false
}"#,
        Some(("dna-iupac", "dna-iupac.v0")),
    );

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn package_validate_accepts_rna_iupac_tokenizer_config() {
    let output = package_support::package_validate_with_tokenizer_config(
        "valid-rna-tokenizer-config",
        r#"{
  "profile": "rna-iupac",
  "add_special_tokens": false
}"#,
        Some(("rna-iupac", "rna-iupac.v0")),
    );

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn package_validate_rejects_nucleotide_kind_profile_mismatch() {
    let value = package_support::validate_package_with_pipeline_config(
        "invalid-nucleotide-kind-profile-mismatch",
        r#"schema_version = "biors.pipeline.v0"
name = "nucleotide-package-fixture-pipeline"

[input]
format = "fasta"
path = "../fixtures/tiny.fasta"

[normalize]
policy = "strip_ascii_whitespace_uppercase"

[validate]
kind = "dna"

[tokenize]
profile = "rna-iupac"

[export]
format = "model-input-json"
max_length = 8
pad_token_id = 0
padding = "fixed_length"
"#,
    );

    assert_eq!(value["error"]["code"], "package.invalid_pipeline_config");
    package_support::assert_invalid_pipeline_config_issue(
        &value,
        "validate.kind must be 'rna' for profile 'rna-iupac'",
    );
}

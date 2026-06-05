mod common;
mod package_support;

#[test]
fn package_validate_rejects_pipeline_config_with_zero_max_length() {
    let value = package_support::validate_package_with_pipeline_config(
        "invalid-pipeline-max-length",
        r#"schema_version = "biors.pipeline.v0"
name = "protein-package-fixture-pipeline"

[input]
format = "fasta"
path = "../fixtures/tiny.fasta"

[normalize]
policy = "strip_ascii_whitespace_uppercase"

[validate]
kind = "protein"

[tokenize]
profile = "protein-20"

[export]
format = "model-input-json"
max_length = 0
pad_token_id = 0
padding = "fixed_length"
"#,
    );

    assert_eq!(value["error"]["code"], "package.invalid_pipeline_config");
    package_support::assert_invalid_pipeline_config_issue(&value, "export.max_length");
}

#[test]
fn package_validate_rejects_pipeline_config_with_invalid_padding() {
    let value = package_support::validate_package_with_pipeline_config(
        "invalid-pipeline-padding",
        r#"schema_version = "biors.pipeline.v0"
name = "protein-package-fixture-pipeline"

[input]
format = "fasta"
path = "../fixtures/tiny.fasta"

[normalize]
policy = "strip_ascii_whitespace_uppercase"

[validate]
kind = "protein"

[tokenize]
profile = "protein-20"

[export]
format = "model-input-json"
max_length = 8
pad_token_id = 0
padding = "left"
"#,
    );

    assert_eq!(value["error"]["code"], "package.invalid_pipeline_config");
    package_support::assert_invalid_pipeline_config_issue(&value, "export.padding");
}

#[test]
fn package_validate_rejects_pipeline_config_with_unknown_field() {
    let value = package_support::validate_package_with_pipeline_config(
        "invalid-pipeline-unknown-field",
        r#"schema_version = "biors.pipeline.v0"
name = "protein-package-fixture-pipeline"
unexpected = true

[input]
format = "fasta"
path = "../fixtures/tiny.fasta"

[normalize]
policy = "strip_ascii_whitespace_uppercase"

[validate]
kind = "protein"

[tokenize]
profile = "protein-20"

[export]
format = "model-input-json"
max_length = 8
pad_token_id = 0
padding = "fixed_length"
"#,
    );

    assert_eq!(value["error"]["code"], "package.invalid_pipeline_config");
    package_support::assert_invalid_pipeline_config_issue(&value, "unknown field");
}

#[test]
fn package_validate_rejects_pipeline_config_with_absolute_input() {
    let temp = common::TempDir::new("absolute-pipeline-input-source");
    let external = temp.write("external.fasta", ">external\nACDE\n");
    let value = package_support::validate_package_with_pipeline_config(
        "invalid-pipeline-absolute-input",
        &package_support::valid_pipeline_config_with_input(&external.display().to_string()),
    );

    assert_eq!(value["error"]["code"], "package.invalid_pipeline_config");
    package_support::assert_invalid_pipeline_config_issue(&value, "package-relative");
}

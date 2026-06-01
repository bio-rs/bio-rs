use biors_core::package::{
    validate_package_manifest, DataShape, DataType, ModelArtifact, ModelFormat, PackageFixture,
    PackageManifest, PackageValidationIssueCode, PipelineStep, RuntimeBackend, RuntimeTarget,
    RuntimeTargetPlatform, SchemaVersion, TokenAsset,
};
use serde_json::Value;

mod common;

fn minimal_manifest() -> PackageManifest {
    PackageManifest {
        schema_version: SchemaVersion::BiorsPackageV0,
        name: "test-pkg".into(),
        package_layout: None,
        metadata: None,
        model: ModelArtifact {
            format: ModelFormat::Onnx,
            path: "model.onnx".into(),
            checksum: None,
            metadata: None,
        },
        tokenizer: None,
        vocab: None,
        preprocessing: vec![],
        postprocessing: vec![],
        runtime: RuntimeTarget {
            backend: RuntimeBackend::OnnxWebgpu,
            target: RuntimeTargetPlatform::BrowserWasmWebgpu,
            version: None,
        },
        expected_input: None,
        expected_output: None,
        fixtures: vec![PackageFixture {
            name: "fixture1".into(),
            input: "input.json".into(),
            expected_output: "output.json".into(),
            input_hash: None,
            expected_output_hash: None,
        }],
    }
}

#[test]
fn validate_package_manifest_accepts_minimal_valid_manifest() {
    let manifest = minimal_manifest();
    let report = validate_package_manifest(&manifest);
    assert!(report.valid);
    assert!(report.issues.is_empty());
    assert!(report.structured_issues.is_empty());
}

#[test]
fn validate_package_manifest_rejects_empty_model_metadata_name() {
    let mut manifest = minimal_manifest();
    manifest.model.metadata = Some(biors_core::package::ModelArtifactMetadata {
        name: " ".into(),
        version: None,
        architecture: None,
        task: None,
        source: None,
        description: None,
    });

    let report = validate_package_manifest(&manifest);

    assert!(!report.valid);
    assert!(report.structured_issues.iter().any(|issue| {
        issue.code == PackageValidationIssueCode::RequiredField
            && issue.field == "model.metadata.name"
    }));
}

#[test]
fn validate_package_manifest_rejects_empty_contract_identifiers() {
    let mut manifest = minimal_manifest();
    manifest.tokenizer = Some(TokenAsset {
        name: " ".into(),
        path: "tokenizers/protein-20.json".into(),
        checksum: None,
        contract_version: Some(" ".into()),
    });
    manifest.vocab = Some(TokenAsset {
        name: "".into(),
        path: "vocabs/protein-20.json".into(),
        checksum: None,
        contract_version: Some("".into()),
    });
    manifest.preprocessing = vec![PipelineStep {
        name: "".into(),
        implementation: " ".into(),
        contract: "".into(),
        contract_version: Some(" ".into()),
        config: None,
    }];

    let report = validate_package_manifest(&manifest);

    assert!(!report.valid);
    let fields: Vec<_> = report
        .structured_issues
        .iter()
        .filter(|issue| issue.code == PackageValidationIssueCode::RequiredField)
        .map(|issue| issue.field.as_str())
        .collect();
    for field in [
        "tokenizer.name",
        "tokenizer.contract_version",
        "vocab.name",
        "vocab.contract_version",
        "preprocessing[0].name",
        "preprocessing[0].implementation",
        "preprocessing[0].contract",
        "preprocessing[0].contract_version",
    ] {
        assert!(fields.contains(&field), "missing {field}: {fields:?}");
    }
}

#[test]
fn validate_package_manifest_rejects_missing_name() {
    let mut manifest = minimal_manifest();
    manifest.name = "".into();
    let report = validate_package_manifest(&manifest);
    assert!(!report.valid);
    assert_eq!(report.structured_issues.len(), 1);
    assert_eq!(
        report.structured_issues[0].code,
        PackageValidationIssueCode::RequiredField
    );
    assert_eq!(report.structured_issues[0].field, "name");
}

#[test]
fn validate_package_manifest_rejects_missing_model_path() {
    let mut manifest = minimal_manifest();
    manifest.model.path = "   ".into();
    let report = validate_package_manifest(&manifest);
    assert!(!report.valid);
    let model_issue = report
        .structured_issues
        .iter()
        .find(|i| i.field == "model.path")
        .expect("model.path issue expected");
    assert_eq!(model_issue.code, PackageValidationIssueCode::RequiredField);
}

#[test]
fn validate_package_manifest_rejects_empty_fixtures() {
    let mut manifest = minimal_manifest();
    manifest.fixtures.clear();
    let report = validate_package_manifest(&manifest);
    assert!(!report.valid);
    assert_eq!(report.structured_issues.len(), 1);
    assert_eq!(
        report.structured_issues[0].code,
        PackageValidationIssueCode::MissingFixture
    );
    assert_eq!(report.structured_issues[0].field, "fixtures");
}

#[test]
fn validate_package_manifest_rejects_fixture_with_missing_fields() {
    let mut manifest = minimal_manifest();
    manifest.fixtures = vec![PackageFixture {
        name: "".into(),
        input: "".into(),
        expected_output: "".into(),
        input_hash: None,
        expected_output_hash: None,
    }];
    let report = validate_package_manifest(&manifest);
    assert!(!report.valid);
    assert_eq!(report.structured_issues.len(), 3);
    let fields: Vec<&str> = report
        .structured_issues
        .iter()
        .map(|i| i.field.as_str())
        .collect();
    assert!(fields.contains(&"fixtures[0].name"));
    assert!(fields.contains(&"fixtures[0].input"));
    assert!(fields.contains(&"fixtures[0].expected_output"));
}

#[test]
fn validate_package_manifest_rejects_duplicate_fixture_names() {
    let mut manifest = minimal_manifest();
    manifest.fixtures.push(PackageFixture {
        name: "fixture1".into(),
        input: "input-2.json".into(),
        expected_output: "output-2.json".into(),
        input_hash: None,
        expected_output_hash: None,
    });
    let report = validate_package_manifest(&manifest);

    assert!(!report.valid);
    let issue = report
        .structured_issues
        .iter()
        .find(|issue| issue.code == PackageValidationIssueCode::DuplicateFixtureName)
        .expect("duplicate fixture issue");
    assert_eq!(issue.field, "fixtures[1].name");
    assert!(issue.message.contains("fixtures[0].name"));
}

#[test]
fn validate_package_manifest_rejects_duplicate_fixture_names_with_different_outputs() {
    let mut manifest = minimal_manifest();
    manifest.fixtures[0].expected_output = "expected-a.json".into();
    manifest.fixtures.push(PackageFixture {
        name: "fixture1".into(),
        input: "input-b.json".into(),
        expected_output: "expected-b.json".into(),
        input_hash: None,
        expected_output_hash: None,
    });
    let report = validate_package_manifest(&manifest);

    assert!(report.structured_issues.iter().any(|issue| {
        issue.code == PackageValidationIssueCode::DuplicateFixtureName
            && issue.field == "fixtures[1].name"
    }));
}

#[test]
fn validate_package_manifest_rejects_empty_shape() {
    let mut manifest = minimal_manifest();
    manifest.expected_input = Some(DataShape {
        shape: vec![],
        dtype: DataType::Float32,
    });
    manifest.expected_output = Some(DataShape {
        shape: vec![],
        dtype: DataType::Uint8,
    });
    let report = validate_package_manifest(&manifest);
    assert!(!report.valid);
    assert_eq!(report.structured_issues.len(), 2);
    let fields: Vec<&str> = report
        .structured_issues
        .iter()
        .map(|i| i.field.as_str())
        .collect();
    assert!(fields.contains(&"expected_input.shape"));
    assert!(fields.contains(&"expected_output.shape"));
}

#[test]
fn validate_package_manifest_accepts_non_empty_shape() {
    let mut manifest = minimal_manifest();
    manifest.expected_input = Some(DataShape {
        shape: vec!["1".into(), "256".into()],
        dtype: DataType::Float32,
    });
    let report = validate_package_manifest(&manifest);
    assert!(report.valid);
}

#[test]
fn validate_package_manifest_rejects_external_process_backend() {
    let mut manifest = minimal_manifest();
    manifest.runtime.backend = RuntimeBackend::ExternalProcess;
    manifest.runtime.target = RuntimeTargetPlatform::LocalCpu;

    let report = validate_package_manifest(&manifest);

    assert!(!report.valid);
    assert!(report.structured_issues.iter().any(|issue| {
        issue.code == PackageValidationIssueCode::UnsupportedRuntimeBackend
            && issue.field == "runtime.backend"
    }));
}

#[test]
fn package_manifest_deserialization_rejects_unknown_fields() {
    for (field_path, mutate) in [
        ("unexpected_top", add_top_unknown as fn(&mut Value)),
        ("model.unexpected_model", add_model_unknown),
        ("metadata.license.unexpected_license", add_metadata_unknown),
        ("fixtures[0].unexpected_fixture", add_fixture_unknown),
        ("runtime.unexpected_runtime", add_runtime_unknown),
        (
            "preprocessing[0].unexpected_step",
            add_pipeline_step_unknown,
        ),
    ] {
        let mut value = serde_json::to_value(common::example_manifest()).expect("manifest value");
        mutate(&mut value);

        let error = serde_json::from_value::<PackageManifest>(value).expect_err(field_path);
        assert!(
            error.to_string().contains("unknown field"),
            "{field_path}: {error}"
        );
    }
}

fn add_top_unknown(value: &mut Value) {
    value["unexpected_top"] = Value::Bool(true);
}

fn add_model_unknown(value: &mut Value) {
    value["model"]["unexpected_model"] = Value::Bool(true);
}

fn add_metadata_unknown(value: &mut Value) {
    value["metadata"]["license"]["unexpected_license"] = Value::Bool(true);
}

fn add_fixture_unknown(value: &mut Value) {
    value["fixtures"][0]["unexpected_fixture"] = Value::Bool(true);
}

fn add_runtime_unknown(value: &mut Value) {
    value["runtime"]["unexpected_runtime"] = Value::Bool(true);
}

fn add_pipeline_step_unknown(value: &mut Value) {
    value["preprocessing"][0]["unexpected_step"] = Value::Bool(true);
}

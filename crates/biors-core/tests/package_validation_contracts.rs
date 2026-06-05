use biors_core::package::{
    validate_package_manifest, PackageValidationIssueCode, PipelineStep, RuntimeBackend,
    RuntimeTargetPlatform, TokenAsset,
};

mod common;

#[test]
fn validate_package_manifest_rejects_empty_contract_identifiers() {
    let mut manifest = common::valid_manifest();
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
fn validate_package_manifest_rejects_empty_postprocessing_identifiers() {
    let mut manifest = common::valid_manifest();
    manifest.postprocessing = vec![PipelineStep {
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
        "postprocessing[0].name",
        "postprocessing[0].implementation",
        "postprocessing[0].contract",
        "postprocessing[0].contract_version",
    ] {
        assert!(fields.contains(&field), "missing {field}: {fields:?}");
    }
}

#[test]
fn validate_package_manifest_rejects_external_process_backend() {
    let mut manifest = common::valid_manifest();
    manifest.runtime.backend = RuntimeBackend::ExternalProcess;
    manifest.runtime.target = RuntimeTargetPlatform::LocalCpu;

    let report = validate_package_manifest(&manifest);

    assert!(!report.valid);
    assert!(report.structured_issues.iter().any(|issue| {
        issue.code == PackageValidationIssueCode::UnsupportedRuntimeBackend
            && issue.field == "runtime.backend"
    }));
}

use biors_core::package::{validate_package_manifest, PackageFixture, PackageValidationIssueCode};

mod common;

#[test]
fn validate_package_manifest_rejects_empty_fixtures() {
    let mut manifest = common::valid_manifest();
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
    let mut manifest = common::valid_manifest();
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
    let mut manifest = common::valid_manifest();
    manifest.fixtures.push(PackageFixture {
        name: "tiny-protein".into(),
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
    let mut manifest = common::valid_manifest();
    manifest.fixtures[0].expected_output = "expected-a.json".into();
    manifest.fixtures.push(PackageFixture {
        name: "tiny-protein".into(),
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
fn validate_package_manifest_rejects_whitespace_normalized_duplicate_fixture_names() {
    let mut manifest = common::valid_manifest();
    manifest.fixtures[0].name = " tiny-protein ".into();
    manifest.fixtures.push(PackageFixture {
        name: "tiny-protein".into(),
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

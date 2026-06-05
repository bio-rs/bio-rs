use biors_core::verification::{
    verify_package_outputs, FixtureObservation, VerificationIssueCode, VerificationStatus,
};

mod common;

#[test]
fn reports_mismatched_fixture_outputs() {
    let report = verify_package_outputs(
        &common::valid_manifest(),
        &[FixtureObservation {
            name: "tiny-protein".to_string(),
            path: "fixtures/tiny.fasta".to_string(),
        }],
        &common::example_base_dir(),
    );

    assert_eq!(report.passed, 0);
    assert_eq!(report.failed, 1);
    assert_eq!(report.results[0].status, VerificationStatus::Failed);
    assert!(report.results[0].content_mismatch);
    assert_eq!(
        report.results[0].issue_code,
        Some(VerificationIssueCode::OutputContentMismatch)
    );
    let diff = report.results[0]
        .content_diff
        .as_ref()
        .expect("content mismatch includes first difference");
    assert_eq!(diff.expected_path, "fixtures/tiny.output.json");
    assert_eq!(diff.observed_path, "fixtures/tiny.fasta");
    assert!(diff.first_difference.is_some());
}

#[test]
fn equivalent_json_with_different_bytes_fails_checksum_but_not_content() {
    let report = verify_package_outputs(
        &common::valid_manifest(),
        &[FixtureObservation {
            name: "tiny-protein".to_string(),
            path: "observed/tiny.reordered.json".to_string(),
        }],
        &common::example_base_dir(),
    );

    assert_eq!(report.passed, 0);
    assert_eq!(report.failed, 1);
    assert_eq!(report.results[0].status, VerificationStatus::Failed);
    assert!(report.results[0].checksum_mismatch);
    assert!(!report.results[0].content_mismatch);
    assert_eq!(
        report.results[0].issue_code,
        Some(VerificationIssueCode::OutputChecksumMismatch)
    );
}

#[test]
fn reports_expected_output_checksum_mismatch_before_observation_compare() {
    let mut manifest = common::valid_manifest();
    manifest.fixtures[0].expected_output_hash =
        Some("sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_string());

    let report = verify_package_outputs(
        &manifest,
        &[FixtureObservation {
            name: "tiny-protein".to_string(),
            path: "observed/tiny.output.json".to_string(),
        }],
        &common::example_base_dir(),
    );

    assert_eq!(report.passed, 0);
    assert_eq!(report.failed, 1);
    assert_eq!(report.results[0].status, VerificationStatus::Failed);
    assert_eq!(
        report.results[0].issue_code,
        Some(VerificationIssueCode::ExpectedOutputChecksumMismatch)
    );
    assert!(report.results[0]
        .issue
        .as_deref()
        .expect("issue")
        .contains("expected output hash mismatch"));
}

#[test]
fn reports_fixture_input_checksum_mismatch_before_expected_output_read() {
    let mut manifest = common::valid_manifest();
    manifest.fixtures[0].input_hash =
        Some("sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_string());

    let report = verify_package_outputs(
        &manifest,
        &[FixtureObservation {
            name: "tiny-protein".to_string(),
            path: "observed/tiny.output.json".to_string(),
        }],
        &common::example_base_dir(),
    );

    assert_eq!(report.passed, 0);
    assert_eq!(report.failed, 1);
    assert_eq!(report.results[0].status, VerificationStatus::Failed);
    assert_eq!(
        report.results[0].issue_code,
        Some(VerificationIssueCode::FixtureInputChecksumMismatch)
    );
    assert!(report.results[0]
        .issue
        .as_deref()
        .expect("issue")
        .contains("fixture input hash mismatch"));
}

#[test]
fn reports_invalid_observation_path() {
    let report = verify_package_outputs(
        &common::valid_manifest(),
        &[FixtureObservation {
            name: "tiny-protein".to_string(),
            path: "../outside.json".to_string(),
        }],
        &common::example_base_dir(),
    );

    assert_eq!(report.passed, 0);
    assert_eq!(report.failed, 1);
    assert_eq!(report.results[0].status, VerificationStatus::Missing);
    assert_eq!(
        report.results[0].issue_code,
        Some(VerificationIssueCode::ObservationPathInvalid)
    );
}

#[test]
fn reports_observed_output_read_failure() {
    let report = verify_package_outputs(
        &common::valid_manifest(),
        &[FixtureObservation {
            name: "tiny-protein".to_string(),
            path: "observed/not-present.json".to_string(),
        }],
        &common::example_base_dir(),
    );

    assert_eq!(report.passed, 0);
    assert_eq!(report.failed, 1);
    assert_eq!(report.results[0].status, VerificationStatus::Missing);
    assert_eq!(
        report.results[0].issue_code,
        Some(VerificationIssueCode::ObservedOutputReadFailed)
    );
}

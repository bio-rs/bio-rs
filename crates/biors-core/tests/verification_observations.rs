use biors_core::verification::{
    verify_package_outputs, FixtureObservation, VerificationIssueCode, VerificationStatus,
};

mod common;

#[test]
fn rejects_duplicate_fixture_observations() {
    let report = verify_package_outputs(
        &common::valid_manifest(),
        &[
            FixtureObservation {
                name: "tiny-protein".to_string(),
                path: "observed/tiny.output.json".to_string(),
            },
            FixtureObservation {
                name: "tiny-protein".to_string(),
                path: "observed/tiny.reordered.json".to_string(),
            },
        ],
        &common::example_base_dir(),
    );

    assert_eq!(report.passed, 0);
    assert_eq!(report.failed, 1);
    assert!(report.observation_issues.is_empty());
    assert_eq!(report.results[0].status, VerificationStatus::Failed);
    assert_eq!(
        report.results[0].issue_code,
        Some(VerificationIssueCode::DuplicateObservation)
    );
}

#[test]
fn rejects_unexpected_fixture_observations() {
    let report = verify_package_outputs(
        &common::valid_manifest(),
        &[
            FixtureObservation {
                name: "tiny-protein".to_string(),
                path: "observed/tiny.output.json".to_string(),
            },
            FixtureObservation {
                name: "stale-output".to_string(),
                path: "observed/stale.output.json".to_string(),
            },
        ],
        &common::example_base_dir(),
    );

    assert_eq!(report.passed, 1);
    assert_eq!(report.failed, 1);
    assert_eq!(report.observation_issues.len(), 1);
    assert_eq!(
        report.observation_issues[0].code,
        VerificationIssueCode::UnexpectedObservation
    );
    assert_eq!(report.observation_issues[0].name, "stale-output");
}

#[test]
fn rejects_duplicate_unexpected_fixture_observations() {
    let report = verify_package_outputs(
        &common::valid_manifest(),
        &[
            FixtureObservation {
                name: "tiny-protein".to_string(),
                path: "observed/tiny.output.json".to_string(),
            },
            FixtureObservation {
                name: "stale-output".to_string(),
                path: "observed/stale.output.json".to_string(),
            },
            FixtureObservation {
                name: "stale-output".to_string(),
                path: "observed/stale.reordered.json".to_string(),
            },
        ],
        &common::example_base_dir(),
    );

    assert_eq!(report.passed, 1);
    assert_eq!(report.failed, 1);
    assert_eq!(report.observation_issues.len(), 1);
    assert_eq!(
        report.observation_issues[0].code,
        VerificationIssueCode::DuplicateObservation
    );
    assert_eq!(report.observation_issues[0].name, "stale-output");
}

#[test]
fn reports_missing_fixture_outputs() {
    let report =
        verify_package_outputs(&common::valid_manifest(), &[], &common::example_base_dir());

    assert_eq!(report.passed, 0);
    assert_eq!(report.failed, 1);
    assert_eq!(report.results[0].status, VerificationStatus::Missing);
    assert_eq!(report.results[0].observed_output_path, None);
    assert_eq!(
        report.results[0].issue_code,
        Some(VerificationIssueCode::ObservationMissing)
    );
    assert_eq!(
        report.results[0].issue.as_deref(),
        Some("missing observation for fixture 'tiny-protein'")
    );
}

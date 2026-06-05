use biors_core::verification::{verify_package_outputs, FixtureObservation, VerificationStatus};

mod common;

#[test]
fn verifies_matching_fixture_outputs() {
    let report = verify_package_outputs(
        &common::valid_manifest(),
        &[FixtureObservation {
            name: "tiny-protein".to_string(),
            path: "observed/tiny.output.json".to_string(),
        }],
        &common::example_base_dir(),
    );

    assert_eq!(report.package, "protein-seed");
    assert_eq!(report.fixtures, 1);
    assert_eq!(report.passed, 1);
    assert_eq!(report.failed, 0);
    assert!(report.observation_issues.is_empty());
    assert_eq!(report.results[0].status, VerificationStatus::Passed);
    assert_eq!(
        report.results[0].expected_output_path,
        "fixtures/tiny.output.json"
    );
    assert_eq!(
        report.results[0].observed_output_path.as_deref(),
        Some("observed/tiny.output.json")
    );
    assert!(!report.results[0].checksum_mismatch);
    assert!(!report.results[0].content_mismatch);
}

use biors_core::{
    verify_package_outputs, FixtureObservation, PackageFixture, PackageManifest, RuntimeTarget,
    VerificationStatus,
};

fn manifest() -> PackageManifest {
    PackageManifest {
        schema_version: "biors.package.v0".to_string(),
        name: "protein-seed".to_string(),
        model: biors_core::ModelArtifact {
            format: "onnx".to_string(),
            path: "models/protein-seed.onnx".to_string(),
        },
        preprocessing: vec![],
        postprocessing: vec![],
        runtime: RuntimeTarget {
            backend: "onnx-webgpu".to_string(),
            target: "browser-wasm-webgpu".to_string(),
        },
        fixtures: vec![PackageFixture {
            name: "tiny-protein".to_string(),
            input: "fixtures/tiny.fasta".to_string(),
            expected_output: "fixtures/tiny.output.json".to_string(),
        }],
    }
}

#[test]
fn verifies_matching_fixture_outputs() {
    let report = verify_package_outputs(
        &manifest(),
        &[FixtureObservation {
            name: "tiny-protein".to_string(),
            output: "fixtures/tiny.output.json".to_string(),
        }],
    );

    assert_eq!(report.package, "protein-seed");
    assert_eq!(report.fixtures, 1);
    assert_eq!(report.passed, 1);
    assert_eq!(report.failed, 0);
    assert_eq!(report.results[0].status, VerificationStatus::Passed);
}

#[test]
fn reports_mismatched_fixture_outputs() {
    let report = verify_package_outputs(
        &manifest(),
        &[FixtureObservation {
            name: "tiny-protein".to_string(),
            output: "fixtures/other.output.json".to_string(),
        }],
    );

    assert_eq!(report.passed, 0);
    assert_eq!(report.failed, 1);
    assert_eq!(report.results[0].status, VerificationStatus::Failed);
    assert_eq!(
        report.results[0].issue,
        Some(
            "expected output 'fixtures/tiny.output.json' but observed 'fixtures/other.output.json'"
                .to_string()
        )
    );
}

#[test]
fn reports_missing_fixture_outputs() {
    let report = verify_package_outputs(&manifest(), &[]);

    assert_eq!(report.passed, 0);
    assert_eq!(report.failed, 1);
    assert_eq!(report.results[0].status, VerificationStatus::Missing);
    assert_eq!(
        report.results[0].issue,
        Some("missing observation for fixture 'tiny-protein'".to_string())
    );
}

use biors_core::{
    verify_package_outputs, FixtureObservation, ModelArtifact, ModelFormat, PackageFixture,
    PackageManifest, RuntimeBackend, RuntimeTarget, RuntimeTargetPlatform, SchemaVersion,
    VerificationStatus,
};

fn manifest() -> PackageManifest {
    PackageManifest {
        schema_version: SchemaVersion::BiorsPackageV0,
        name: "protein-seed".to_string(),
        model: ModelArtifact {
            format: ModelFormat::Onnx,
            path: "models/protein-seed.onnx".to_string(),
            checksum: None,
        },
        tokenizer: None,
        vocab: None,
        preprocessing: vec![],
        postprocessing: vec![],
        runtime: RuntimeTarget {
            backend: RuntimeBackend::OnnxWebgpu,
            target: RuntimeTargetPlatform::BrowserWasmWebgpu,
        },
        expected_input: None,
        expected_output: None,
        fixtures: vec![PackageFixture {
            name: "tiny-protein".to_string(),
            input: "fixtures/tiny.fasta".to_string(),
            expected_output: "fixtures/tiny.output.json".to_string(),
            input_hash: None,
            expected_output_hash: None,
        }],
    }
}

fn example_base_dir() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../examples/protein-package")
}

#[test]
fn verifies_matching_fixture_outputs() {
    let report = verify_package_outputs(
        &manifest(),
        &[FixtureObservation {
            name: "tiny-protein".to_string(),
            path: "observed/tiny.output.json".to_string(),
        }],
        &example_base_dir(),
    );

    assert_eq!(report.package, "protein-seed");
    assert_eq!(report.fixtures, 1);
    assert_eq!(report.passed, 1);
    assert_eq!(report.failed, 0);
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

#[test]
fn reports_mismatched_fixture_outputs() {
    let report = verify_package_outputs(
        &manifest(),
        &[FixtureObservation {
            name: "tiny-protein".to_string(),
            path: "fixtures/tiny.fasta".to_string(),
        }],
        &example_base_dir(),
    );

    assert_eq!(report.passed, 0);
    assert_eq!(report.failed, 1);
    assert_eq!(report.results[0].status, VerificationStatus::Failed);
    assert!(report.results[0].content_mismatch);
}

#[test]
fn treats_equivalent_json_with_different_key_order_as_match() {
    let report = verify_package_outputs(
        &manifest(),
        &[FixtureObservation {
            name: "tiny-protein".to_string(),
            path: "observed/tiny.reordered.json".to_string(),
        }],
        &example_base_dir(),
    );

    assert_eq!(report.passed, 1);
    assert_eq!(report.failed, 0);
    assert_eq!(report.results[0].status, VerificationStatus::Passed);
}

#[test]
fn reports_missing_fixture_outputs() {
    let report = verify_package_outputs(&manifest(), &[], &example_base_dir());

    assert_eq!(report.passed, 0);
    assert_eq!(report.failed, 1);
    assert_eq!(report.results[0].status, VerificationStatus::Missing);
    assert_eq!(report.results[0].observed_output_path, None);
}

#[test]
fn computes_stable_fixture_hashes() {
    assert_eq!(
        biors_core::stable_input_hash(">seq1\nACDE\n"),
        "fnv1a64:08a331cb13c7bd72"
    );
}

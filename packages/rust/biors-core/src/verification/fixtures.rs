use super::diff::{content_mismatch_diff, contents_match};
use super::{
    FixtureObservation, FixtureVerificationResult, PackageVerificationReport,
    VerificationIssueCode, VerificationStatus,
};
use crate::hash::sha256_digest;
use crate::package::{
    read_package_file, resolve_package_asset_path, PackageFixture, PackageManifest,
};
use std::path::Path;

/// Verify package outputs using the package directory for both manifest assets and observations.
pub fn verify_package_outputs(
    manifest: &PackageManifest,
    observations: &[FixtureObservation],
    manifest_base_dir: &Path,
) -> PackageVerificationReport {
    verify_package_outputs_with_observation_base(
        manifest,
        observations,
        manifest_base_dir,
        manifest_base_dir,
    )
}

/// Verify package outputs using separate base directories for manifest assets and observations.
pub fn verify_package_outputs_with_observation_base(
    manifest: &PackageManifest,
    observations: &[FixtureObservation],
    manifest_base_dir: &Path,
    observations_base_dir: &Path,
) -> PackageVerificationReport {
    let results: Vec<_> = manifest
        .fixtures
        .iter()
        .map(|fixture| {
            verify_fixture(
                fixture,
                observations
                    .iter()
                    .find(|candidate| candidate.name == fixture.name),
                manifest_base_dir,
                observations_base_dir,
            )
        })
        .collect();

    let passed = results
        .iter()
        .filter(|result| result.status == VerificationStatus::Passed)
        .count();

    PackageVerificationReport {
        package: manifest.name.clone(),
        fixtures: manifest.fixtures.len(),
        passed,
        failed: manifest.fixtures.len() - passed,
        results,
    }
}

fn verify_fixture(
    fixture: &PackageFixture,
    observation: Option<&FixtureObservation>,
    manifest_base_dir: &Path,
    observations_base_dir: &Path,
) -> FixtureVerificationResult {
    let mut result = FixtureVerificationResult::new(fixture, observation);

    let input_bytes = match read_package_file(manifest_base_dir, &fixture.input) {
        Ok(bytes) => bytes,
        Err(error) => {
            return result.failed(
                VerificationIssueCode::FixtureInputReadFailed,
                error.to_string(),
            );
        }
    };

    if let Some(error) = validate_declared_hash(
        &fixture.input,
        &input_bytes,
        fixture.input_hash.as_deref(),
        VerificationIssueCode::FixtureInputChecksumMismatch,
        "fixture input hash mismatch",
    ) {
        return result.failed_with_checksum(error.0, error.1);
    }

    let expected_bytes = match read_package_file(manifest_base_dir, &fixture.expected_output) {
        Ok(bytes) => bytes,
        Err(error) => {
            return result.failed(
                VerificationIssueCode::ExpectedOutputReadFailed,
                error.to_string(),
            );
        }
    };

    let expected_output_hash = sha256_digest(&expected_bytes);
    result.expected_output_hash = Some(expected_output_hash.clone());

    if let Some(error) = validate_declared_hash(
        &fixture.expected_output,
        &expected_bytes,
        fixture.expected_output_hash.as_deref(),
        VerificationIssueCode::ExpectedOutputChecksumMismatch,
        "expected output hash mismatch",
    ) {
        return result.failed_with_checksum(error.0, error.1);
    }

    let Some(observation) = observation else {
        return result.missing(
            VerificationIssueCode::ObservationMissing,
            format!("missing observation for fixture '{}'", fixture.name),
        );
    };

    let observed_bytes = match read_observed_output(observation, observations_base_dir) {
        Ok(bytes) => bytes,
        Err((code, issue)) => return result.missing(code, issue),
    };

    compare_observed_output(
        result,
        fixture,
        observation,
        &expected_bytes,
        &expected_output_hash,
        &observed_bytes,
    )
}

fn validate_declared_hash(
    path: &str,
    bytes: &[u8],
    declared_hash: Option<&str>,
    code: VerificationIssueCode,
    label: &str,
) -> Option<(VerificationIssueCode, String)> {
    let declared_hash = declared_hash?;
    let actual_hash = sha256_digest(bytes);
    if actual_hash == declared_hash {
        return None;
    }

    Some((
        code,
        format!("{label} for '{path}': expected '{declared_hash}' but computed '{actual_hash}'"),
    ))
}

fn read_observed_output(
    observation: &FixtureObservation,
    observations_base_dir: &Path,
) -> Result<Vec<u8>, (VerificationIssueCode, String)> {
    let observed_path = resolve_package_asset_path(observations_base_dir, &observation.path)
        .map_err(|error| {
            (
                VerificationIssueCode::ObservationPathInvalid,
                error.to_string(),
            )
        })?;

    std::fs::read(&observed_path).map_err(|error| {
        (
            VerificationIssueCode::ObservedOutputReadFailed,
            format!(
                "failed to read observed output '{}' at '{}': {error}",
                observation.path,
                observed_path.display()
            ),
        )
    })
}

fn compare_observed_output(
    mut result: FixtureVerificationResult,
    fixture: &PackageFixture,
    observation: &FixtureObservation,
    expected_bytes: &[u8],
    expected_output_hash: &str,
    observed_bytes: &[u8],
) -> FixtureVerificationResult {
    let observed_hash = sha256_digest(observed_bytes);
    result.observed_output_hash = Some(observed_hash.clone());

    if observed_hash != expected_output_hash {
        result.mark_checksum_mismatch(VerificationIssueCode::OutputChecksumMismatch);
    }

    if !contents_match(expected_bytes, observed_bytes) {
        result.mark_content_mismatch(
            VerificationIssueCode::OutputContentMismatch,
            content_mismatch_diff(
                &fixture.expected_output,
                &observation.path,
                expected_bytes,
                observed_bytes,
            ),
        );
    }

    result.finish_output_compare(
        expected_output_hash,
        &observed_hash,
        &fixture.expected_output,
        &observation.path,
    )
}

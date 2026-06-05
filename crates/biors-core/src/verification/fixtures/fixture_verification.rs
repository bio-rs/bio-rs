use super::super::types::{FixtureVerificationResult, VerificationIssueCode};
use super::checksums::validate_declared_hash;
use super::observations::FixtureObservationMatch;
use super::output_comparison::{compare_observed_output, read_observed_output};
use crate::hash::sha256_bytes_digest;
use crate::package::{read_package_file, PackageFixture};
use std::path::Path;

pub(super) fn verify_fixture(
    fixture: &PackageFixture,
    observation: FixtureObservationMatch<'_>,
    manifest_base_dir: &Path,
    observations_base_dir: &Path,
) -> FixtureVerificationResult {
    let result_observation = match observation {
        FixtureObservationMatch::Unique(observation)
        | FixtureObservationMatch::Duplicate(observation) => Some(observation),
        FixtureObservationMatch::Missing => None,
    };
    let mut result = FixtureVerificationResult::new(fixture, result_observation);

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

    let expected_output_hash = sha256_bytes_digest(&expected_bytes);
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

    let observation = match observation {
        FixtureObservationMatch::Missing => {
            return result.missing(
                VerificationIssueCode::ObservationMissing,
                format!("missing observation for fixture '{}'", fixture.name),
            );
        }
        FixtureObservationMatch::Duplicate(_) => {
            return result.failed(
                VerificationIssueCode::DuplicateObservation,
                format!("duplicate observations for fixture '{}'", fixture.name),
            );
        }
        FixtureObservationMatch::Unique(observation) => observation,
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

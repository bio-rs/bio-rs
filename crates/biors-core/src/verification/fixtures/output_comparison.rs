use super::super::diff::{content_mismatch_diff, contents_match};
use super::super::types::{FixtureObservation, FixtureVerificationResult, VerificationIssueCode};
use crate::hash::sha256_bytes_digest;
use crate::package::{resolve_package_asset_path, PackageFixture};
use std::path::Path;

pub(super) fn read_observed_output(
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

pub(super) fn compare_observed_output(
    mut result: FixtureVerificationResult,
    fixture: &PackageFixture,
    observation: &FixtureObservation,
    expected_bytes: &[u8],
    expected_output_hash: &str,
    observed_bytes: &[u8],
) -> FixtureVerificationResult {
    let observed_hash = sha256_bytes_digest(observed_bytes);
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

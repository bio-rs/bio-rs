mod checksums;
mod fixture_verification;
mod observations;
mod output_comparison;

use super::types::{FixtureObservation, PackageVerificationReport, VerificationStatus};
use crate::package::PackageManifest;
use fixture_verification::verify_fixture;
use observations::{index_observations, observation_for_fixture, unexpected_observation_issues};
use std::collections::HashSet;
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
    let observation_index = index_observations(observations);
    let fixture_names: HashSet<_> = manifest
        .fixtures
        .iter()
        .map(|fixture| fixture.name.as_str())
        .collect();
    let observation_issues = unexpected_observation_issues(&observation_index, &fixture_names);

    let results: Vec<_> = manifest
        .fixtures
        .iter()
        .map(|fixture| {
            verify_fixture(
                fixture,
                observation_for_fixture(fixture, &observation_index),
                manifest_base_dir,
                observations_base_dir,
            )
        })
        .collect();

    let passed = results
        .iter()
        .filter(|result| result.status == VerificationStatus::Passed)
        .count();
    let failed = results.len() - passed + observation_issues.len();

    PackageVerificationReport {
        package: manifest.name.clone(),
        fixtures: manifest.fixtures.len(),
        passed,
        failed,
        results,
        observation_issues,
    }
}

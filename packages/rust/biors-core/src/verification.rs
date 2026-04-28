use crate::package::{
    read_package_file, resolve_package_asset_path, sha256_digest, PackageManifest,
};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FixtureObservation {
    pub name: String,
    pub path: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PackageVerificationReport {
    pub package: String,
    pub fixtures: usize,
    pub passed: usize,
    pub failed: usize,
    pub results: Vec<FixtureVerificationResult>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FixtureVerificationResult {
    pub name: String,
    pub input_path: String,
    pub expected_output_path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub observed_output_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expected_output_hash: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub observed_output_hash: Option<String>,
    pub status: VerificationStatus,
    pub checksum_mismatch: bool,
    pub content_mismatch: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub issue: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VerificationStatus {
    Failed,
    Missing,
    Passed,
}

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
            let observation = observations
                .iter()
                .find(|candidate| candidate.name == fixture.name);

            let mut result = FixtureVerificationResult {
                name: fixture.name.clone(),
                input_path: fixture.input.clone(),
                expected_output_path: fixture.expected_output.clone(),
                observed_output_path: observation.map(|item| item.path.clone()),
                expected_output_hash: fixture.expected_output_hash.clone(),
                observed_output_hash: None,
                status: VerificationStatus::Passed,
                checksum_mismatch: false,
                content_mismatch: false,
                issue: None,
            };

            let input_bytes = match read_package_file(manifest_base_dir, &fixture.input) {
                Ok(bytes) => bytes,
                Err(error) => {
                    result.status = VerificationStatus::Failed;
                    result.issue = Some(error);
                    return result;
                }
            };

            if let Some(expected_input_hash) = &fixture.input_hash {
                let input_hash = sha256_digest(&input_bytes);
                if &input_hash != expected_input_hash {
                    result.status = VerificationStatus::Failed;
                    result.checksum_mismatch = true;
                    result.issue = Some(format!(
                        "fixture input hash mismatch for '{}': expected '{}' but computed '{}'",
                        fixture.input, expected_input_hash, input_hash
                    ));
                    return result;
                }
            }

            let expected_bytes =
                match read_package_file(manifest_base_dir, &fixture.expected_output) {
                    Ok(bytes) => bytes,
                    Err(error) => {
                        result.status = VerificationStatus::Failed;
                        result.issue = Some(error);
                        return result;
                    }
                };

            let expected_output_hash = sha256_digest(&expected_bytes);
            result.expected_output_hash = Some(expected_output_hash.clone());

            if let Some(declared_output_hash) = &fixture.expected_output_hash {
                if &expected_output_hash != declared_output_hash {
                    result.status = VerificationStatus::Failed;
                    result.checksum_mismatch = true;
                    result.issue = Some(format!(
                        "expected output hash mismatch for '{}': expected '{}' but computed '{}'",
                        fixture.expected_output, declared_output_hash, expected_output_hash
                    ));
                    return result;
                }
            }

            let Some(observation) = observation else {
                result.status = VerificationStatus::Missing;
                result.issue = Some(format!(
                    "missing observation for fixture '{}'",
                    fixture.name
                ));
                return result;
            };

            let observed_path =
                match resolve_package_asset_path(observations_base_dir, &observation.path) {
                    Ok(path) => path,
                    Err(error) => {
                        result.status = VerificationStatus::Missing;
                        result.issue = Some(error);
                        return result;
                    }
                };
            let observed_bytes = match std::fs::read(&observed_path) {
                Ok(bytes) => bytes,
                Err(error) => {
                    result.status = VerificationStatus::Missing;
                    result.issue = Some(format!(
                        "failed to read observed output '{}' at '{}': {error}",
                        observation.path,
                        observed_path.display()
                    ));
                    return result;
                }
            };

            let observed_hash = sha256_digest(&observed_bytes);
            result.observed_output_hash = Some(observed_hash.clone());

            if observed_hash != expected_output_hash {
                result.status = VerificationStatus::Failed;
                result.checksum_mismatch = true;
            }

            if !contents_match(&expected_bytes, &observed_bytes) {
                result.status = VerificationStatus::Failed;
                result.content_mismatch = true;
            }

            if result.status == VerificationStatus::Failed {
                let mut issues = Vec::new();
                if result.checksum_mismatch {
                    issues.push(format!(
                        "output checksum mismatch: expected '{}' but observed '{}'",
                        expected_output_hash, observed_hash
                    ));
                }
                if result.content_mismatch {
                    issues.push(format!(
                        "output content mismatch between '{}' and '{}'",
                        fixture.expected_output, observation.path
                    ));
                }
                result.issue = Some(issues.join("; "));
            }

            result
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

pub fn stable_input_hash(input: &str) -> String {
    let mut hash = 0xcbf29ce484222325u64;
    for byte in input.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("fnv1a64:{hash:016x}")
}

fn contents_match(expected: &[u8], observed: &[u8]) -> bool {
    match (
        serde_json::from_slice::<serde_json::Value>(expected),
        serde_json::from_slice::<serde_json::Value>(observed),
    ) {
        (Ok(expected), Ok(observed)) => expected == observed,
        _ => expected == observed,
    }
}

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
    pub issue_code: Option<VerificationIssueCode>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_diff: Option<ContentMismatchDiff>,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VerificationIssueCode {
    ExpectedOutputChecksumMismatch,
    ExpectedOutputReadFailed,
    FixtureInputChecksumMismatch,
    FixtureInputReadFailed,
    ObservationMissing,
    ObservationPathInvalid,
    ObservedOutputReadFailed,
    OutputChecksumMismatch,
    OutputContentMismatch,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContentMismatchDiff {
    pub expected_path: String,
    pub observed_path: String,
    pub expected_len: usize,
    pub observed_len: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first_difference: Option<FirstDifference>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FirstDifference {
    pub byte_offset: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expected_byte: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub observed_byte: Option<u8>,
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
                issue_code: None,
                content_diff: None,
                issue: None,
            };

            let input_bytes = match read_package_file(manifest_base_dir, &fixture.input) {
                Ok(bytes) => bytes,
                Err(error) => {
                    result.status = VerificationStatus::Failed;
                    result.issue_code = Some(VerificationIssueCode::FixtureInputReadFailed);
                    result.issue = Some(error);
                    return result;
                }
            };

            if let Some(expected_input_hash) = &fixture.input_hash {
                let input_hash = sha256_digest(&input_bytes);
                if &input_hash != expected_input_hash {
                    result.status = VerificationStatus::Failed;
                    result.checksum_mismatch = true;
                    result.issue_code = Some(VerificationIssueCode::FixtureInputChecksumMismatch);
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
                        result.issue_code = Some(VerificationIssueCode::ExpectedOutputReadFailed);
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
                    result.issue_code = Some(VerificationIssueCode::ExpectedOutputChecksumMismatch);
                    result.issue = Some(format!(
                        "expected output hash mismatch for '{}': expected '{}' but computed '{}'",
                        fixture.expected_output, declared_output_hash, expected_output_hash
                    ));
                    return result;
                }
            }

            let Some(observation) = observation else {
                result.status = VerificationStatus::Missing;
                result.issue_code = Some(VerificationIssueCode::ObservationMissing);
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
                        result.issue_code = Some(VerificationIssueCode::ObservationPathInvalid);
                        result.issue = Some(error);
                        return result;
                    }
                };
            let observed_bytes = match std::fs::read(&observed_path) {
                Ok(bytes) => bytes,
                Err(error) => {
                    result.status = VerificationStatus::Missing;
                    result.issue_code = Some(VerificationIssueCode::ObservedOutputReadFailed);
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
                result.issue_code = Some(VerificationIssueCode::OutputChecksumMismatch);
            }

            if !contents_match(&expected_bytes, &observed_bytes) {
                result.status = VerificationStatus::Failed;
                result.content_mismatch = true;
                result.issue_code = Some(VerificationIssueCode::OutputContentMismatch);
                result.content_diff = Some(content_mismatch_diff(
                    &fixture.expected_output,
                    &observation.path,
                    &expected_bytes,
                    &observed_bytes,
                ));
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
    let mut hasher = StableInputHasher::new();
    hasher.update(input.as_bytes());
    hasher.finalize()
}

#[derive(Debug, Clone, Copy)]
pub struct StableInputHasher {
    hash: u64,
}

impl StableInputHasher {
    pub const fn new() -> Self {
        Self {
            hash: 0xcbf29ce484222325,
        }
    }

    pub fn update(&mut self, bytes: &[u8]) {
        for byte in bytes {
            self.hash ^= u64::from(*byte);
            self.hash = self.hash.wrapping_mul(0x100000001b3);
        }
    }

    pub fn finalize(self) -> String {
        let hash = self.hash;
        format!("fnv1a64:{hash:016x}")
    }
}

impl Default for StableInputHasher {
    fn default() -> Self {
        Self::new()
    }
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

fn content_mismatch_diff(
    expected_path: &str,
    observed_path: &str,
    expected: &[u8],
    observed: &[u8],
) -> ContentMismatchDiff {
    let expected = canonical_content_bytes(expected);
    let observed = canonical_content_bytes(observed);
    ContentMismatchDiff {
        expected_path: expected_path.to_string(),
        observed_path: observed_path.to_string(),
        expected_len: expected.len(),
        observed_len: observed.len(),
        first_difference: first_difference(&expected, &observed),
    }
}

fn canonical_content_bytes(bytes: &[u8]) -> Vec<u8> {
    match serde_json::from_slice::<serde_json::Value>(bytes) {
        Ok(json) => serde_json::to_vec(&json).unwrap_or_else(|_| bytes.to_vec()),
        Err(_) => bytes.to_vec(),
    }
}

fn first_difference(expected: &[u8], observed: &[u8]) -> Option<FirstDifference> {
    let shared_len = expected.len().min(observed.len());
    for index in 0..shared_len {
        if expected[index] != observed[index] {
            return Some(FirstDifference {
                byte_offset: index,
                expected_byte: Some(expected[index]),
                observed_byte: Some(observed[index]),
            });
        }
    }

    if expected.len() != observed.len() {
        return Some(FirstDifference {
            byte_offset: shared_len,
            expected_byte: expected.get(shared_len).copied(),
            observed_byte: observed.get(shared_len).copied(),
        });
    }

    None
}

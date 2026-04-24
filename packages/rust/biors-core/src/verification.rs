use crate::PackageManifest;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FixtureObservation {
    pub name: String,
    pub output: String,
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
    pub input: String,
    pub expected_output: String,
    pub observed_output: Option<String>,
    pub status: VerificationStatus,
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
) -> PackageVerificationReport {
    let results: Vec<_> = manifest
        .fixtures
        .iter()
        .map(|fixture| {
            let observation = observations
                .iter()
                .find(|candidate| candidate.name == fixture.name);

            match observation {
                Some(observation) if observation.output == fixture.expected_output => {
                    FixtureVerificationResult {
                        name: fixture.name.clone(),
                        input: fixture.input.clone(),
                        expected_output: fixture.expected_output.clone(),
                        observed_output: Some(observation.output.clone()),
                        status: VerificationStatus::Passed,
                        issue: None,
                    }
                }
                Some(observation) => FixtureVerificationResult {
                    name: fixture.name.clone(),
                    input: fixture.input.clone(),
                    expected_output: fixture.expected_output.clone(),
                    observed_output: Some(observation.output.clone()),
                    status: VerificationStatus::Failed,
                    issue: Some(format!(
                        "expected output '{}' but observed '{}'",
                        fixture.expected_output, observation.output
                    )),
                },
                None => FixtureVerificationResult {
                    name: fixture.name.clone(),
                    input: fixture.input.clone(),
                    expected_output: fixture.expected_output.clone(),
                    observed_output: None,
                    status: VerificationStatus::Missing,
                    issue: Some(format!(
                        "missing observation for fixture '{}'",
                        fixture.name
                    )),
                },
            }
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

use serde::{Deserialize, Serialize};

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

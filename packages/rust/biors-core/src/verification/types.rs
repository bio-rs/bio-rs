use serde::{Deserialize, Serialize};

use crate::package::PackageFixture;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Observed output path for one named package fixture.
pub struct FixtureObservation {
    /// Fixture name matching `PackageFixture::name`.
    pub name: String,
    /// Observation path relative to the observations base directory.
    pub path: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Aggregate verification report for all fixtures in a package.
pub struct PackageVerificationReport {
    /// Package name from the manifest.
    pub package: String,
    /// Total number of manifest fixtures.
    pub fixtures: usize,
    /// Number of fixtures that passed.
    pub passed: usize,
    /// Number of fixtures that failed or were missing.
    pub failed: usize,
    /// Per-fixture verification results.
    pub results: Vec<FixtureVerificationResult>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Verification result for a single package fixture.
pub struct FixtureVerificationResult {
    /// Fixture name.
    pub name: String,
    /// Manifest-declared fixture input path.
    pub input_path: String,
    /// Manifest-declared expected output path.
    pub expected_output_path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Observation path when one was supplied.
    pub observed_output_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Computed or manifest-declared expected output hash.
    pub expected_output_hash: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Computed observed output hash.
    pub observed_output_hash: Option<String>,
    /// Verification status.
    pub status: VerificationStatus,
    /// True when a checksum comparison failed.
    pub checksum_mismatch: bool,
    /// True when canonical output content comparison failed.
    pub content_mismatch: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Stable machine-readable issue code.
    pub issue_code: Option<VerificationIssueCode>,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// First-difference metadata for content mismatches.
    pub content_diff: Option<ContentMismatchDiff>,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Human-readable issue summary.
    pub issue: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
/// Verification status for a fixture.
pub enum VerificationStatus {
    /// Fixture was processed but failed validation.
    Failed,
    /// Required observation or observed file was missing.
    Missing,
    /// Fixture matched expected output.
    Passed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
/// Stable machine-readable verification issue codes.
pub enum VerificationIssueCode {
    /// Expected output checksum declared in the manifest did not match the file.
    ExpectedOutputChecksumMismatch,
    /// Expected output file could not be read.
    ExpectedOutputReadFailed,
    /// Fixture input checksum declared in the manifest did not match the file.
    FixtureInputChecksumMismatch,
    /// Fixture input file could not be read.
    FixtureInputReadFailed,
    /// No observation was supplied for the fixture.
    ObservationMissing,
    /// Observation path was invalid.
    ObservationPathInvalid,
    /// Observed output file could not be read.
    ObservedOutputReadFailed,
    /// Observed output hash differed from expected output hash.
    OutputChecksumMismatch,
    /// Observed output content differed from expected output content.
    OutputContentMismatch,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Canonical content mismatch metadata for expected and observed outputs.
pub struct ContentMismatchDiff {
    /// Expected output path.
    pub expected_path: String,
    /// Observed output path.
    pub observed_path: String,
    /// Canonical expected content length in bytes.
    pub expected_len: usize,
    /// Canonical observed content length in bytes.
    pub observed_len: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// First differing byte if a byte-level difference can be identified.
    pub first_difference: Option<FirstDifference>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// First differing byte between expected and observed canonical content.
pub struct FirstDifference {
    /// Zero-based byte offset of the first difference.
    pub byte_offset: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Expected byte at the differing offset, if present.
    pub expected_byte: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Observed byte at the differing offset, if present.
    pub observed_byte: Option<u8>,
}

impl FixtureVerificationResult {
    pub(crate) fn new(fixture: &PackageFixture, observation: Option<&FixtureObservation>) -> Self {
        Self {
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
        }
    }

    pub(crate) fn failed(mut self, code: VerificationIssueCode, issue: String) -> Self {
        self.status = VerificationStatus::Failed;
        self.issue_code = Some(code);
        self.issue = Some(issue);
        self
    }

    pub(crate) fn failed_with_checksum(
        mut self,
        code: VerificationIssueCode,
        issue: String,
    ) -> Self {
        self.checksum_mismatch = true;
        self.failed(code, issue)
    }

    pub(crate) fn missing(mut self, code: VerificationIssueCode, issue: String) -> Self {
        self.status = VerificationStatus::Missing;
        self.issue_code = Some(code);
        self.issue = Some(issue);
        self
    }

    pub(crate) fn mark_checksum_mismatch(&mut self, code: VerificationIssueCode) {
        self.status = VerificationStatus::Failed;
        self.checksum_mismatch = true;
        self.issue_code = Some(code);
    }

    pub(crate) fn mark_content_mismatch(
        &mut self,
        code: VerificationIssueCode,
        diff: ContentMismatchDiff,
    ) {
        self.status = VerificationStatus::Failed;
        self.content_mismatch = true;
        self.issue_code = Some(code);
        self.content_diff = Some(diff);
    }

    pub(crate) fn finish_output_compare(
        mut self,
        expected_hash: &str,
        observed_hash: &str,
        expected_path: &str,
        observed_path: &str,
    ) -> Self {
        if self.status != VerificationStatus::Failed {
            return self;
        }

        let mut issues = Vec::new();
        if self.checksum_mismatch {
            issues.push(format!(
                "output checksum mismatch: expected '{expected_hash}' but observed '{observed_hash}'"
            ));
        }
        if self.content_mismatch {
            issues.push(format!(
                "output content mismatch between '{expected_path}' and '{observed_path}'"
            ));
        }
        self.issue = Some(issues.join("; "));
        self
    }
}

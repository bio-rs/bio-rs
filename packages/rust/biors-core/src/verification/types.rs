use serde::{Deserialize, Serialize};

use crate::package::PackageFixture;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OutputDiffReport {
    pub expected_path: String,
    pub observed_path: String,
    pub expected_sha256: String,
    pub observed_sha256: String,
    pub matches: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_diff: Option<ContentMismatchDiff>,
}

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

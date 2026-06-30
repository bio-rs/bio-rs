use biors_core::{
    package::{PackageValidationIssueCode, PackageValidationReport},
    verification::{PackageVerificationReport, VerificationIssueCode, VerificationStatus},
};

pub(crate) fn classify_validation_code(report: &PackageValidationReport) -> &'static str {
    let has_issue = |code| {
        report
            .structured_issues
            .iter()
            .any(|issue| issue.code == code)
    };
    if has_issue(PackageValidationIssueCode::InvalidChecksumFormat) {
        "package.invalid_checksum_format"
    } else if has_issue(PackageValidationIssueCode::ChecksumMismatch) {
        "package.checksum_mismatch"
    } else if has_issue(PackageValidationIssueCode::InvalidAssetPath) {
        "package.invalid_asset_path"
    } else if has_issue(PackageValidationIssueCode::LayoutMismatch) {
        "package.layout_mismatch"
    } else if has_issue(PackageValidationIssueCode::AssetReadFailed) {
        "package.asset_read_failed"
    } else if has_issue(PackageValidationIssueCode::InvalidPipelineConfig) {
        "package.invalid_pipeline_config"
    } else if has_issue(PackageValidationIssueCode::InvalidTokenizerConfig) {
        "package.invalid_tokenizer_config"
    } else if has_issue(PackageValidationIssueCode::InvalidVocabConfig) {
        "package.invalid_vocab_config"
    } else {
        "package.validation_failed"
    }
}

pub(crate) fn classify_verification_code(report: &PackageVerificationReport) -> &'static str {
    let has_observation_issue = |code| {
        report
            .observation_issues
            .iter()
            .any(|issue| issue.code == code)
    };
    let has_result_issue = |code| {
        report
            .results
            .iter()
            .any(|result| result.issue_code == Some(code))
    };

    if has_observation_issue(VerificationIssueCode::DuplicateObservation)
        || has_result_issue(VerificationIssueCode::DuplicateObservation)
    {
        "package.duplicate_observation"
    } else if has_observation_issue(VerificationIssueCode::UnexpectedObservation) {
        "package.unexpected_observation"
    } else if has_result_issue(VerificationIssueCode::ObservationMissing) {
        "package.observed_output_missing"
    } else if has_result_issue(VerificationIssueCode::ObservationPathInvalid) {
        "package.invalid_asset_path"
    } else if has_result_issue(VerificationIssueCode::ObservedOutputReadFailed)
        || report
            .results
            .iter()
            .any(|result| matches!(result.status, VerificationStatus::Missing))
    {
        "package.observed_output_missing"
    } else if report.results.iter().any(|result| result.content_mismatch) {
        "package.output_content_mismatch"
    } else if report.results.iter().any(|result| result.checksum_mismatch) {
        "package.checksum_mismatch"
    } else {
        "package.verification_failed"
    }
}

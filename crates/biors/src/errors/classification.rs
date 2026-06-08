use biors_core::{
    package::{PackageValidationIssueCode, PackageValidationReport},
    verification::{PackageVerificationReport, VerificationIssueCode, VerificationStatus},
};

pub(crate) fn classify_validation_code(report: &PackageValidationReport) -> &'static str {
    if report
        .structured_issues
        .iter()
        .any(|issue| issue.code == PackageValidationIssueCode::InvalidChecksumFormat)
    {
        "package.invalid_checksum_format"
    } else if report
        .structured_issues
        .iter()
        .any(|issue| issue.code == PackageValidationIssueCode::ChecksumMismatch)
    {
        "package.checksum_mismatch"
    } else if report
        .structured_issues
        .iter()
        .any(|issue| issue.code == PackageValidationIssueCode::InvalidAssetPath)
    {
        "package.invalid_asset_path"
    } else if report
        .structured_issues
        .iter()
        .any(|issue| issue.code == PackageValidationIssueCode::LayoutMismatch)
    {
        "package.layout_mismatch"
    } else if report
        .structured_issues
        .iter()
        .any(|issue| issue.code == PackageValidationIssueCode::AssetReadFailed)
    {
        "package.asset_read_failed"
    } else if report
        .structured_issues
        .iter()
        .any(|issue| issue.code == PackageValidationIssueCode::InvalidPipelineConfig)
    {
        "package.invalid_pipeline_config"
    } else if report
        .structured_issues
        .iter()
        .any(|issue| issue.code == PackageValidationIssueCode::InvalidTokenizerConfig)
    {
        "package.invalid_tokenizer_config"
    } else if report
        .structured_issues
        .iter()
        .any(|issue| issue.code == PackageValidationIssueCode::InvalidVocabConfig)
    {
        "package.invalid_vocab_config"
    } else {
        "package.validation_failed"
    }
}

pub(crate) fn classify_verification_code(report: &PackageVerificationReport) -> &'static str {
    if report
        .observation_issues
        .iter()
        .any(|issue| issue.code == VerificationIssueCode::DuplicateObservation)
        || report
            .results
            .iter()
            .any(|result| result.issue_code == Some(VerificationIssueCode::DuplicateObservation))
    {
        "package.duplicate_observation"
    } else if report
        .observation_issues
        .iter()
        .any(|issue| issue.code == VerificationIssueCode::UnexpectedObservation)
    {
        "package.unexpected_observation"
    } else if report
        .results
        .iter()
        .any(|result| result.issue_code == Some(VerificationIssueCode::ObservationMissing))
    {
        "package.observed_output_missing"
    } else if report
        .results
        .iter()
        .any(|result| result.issue_code == Some(VerificationIssueCode::ObservationPathInvalid))
    {
        "package.invalid_asset_path"
    } else if report.results.iter().any(|result| {
        result.issue_code == Some(VerificationIssueCode::ObservedOutputReadFailed)
            || matches!(result.status, VerificationStatus::Missing)
    }) {
        "package.observed_output_missing"
    } else if report.results.iter().any(|result| result.content_mismatch) {
        "package.output_content_mismatch"
    } else if report.results.iter().any(|result| result.checksum_mismatch) {
        "package.checksum_mismatch"
    } else {
        "package.verification_failed"
    }
}

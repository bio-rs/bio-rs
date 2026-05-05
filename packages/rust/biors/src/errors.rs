use crate::exit_code;
use biors_core::{
    BioRsError, ErrorLocation, FastaReadError, ModelInputBuildError, PackageValidationIssueCode,
    PackageValidationReport, PackageVerificationReport, VerificationIssueCode, VerificationStatus,
};
use serde::Serialize;
use std::path::PathBuf;

#[derive(Debug)]
pub(crate) enum CliError {
    Core(BioRsError),
    ModelInput(ModelInputBuildError),
    Json(serde_json::Error),
    CurrentDir(std::io::Error),
    Read {
        path: PathBuf,
        source: std::io::Error,
    },
    Serialization(serde_json::Error),
    Write(std::io::Error),
    Validation {
        code: &'static str,
        message: String,
        location: Option<String>,
    },
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
pub(crate) enum ErrorLocationValue {
    Core(ErrorLocation),
    Label(String),
}

impl CliError {
    pub(crate) const fn code(&self) -> &'static str {
        match self {
            Self::Core(error) => error.code(),
            Self::ModelInput(ModelInputBuildError::InvalidPolicy { .. }) => {
                "model_input.invalid_policy"
            }
            Self::ModelInput(ModelInputBuildError::InvalidTokenizedSequence { .. }) => {
                "model_input.invalid_sequence"
            }
            Self::Json(_) => "json.invalid",
            Self::CurrentDir(_) => "io.read_failed",
            Self::Read { .. } => "io.read_failed",
            Self::Serialization(_) => "json.serialization_failed",
            Self::Write(_) => "io.write_failed",
            Self::Validation { code, .. } => code,
        }
    }

    pub(crate) fn location(&self) -> Option<ErrorLocationValue> {
        match self {
            Self::Core(error) => error.location().map(ErrorLocationValue::Core),
            Self::ModelInput(ModelInputBuildError::InvalidPolicy { .. }) => None,
            Self::ModelInput(ModelInputBuildError::InvalidTokenizedSequence { id, .. }) => {
                Some(ErrorLocationValue::Label(id.clone()))
            }
            Self::Read { path, .. } => Some(ErrorLocationValue::Label(path.display().to_string())),
            Self::Validation { location, .. } => location.clone().map(ErrorLocationValue::Label),
            Self::Json(_) | Self::CurrentDir(_) | Self::Serialization(_) | Self::Write(_) => None,
        }
    }

    pub(crate) const fn exit_code(&self) -> i32 {
        match self {
            Self::Core(_) | Self::ModelInput(_) | Self::Json(_) | Self::Validation { .. } => {
                exit_code::USER_INPUT_FAILURE
            }
            Self::Read { .. } | Self::CurrentDir(_) | Self::Serialization(_) | Self::Write(_) => {
                exit_code::IO_OR_INTERNAL_FAILURE
            }
        }
    }

    /// Convert a `FastaReadError` into the appropriate `CliError` variant.
    pub(crate) fn from_fasta_read(path: PathBuf, error: FastaReadError) -> Self {
        match error {
            FastaReadError::Parse(error) => Self::Core(error),
            FastaReadError::Io(source) => Self::Read { path, source },
        }
    }
}

impl std::fmt::Display for CliError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Core(error) => write!(f, "{error}"),
            Self::ModelInput(error) => write!(f, "{error}"),
            Self::Json(error) => write!(f, "{error}"),
            Self::CurrentDir(error) => write!(f, "failed to determine current directory: {error}"),
            Self::Read { path, source } => {
                write!(f, "failed to read '{}': {source}", path.display())
            }
            Self::Serialization(error) => write!(f, "{error}"),
            Self::Write(error) => write!(f, "failed to write output: {error}"),
            Self::Validation { message, .. } => write!(f, "{message}"),
        }
    }
}

impl std::error::Error for CliError {}

impl From<BioRsError> for CliError {
    fn from(error: BioRsError) -> Self {
        Self::Core(error)
    }
}

impl From<ModelInputBuildError> for CliError {
    fn from(error: ModelInputBuildError) -> Self {
        Self::ModelInput(error)
    }
}

impl From<serde_json::Error> for CliError {
    fn from(error: serde_json::Error) -> Self {
        Self::Serialization(error)
    }
}

/// Map a package validation report to a stable CLI error code.
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
        .any(|issue| issue.code == PackageValidationIssueCode::AssetReadFailed)
    {
        "package.asset_read_failed"
    } else {
        "package.validation_failed"
    }
}

/// Map a package verification report to a stable CLI error code.
pub(crate) fn classify_verification_code(report: &PackageVerificationReport) -> &'static str {
    if report
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

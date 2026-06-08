mod classification;

use crate::exit_code;
use biors_core::{
    error::{BioRsError, Diagnostic, ErrorLocation, FastaReadError},
    formats::FormatReadError,
    model_input::ModelInputBuildError,
    molecule::MoleculeReadError,
    reports::ReportBuildError,
    structure::StructureReadError,
};
pub(crate) use classification::{classify_validation_code, classify_verification_code};
use serde::Serialize;
use serde_json::Value;
use std::path::PathBuf;

#[derive(Debug)]
pub(crate) enum CliError {
    Core(BioRsError),
    Format(FormatReadError),
    Structure(StructureReadError),
    Molecule(MoleculeReadError),
    Report(ReportBuildError),
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
    ValidationDetails {
        code: &'static str,
        message: String,
        location: Option<String>,
        details: Value,
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
            Self::Format(error) => error.code(),
            Self::Structure(error) => error.code(),
            Self::Molecule(error) => error.code(),
            Self::Report(error) => error.code(),
            Self::ModelInput(ModelInputBuildError::InvalidPolicy { .. }) => {
                "model_input.invalid_policy"
            }
            Self::ModelInput(ModelInputBuildError::InvalidInputHash { .. }) => {
                "workflow.invalid_input_hash"
            }
            Self::ModelInput(ModelInputBuildError::EmptyTokenizedSequence { .. }) => {
                "model_input.invalid_sequence"
            }
            Self::ModelInput(ModelInputBuildError::InvalidTokenizedSequence { .. }) => {
                "model_input.invalid_sequence"
            }
            Self::Json(_) => "json.invalid",
            Self::CurrentDir(_) => "io.read_failed",
            Self::Read { .. } => "io.read_failed",
            Self::Serialization(_) => "json.serialization_failed",
            Self::Write(_) => "io.write_failed",
            Self::Validation { code, .. } | Self::ValidationDetails { code, .. } => code,
        }
    }

    pub(crate) fn location(&self) -> Option<ErrorLocationValue> {
        match self {
            Self::Core(error) => error.location().map(ErrorLocationValue::Core),
            Self::Format(error) => error.location().map(ErrorLocationValue::Core),
            Self::Structure(error) => error.location().map(ErrorLocationValue::Core),
            Self::Molecule(error) => error.location().map(ErrorLocationValue::Core),
            Self::Report(_) => None,
            Self::ModelInput(ModelInputBuildError::InvalidPolicy { .. }) => None,
            Self::ModelInput(ModelInputBuildError::InvalidInputHash { .. }) => None,
            Self::ModelInput(ModelInputBuildError::EmptyTokenizedSequence { id }) => {
                Some(ErrorLocationValue::Label(id.clone()))
            }
            Self::ModelInput(ModelInputBuildError::InvalidTokenizedSequence { id, .. }) => {
                Some(ErrorLocationValue::Label(id.clone()))
            }
            Self::Read { path, .. } => Some(ErrorLocationValue::Label(path.display().to_string())),
            Self::Validation { location, .. } | Self::ValidationDetails { location, .. } => {
                location.clone().map(ErrorLocationValue::Label)
            }
            Self::Json(_) | Self::CurrentDir(_) | Self::Serialization(_) | Self::Write(_) => None,
        }
    }

    pub(crate) fn details(&self) -> Option<&Value> {
        match self {
            Self::ValidationDetails { details, .. } => Some(details),
            _ => None,
        }
    }

    pub(crate) const fn exit_code(&self) -> i32 {
        match self {
            Self::Core(_)
            | Self::Format(_)
            | Self::Structure(_)
            | Self::Molecule(_)
            | Self::Report(_)
            | Self::ModelInput(_)
            | Self::Json(_)
            | Self::Validation { .. }
            | Self::ValidationDetails { .. } => exit_code::USER_INPUT_FAILURE,
            Self::Read { .. } | Self::CurrentDir(_) | Self::Serialization(_) | Self::Write(_) => {
                exit_code::IO_OR_INTERNAL_FAILURE
            }
        }
    }

    pub(crate) fn from_fasta_read(path: PathBuf, error: FastaReadError) -> Self {
        match error {
            FastaReadError::Parse(error) => Self::Core(error),
            FastaReadError::Io(source) => Self::Read { path, source },
        }
    }

    pub(crate) fn from_format_read(path: PathBuf, error: FormatReadError) -> Self {
        match error {
            FormatReadError::Io(source) => Self::Read { path, source },
            error => Self::Format(error),
        }
    }

    pub(crate) fn from_structure_read(path: PathBuf, error: StructureReadError) -> Self {
        match error {
            StructureReadError::Io(source) => Self::Read { path, source },
            error => Self::Structure(error),
        }
    }

    pub(crate) fn from_molecule_read(path: PathBuf, error: MoleculeReadError) -> Self {
        match error {
            MoleculeReadError::Io(source) => Self::Read { path, source },
            error => Self::Molecule(error),
        }
    }
}

impl std::fmt::Display for CliError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Core(error) => write!(f, "{error}"),
            Self::Format(error) => write!(f, "{error}"),
            Self::Structure(error) => write!(f, "{error}"),
            Self::Molecule(error) => write!(f, "{error}"),
            Self::Report(error) => write!(f, "{error}"),
            Self::ModelInput(error) => write!(f, "{error}"),
            Self::Json(error) => write!(f, "{error}"),
            Self::CurrentDir(error) => write!(f, "failed to determine current directory: {error}"),
            Self::Read { path, source } => {
                write!(f, "failed to read '{}': {source}", path.display())
            }
            Self::Serialization(error) => write!(f, "{error}"),
            Self::Write(error) => write!(f, "failed to write output: {error}"),
            Self::Validation { message, .. } | Self::ValidationDetails { message, .. } => {
                write!(f, "{message}")
            }
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

impl From<ReportBuildError> for CliError {
    fn from(error: ReportBuildError) -> Self {
        Self::Report(error)
    }
}

impl From<serde_json::Error> for CliError {
    fn from(error: serde_json::Error) -> Self {
        Self::Serialization(error)
    }
}

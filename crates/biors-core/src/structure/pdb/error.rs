use std::fmt;

use crate::error::{Diagnostic, ErrorLocation};

/// Public PDB parse errors.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PdbParseError {
    /// Input contained no bytes.
    EmptyInput,
    /// A required fixed-column atom field was blank or unavailable.
    MissingAtomField { field: &'static str, line: usize },
    /// A fixed-column atom field could not be parsed.
    InvalidAtomField {
        field: &'static str,
        value: String,
        line: usize,
    },
}

impl PdbParseError {
    /// Stable machine-readable error code.
    pub const fn code(&self) -> &'static str {
        match self {
            Self::EmptyInput => "pdb.empty_input",
            Self::MissingAtomField { .. } => "pdb.missing_atom_field",
            Self::InvalidAtomField { .. } => "pdb.invalid_atom_field",
        }
    }

    /// Optional structured source location.
    pub const fn location(&self) -> Option<ErrorLocation> {
        match self {
            Self::EmptyInput => None,
            Self::MissingAtomField { line, .. } | Self::InvalidAtomField { line, .. } => {
                Some(ErrorLocation::line(*line))
            }
        }
    }
}

impl fmt::Display for PdbParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyInput => write!(f, "PDB input is empty"),
            Self::MissingAtomField { field, line } => {
                write!(f, "PDB atom record at line {line} is missing {field}")
            }
            Self::InvalidAtomField { field, value, line } => write!(
                f,
                "PDB atom record at line {line} has invalid {field} value '{value}'"
            ),
        }
    }
}

impl std::error::Error for PdbParseError {}

impl Diagnostic for PdbParseError {
    fn code(&self) -> &'static str {
        PdbParseError::code(self)
    }

    fn message(&self) -> String {
        self.to_string()
    }

    fn location(&self) -> Option<ErrorLocation> {
        PdbParseError::location(self)
    }
}

/// Error type for streaming structure reader APIs.
#[derive(Debug)]
pub enum StructureReadError {
    /// PDB syntax error.
    PdbParse(PdbParseError),
    /// Underlying I/O or UTF-8 read failure.
    Io(std::io::Error),
}

impl StructureReadError {
    /// Stable machine-readable error code.
    pub const fn code(&self) -> &'static str {
        match self {
            Self::PdbParse(error) => error.code(),
            Self::Io(_) => "io.read_failed",
        }
    }
}

impl fmt::Display for StructureReadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PdbParse(error) => write!(f, "{error}"),
            Self::Io(error) => write!(f, "{error}"),
        }
    }
}

impl std::error::Error for StructureReadError {}

impl Diagnostic for StructureReadError {
    fn code(&self) -> &'static str {
        StructureReadError::code(self)
    }

    fn message(&self) -> String {
        self.to_string()
    }

    fn location(&self) -> Option<ErrorLocation> {
        match self {
            Self::PdbParse(error) => error.location(),
            Self::Io(_) => None,
        }
    }
}

impl From<PdbParseError> for StructureReadError {
    fn from(error: PdbParseError) -> Self {
        Self::PdbParse(error)
    }
}

impl From<std::io::Error> for StructureReadError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}

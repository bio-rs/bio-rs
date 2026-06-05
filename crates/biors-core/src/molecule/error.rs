use std::fmt;

use crate::error::{Diagnostic, ErrorLocation};

use super::mol2::Mol2ParseError;
use super::sdf::SdfParseError;
use super::smiles::SmilesParseError;

/// Error type for streaming molecule reader APIs.
#[derive(Debug)]
pub enum MoleculeReadError {
    /// SMILES syntax error.
    SmilesParse(SmilesParseError),
    /// SDF syntax error.
    SdfParse(SdfParseError),
    /// MOL2 syntax error.
    Mol2Parse(Mol2ParseError),
    /// Underlying I/O or UTF-8 read failure.
    Io(std::io::Error),
}

impl MoleculeReadError {
    /// Stable machine-readable error code.
    pub const fn code(&self) -> &'static str {
        match self {
            Self::SmilesParse(error) => error.code(),
            Self::SdfParse(error) => error.code(),
            Self::Mol2Parse(error) => error.code(),
            Self::Io(_) => "io.read_failed",
        }
    }
}

impl fmt::Display for MoleculeReadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SmilesParse(error) => write!(f, "{error}"),
            Self::SdfParse(error) => write!(f, "{error}"),
            Self::Mol2Parse(error) => write!(f, "{error}"),
            Self::Io(error) => write!(f, "{error}"),
        }
    }
}

impl std::error::Error for MoleculeReadError {}

impl Diagnostic for MoleculeReadError {
    fn code(&self) -> &'static str {
        MoleculeReadError::code(self)
    }

    fn message(&self) -> String {
        self.to_string()
    }

    fn location(&self) -> Option<ErrorLocation> {
        match self {
            Self::SmilesParse(error) => error.location(),
            Self::SdfParse(error) => error.location(),
            Self::Mol2Parse(error) => error.location(),
            Self::Io(_) => None,
        }
    }
}

impl From<SmilesParseError> for MoleculeReadError {
    fn from(error: SmilesParseError) -> Self {
        Self::SmilesParse(error)
    }
}

impl From<SdfParseError> for MoleculeReadError {
    fn from(error: SdfParseError) -> Self {
        Self::SdfParse(error)
    }
}

impl From<Mol2ParseError> for MoleculeReadError {
    fn from(error: Mol2ParseError) -> Self {
        Self::Mol2Parse(error)
    }
}

impl From<std::io::Error> for MoleculeReadError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}

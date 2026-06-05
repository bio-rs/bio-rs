use std::fmt;

use crate::error::{Diagnostic, ErrorLocation};

/// Public SDF parse errors.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SdfParseError {
    EmptyInput,
    MissingCountsLine {
        line: usize,
        record_index: usize,
    },
    InvalidCountsLine {
        line: usize,
        record_index: usize,
    },
    InvalidAtomLine {
        line: usize,
        record_index: usize,
    },
    InvalidBondLine {
        line: usize,
        record_index: usize,
    },
    UnsupportedBondType {
        line: usize,
        bond_type: String,
        record_index: usize,
    },
    InvalidV3000Line {
        line: usize,
        record_index: usize,
    },
}

impl SdfParseError {
    pub const fn code(&self) -> &'static str {
        match self {
            Self::EmptyInput => "sdf.empty_input",
            Self::MissingCountsLine { .. } => "sdf.missing_counts_line",
            Self::InvalidCountsLine { .. } => "sdf.invalid_counts_line",
            Self::InvalidAtomLine { .. } => "sdf.invalid_atom_line",
            Self::InvalidBondLine { .. } => "sdf.invalid_bond_line",
            Self::UnsupportedBondType { .. } => "sdf.unsupported_bond_type",
            Self::InvalidV3000Line { .. } => "sdf.invalid_v3000_line",
        }
    }

    pub const fn location(&self) -> Option<ErrorLocation> {
        match self {
            Self::EmptyInput => None,
            Self::MissingCountsLine { line, record_index }
            | Self::InvalidCountsLine { line, record_index }
            | Self::InvalidAtomLine { line, record_index }
            | Self::InvalidBondLine { line, record_index }
            | Self::UnsupportedBondType {
                line, record_index, ..
            }
            | Self::InvalidV3000Line { line, record_index } => {
                Some(ErrorLocation::record(*line, *record_index))
            }
        }
    }
}

impl fmt::Display for SdfParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyInput => write!(f, "SDF input is empty"),
            Self::MissingCountsLine { record_index, .. } => {
                write!(
                    f,
                    "SDF record at index {record_index} is missing a counts line"
                )
            }
            Self::InvalidCountsLine { record_index, .. } => {
                write!(
                    f,
                    "SDF record at index {record_index} has an invalid counts line"
                )
            }
            Self::InvalidAtomLine { record_index, .. } => {
                write!(
                    f,
                    "SDF record at index {record_index} has an invalid atom line"
                )
            }
            Self::InvalidBondLine { record_index, .. } => {
                write!(
                    f,
                    "SDF record at index {record_index} has an invalid bond line"
                )
            }
            Self::UnsupportedBondType {
                bond_type,
                record_index,
                ..
            } => write!(
                f,
                "SDF record at index {record_index} uses unsupported bond type '{bond_type}'"
            ),
            Self::InvalidV3000Line { record_index, .. } => {
                write!(
                    f,
                    "SDF record at index {record_index} has an invalid V3000 line"
                )
            }
        }
    }
}

impl std::error::Error for SdfParseError {}

impl Diagnostic for SdfParseError {
    fn code(&self) -> &'static str {
        SdfParseError::code(self)
    }

    fn message(&self) -> String {
        self.to_string()
    }

    fn location(&self) -> Option<ErrorLocation> {
        SdfParseError::location(self)
    }
}

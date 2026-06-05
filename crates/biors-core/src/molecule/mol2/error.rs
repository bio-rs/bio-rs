use std::fmt;

use crate::error::{Diagnostic, ErrorLocation};

/// Public MOL2 parse errors.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Mol2ParseError {
    EmptyInput,
    MissingMoleculeSection {
        line: usize,
        record_index: usize,
    },
    MissingMoleculeName {
        line: usize,
        record_index: usize,
    },
    MissingCountsLine {
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
}

impl Mol2ParseError {
    pub const fn code(&self) -> &'static str {
        match self {
            Self::EmptyInput => "mol2.empty_input",
            Self::MissingMoleculeSection { .. } => "mol2.missing_molecule_section",
            Self::MissingMoleculeName { .. } => "mol2.missing_molecule_name",
            Self::MissingCountsLine { .. } => "mol2.missing_counts_line",
            Self::InvalidAtomLine { .. } => "mol2.invalid_atom_line",
            Self::InvalidBondLine { .. } => "mol2.invalid_bond_line",
            Self::UnsupportedBondType { .. } => "mol2.unsupported_bond_type",
        }
    }

    pub const fn location(&self) -> Option<ErrorLocation> {
        match self {
            Self::EmptyInput => None,
            Self::MissingMoleculeSection { line, record_index }
            | Self::MissingMoleculeName { line, record_index }
            | Self::MissingCountsLine { line, record_index }
            | Self::InvalidAtomLine { line, record_index }
            | Self::InvalidBondLine { line, record_index }
            | Self::UnsupportedBondType {
                line, record_index, ..
            } => Some(ErrorLocation::record(*line, *record_index)),
        }
    }
}

impl fmt::Display for Mol2ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyInput => write!(f, "MOL2 input is empty"),
            Self::MissingMoleculeSection { record_index, .. } => {
                write!(
                    f,
                    "MOL2 record at index {record_index} is missing @<TRIPOS>MOLECULE"
                )
            }
            Self::MissingMoleculeName { record_index, .. } => {
                write!(
                    f,
                    "MOL2 record at index {record_index} is missing a molecule name"
                )
            }
            Self::MissingCountsLine { record_index, .. } => {
                write!(
                    f,
                    "MOL2 record at index {record_index} is missing a counts line"
                )
            }
            Self::InvalidAtomLine { record_index, .. } => {
                write!(
                    f,
                    "MOL2 record at index {record_index} has an invalid atom line"
                )
            }
            Self::InvalidBondLine { record_index, .. } => {
                write!(
                    f,
                    "MOL2 record at index {record_index} has an invalid bond line"
                )
            }
            Self::UnsupportedBondType {
                bond_type,
                record_index,
                ..
            } => write!(
                f,
                "MOL2 record at index {record_index} uses unsupported bond type '{bond_type}'"
            ),
        }
    }
}

impl std::error::Error for Mol2ParseError {}

impl Diagnostic for Mol2ParseError {
    fn code(&self) -> &'static str {
        Mol2ParseError::code(self)
    }

    fn message(&self) -> String {
        self.to_string()
    }

    fn location(&self) -> Option<ErrorLocation> {
        Mol2ParseError::location(self)
    }
}

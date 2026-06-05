use std::fmt;

use crate::error::{Diagnostic, ErrorLocation};

/// Public SMILES parse errors.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SmilesParseError {
    /// Input contained no parseable SMILES records.
    EmptyInput,
    /// A record line did not contain a SMILES token.
    MissingSmiles { line: usize, record_index: usize },
    /// An atom token was expected.
    MissingAtom {
        line: usize,
        column: usize,
        record_index: usize,
    },
    /// An unsupported character appeared in a SMILES token.
    UnexpectedCharacter {
        line: usize,
        column: usize,
        character: char,
        record_index: usize,
    },
    /// A bond marker appeared where no following atom or ring closure was present.
    DanglingBond {
        line: usize,
        column: usize,
        record_index: usize,
    },
    /// A branch opened before any atom was available.
    InvalidBranch {
        line: usize,
        column: usize,
        record_index: usize,
    },
    /// A branch was opened but never closed.
    UnclosedBranch {
        line: usize,
        column: usize,
        record_index: usize,
    },
    /// A closing branch token had no corresponding open branch.
    UnmatchedBranch {
        line: usize,
        column: usize,
        record_index: usize,
    },
    /// A ring-closure token was opened but never closed.
    UnclosedRing {
        line: usize,
        column: usize,
        ring: u16,
        record_index: usize,
    },
    /// Ring closure appeared before any atom.
    InvalidRingClosure {
        line: usize,
        column: usize,
        record_index: usize,
    },
    /// A bracket atom could not be parsed.
    InvalidBracketAtom {
        line: usize,
        column: usize,
        token: String,
        reason: String,
        record_index: usize,
    },
}

impl SmilesParseError {
    /// Stable machine-readable error code.
    pub const fn code(&self) -> &'static str {
        match self {
            Self::EmptyInput => "smiles.empty_input",
            Self::MissingSmiles { .. } => "smiles.missing_smiles",
            Self::MissingAtom { .. } => "smiles.missing_atom",
            Self::UnexpectedCharacter { .. } => "smiles.unexpected_character",
            Self::DanglingBond { .. } => "smiles.dangling_bond",
            Self::InvalidBranch { .. } => "smiles.invalid_branch",
            Self::UnclosedBranch { .. } => "smiles.unclosed_branch",
            Self::UnmatchedBranch { .. } => "smiles.unmatched_branch",
            Self::UnclosedRing { .. } => "smiles.unclosed_ring",
            Self::InvalidRingClosure { .. } => "smiles.invalid_ring_closure",
            Self::InvalidBracketAtom { .. } => "smiles.invalid_bracket_atom",
        }
    }

    /// Optional structured source location.
    pub const fn location(&self) -> Option<ErrorLocation> {
        match self {
            Self::EmptyInput => None,
            Self::MissingSmiles { line, record_index }
            | Self::MissingAtom {
                line, record_index, ..
            }
            | Self::UnexpectedCharacter {
                line, record_index, ..
            }
            | Self::DanglingBond {
                line, record_index, ..
            }
            | Self::InvalidBranch {
                line, record_index, ..
            }
            | Self::UnclosedBranch {
                line, record_index, ..
            }
            | Self::UnmatchedBranch {
                line, record_index, ..
            }
            | Self::UnclosedRing {
                line, record_index, ..
            }
            | Self::InvalidRingClosure {
                line, record_index, ..
            }
            | Self::InvalidBracketAtom {
                line, record_index, ..
            } => Some(ErrorLocation::record(*line, *record_index)),
        }
    }
}

impl fmt::Display for SmilesParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyInput => write!(f, "SMILES input is empty"),
            Self::MissingSmiles { record_index, line } => write!(
                f,
                "SMILES record at index {record_index} is empty at line {line}"
            ),
            Self::MissingAtom {
                record_index,
                column,
                ..
            } => write!(
                f,
                "SMILES record at index {record_index} is missing an atom near column {column}"
            ),
            Self::UnexpectedCharacter {
                character,
                column,
                record_index,
                ..
            } => write!(
                f,
                "SMILES record at index {record_index} has unexpected character '{character}' at column {column}"
            ),
            Self::DanglingBond {
                column,
                record_index,
                ..
            } => write!(
                f,
                "SMILES record at index {record_index} has a dangling bond marker at column {column}"
            ),
            Self::InvalidBranch {
                column,
                record_index,
                ..
            } => write!(
                f,
                "SMILES record at index {record_index} opens a branch before an atom at column {column}"
            ),
            Self::UnclosedBranch {
                column,
                record_index,
                ..
            } => write!(
                f,
                "SMILES record at index {record_index} has an unclosed branch opened at column {column}"
            ),
            Self::UnmatchedBranch {
                column,
                record_index,
                ..
            } => write!(
                f,
                "SMILES record at index {record_index} has an unmatched branch close at column {column}"
            ),
            Self::UnclosedRing {
                ring,
                record_index,
                ..
            } => write!(
                f,
                "SMILES record at index {record_index} has unclosed ring closure {ring}"
            ),
            Self::InvalidRingClosure {
                column,
                record_index,
                ..
            } => write!(
                f,
                "SMILES record at index {record_index} has a ring closure before an atom at column {column}"
            ),
            Self::InvalidBracketAtom {
                token,
                reason,
                record_index,
                ..
            } => write!(
                f,
                "SMILES record at index {record_index} has invalid bracket atom '{token}': {reason}"
            ),
        }
    }
}

impl std::error::Error for SmilesParseError {}

impl Diagnostic for SmilesParseError {
    fn code(&self) -> &'static str {
        SmilesParseError::code(self)
    }

    fn message(&self) -> String {
        self.to_string()
    }

    fn location(&self) -> Option<ErrorLocation> {
        SmilesParseError::location(self)
    }
}

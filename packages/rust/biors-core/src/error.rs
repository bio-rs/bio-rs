use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ErrorLocation {
    pub line: Option<usize>,
    pub record_index: Option<usize>,
}

impl ErrorLocation {
    pub const fn line(line: usize) -> Self {
        Self {
            line: Some(line),
            record_index: None,
        }
    }

    pub const fn record(line: usize, record_index: usize) -> Self {
        Self {
            line: Some(line),
            record_index: Some(record_index),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BioRsError {
    EmptyInput,
    MissingHeader {
        line: usize,
    },
    MissingSequence {
        id: String,
        line: usize,
        record_index: usize,
    },
}

impl BioRsError {
    pub const fn code(&self) -> &'static str {
        match self {
            Self::EmptyInput => "fasta.empty_input",
            Self::MissingHeader { .. } => "fasta.missing_header",
            Self::MissingSequence { .. } => "fasta.missing_sequence",
        }
    }

    pub const fn location(&self) -> Option<ErrorLocation> {
        match self {
            Self::EmptyInput => None,
            Self::MissingHeader { line } => Some(ErrorLocation::line(*line)),
            Self::MissingSequence {
                line, record_index, ..
            } => Some(ErrorLocation::record(*line, *record_index)),
        }
    }
}

impl fmt::Display for BioRsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyInput => write!(f, "FASTA input is empty"),
            Self::MissingHeader { line } => write!(
                f,
                "FASTA input must start with a header line beginning with '>' at line {line}"
            ),
            Self::MissingSequence {
                id, record_index, ..
            } => write!(
                f,
                "FASTA record '{id}' at index {record_index} does not contain a sequence"
            ),
        }
    }
}

impl std::error::Error for BioRsError {}

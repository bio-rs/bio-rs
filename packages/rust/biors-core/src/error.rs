use serde::{Deserialize, Serialize};
use std::fmt;

/// Common interface for errors and validation issues that expose stable CLI/API diagnostics.
pub trait Diagnostic {
    /// Stable machine-readable diagnostic code.
    fn code(&self) -> &'static str;

    /// Human-readable diagnostic message.
    fn message(&self) -> String;

    /// Optional structured location for the diagnostic.
    fn location(&self) -> Option<ErrorLocation> {
        None
    }
}

/// Machine-readable location metadata for a parse or validation error.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ErrorLocation {
    /// One-based source line when the error can be located in input text.
    pub line: Option<usize>,
    /// Zero-based FASTA record index when the error belongs to a record.
    pub record_index: Option<usize>,
}

impl ErrorLocation {
    /// Construct a location that points only to a source line.
    pub const fn line(line: usize) -> Self {
        Self {
            line: Some(line),
            record_index: None,
        }
    }

    /// Construct a location that points to a source line and FASTA record index.
    pub const fn record(line: usize, record_index: usize) -> Self {
        Self {
            line: Some(line),
            record_index: Some(record_index),
        }
    }
}

/// Public parse errors produced by FASTA string APIs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BioRsError {
    /// Input contained no non-whitespace FASTA content.
    EmptyInput,
    /// A FASTA header line did not contain an identifier.
    MissingIdentifier {
        /// One-based line number of the header.
        line: usize,
        /// Zero-based record index being parsed.
        record_index: usize,
    },
    /// Sequence content appeared before any FASTA header.
    MissingHeader {
        /// One-based line number where sequence content was seen.
        line: usize,
    },
    /// A FASTA record had a header but no sequence residues.
    MissingSequence {
        /// FASTA identifier for the empty record.
        id: String,
        /// One-based line number of the record header.
        line: usize,
        /// Zero-based record index.
        record_index: usize,
    },
}

impl BioRsError {
    /// Stable machine-readable error code used by CLI JSON envelopes.
    pub const fn code(&self) -> &'static str {
        match self {
            Self::EmptyInput => "fasta.empty_input",
            Self::MissingIdentifier { .. } => "fasta.missing_identifier",
            Self::MissingHeader { .. } => "fasta.missing_header",
            Self::MissingSequence { .. } => "fasta.missing_sequence",
        }
    }

    /// Optional structured source location for this error.
    pub const fn location(&self) -> Option<ErrorLocation> {
        match self {
            Self::EmptyInput => None,
            Self::MissingIdentifier { line, record_index } => {
                Some(ErrorLocation::record(*line, *record_index))
            }
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
            Self::MissingIdentifier { record_index, .. } => write!(
                f,
                "FASTA record at index {record_index} has an empty identifier"
            ),
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

impl Diagnostic for BioRsError {
    fn code(&self) -> &'static str {
        BioRsError::code(self)
    }

    fn message(&self) -> String {
        self.to_string()
    }

    fn location(&self) -> Option<ErrorLocation> {
        BioRsError::location(self)
    }
}

/// Error type for streaming FASTA reader APIs.
#[derive(Debug)]
pub enum FastaReadError {
    /// FASTA syntax or record validation error.
    Parse(BioRsError),
    /// Underlying I/O or UTF-8 read failure.
    Io(std::io::Error),
}

impl FastaReadError {
    /// Stable machine-readable error code used by CLI JSON envelopes.
    pub const fn code(&self) -> &'static str {
        match self {
            Self::Parse(error) => error.code(),
            Self::Io(_) => "io.read_failed",
        }
    }
}

impl fmt::Display for FastaReadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Parse(error) => write!(f, "{error}"),
            Self::Io(error) => write!(f, "{error}"),
        }
    }
}

impl std::error::Error for FastaReadError {}

impl Diagnostic for FastaReadError {
    fn code(&self) -> &'static str {
        FastaReadError::code(self)
    }

    fn message(&self) -> String {
        self.to_string()
    }

    fn location(&self) -> Option<ErrorLocation> {
        match self {
            Self::Parse(error) => error.location(),
            Self::Io(_) => None,
        }
    }
}

impl From<BioRsError> for FastaReadError {
    fn from(error: BioRsError) -> Self {
        Self::Parse(error)
    }
}

impl From<std::io::Error> for FastaReadError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}

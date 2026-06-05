use std::fmt;

use crate::error::{Diagnostic, ErrorLocation};

/// Public FASTQ parse errors.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FastqParseError {
    /// Input contained no bytes.
    EmptyInput,
    /// A record did not start with an `@` header.
    MissingHeader {
        /// One-based line number.
        line: usize,
        /// Zero-based record index.
        record_index: usize,
    },
    /// A header had no identifier.
    MissingIdentifier {
        /// One-based line number.
        line: usize,
        /// Zero-based record index.
        record_index: usize,
    },
    /// The sequence body ended before a `+` separator line was found.
    MissingSeparator {
        /// FASTQ identifier.
        id: String,
        /// One-based header line.
        line: usize,
        /// Zero-based record index.
        record_index: usize,
    },
    /// A record had no sequence symbols before the separator.
    MissingSequence {
        /// FASTQ identifier.
        id: String,
        /// One-based header line.
        line: usize,
        /// Zero-based record index.
        record_index: usize,
    },
    /// The optional separator identifier did not match the header identifier.
    SeparatorIdentifierMismatch {
        /// FASTQ identifier from the header.
        id: String,
        /// Identifier found after `+`.
        separator_id: String,
        /// One-based separator line.
        line: usize,
        /// Zero-based record index.
        record_index: usize,
    },
    /// The quality body ended before enough quality symbols were read.
    MissingQuality {
        /// FASTQ identifier.
        id: String,
        /// Expected number of quality symbols.
        expected: usize,
        /// Observed number of quality symbols.
        observed: usize,
        /// Zero-based record index.
        record_index: usize,
    },
    /// The quality body length differed from sequence length.
    QualityLengthMismatch {
        /// FASTQ identifier.
        id: String,
        /// Expected number of quality symbols.
        expected: usize,
        /// Observed number of quality symbols.
        observed: usize,
        /// One-based line where the mismatch became known.
        line: usize,
        /// Zero-based record index.
        record_index: usize,
    },
}

impl FastqParseError {
    /// Stable machine-readable error code.
    pub const fn code(&self) -> &'static str {
        match self {
            Self::EmptyInput => "fastq.empty_input",
            Self::MissingHeader { .. } => "fastq.missing_header",
            Self::MissingIdentifier { .. } => "fastq.missing_identifier",
            Self::MissingSeparator { .. } => "fastq.missing_separator",
            Self::MissingSequence { .. } => "fastq.missing_sequence",
            Self::SeparatorIdentifierMismatch { .. } => "fastq.separator_identifier_mismatch",
            Self::MissingQuality { .. } => "fastq.missing_quality",
            Self::QualityLengthMismatch { .. } => "fastq.quality_length_mismatch",
        }
    }

    /// Optional structured source location.
    pub const fn location(&self) -> Option<ErrorLocation> {
        match self {
            Self::EmptyInput | Self::MissingQuality { .. } => None,
            Self::MissingHeader { line, record_index }
            | Self::MissingIdentifier { line, record_index }
            | Self::MissingSeparator {
                line, record_index, ..
            }
            | Self::MissingSequence {
                line, record_index, ..
            }
            | Self::SeparatorIdentifierMismatch {
                line, record_index, ..
            }
            | Self::QualityLengthMismatch {
                line, record_index, ..
            } => Some(ErrorLocation::record(*line, *record_index)),
        }
    }
}

impl fmt::Display for FastqParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyInput => write!(f, "FASTQ input is empty"),
            Self::MissingHeader { record_index, line } => write!(
                f,
                "FASTQ record at index {record_index} must start with '@' at line {line}"
            ),
            Self::MissingIdentifier { record_index, .. } => write!(
                f,
                "FASTQ record at index {record_index} has an empty identifier"
            ),
            Self::MissingSeparator {
                id, record_index, ..
            } => write!(
                f,
                "FASTQ record '{id}' at index {record_index} is missing a '+' separator"
            ),
            Self::MissingSequence {
                id, record_index, ..
            } => write!(
                f,
                "FASTQ record '{id}' at index {record_index} does not contain a sequence"
            ),
            Self::SeparatorIdentifierMismatch {
                id,
                separator_id,
                record_index,
                ..
            } => write!(
                f,
                "FASTQ record '{id}' at index {record_index} has separator identifier '{separator_id}'"
            ),
            Self::MissingQuality {
                id,
                expected,
                observed,
                record_index,
            } => write!(
                f,
                "FASTQ record '{id}' at index {record_index} has {observed} quality symbols but expected {expected}"
            ),
            Self::QualityLengthMismatch {
                id,
                expected,
                observed,
                record_index,
                ..
            } => write!(
                f,
                "FASTQ record '{id}' at index {record_index} has {observed} quality symbols but expected {expected}"
            ),
        }
    }
}

impl std::error::Error for FastqParseError {}

impl Diagnostic for FastqParseError {
    fn code(&self) -> &'static str {
        FastqParseError::code(self)
    }

    fn message(&self) -> String {
        self.to_string()
    }

    fn location(&self) -> Option<ErrorLocation> {
        FastqParseError::location(self)
    }
}

/// Error type for streaming format reader APIs.
#[derive(Debug)]
pub enum FormatReadError {
    /// FASTQ syntax error.
    FastqParse(FastqParseError),
    /// Underlying I/O or UTF-8 read failure.
    Io(std::io::Error),
}

impl FormatReadError {
    /// Stable machine-readable error code.
    pub const fn code(&self) -> &'static str {
        match self {
            Self::FastqParse(error) => error.code(),
            Self::Io(_) => "io.read_failed",
        }
    }
}

impl fmt::Display for FormatReadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FastqParse(error) => write!(f, "{error}"),
            Self::Io(error) => write!(f, "{error}"),
        }
    }
}

impl std::error::Error for FormatReadError {}

impl Diagnostic for FormatReadError {
    fn code(&self) -> &'static str {
        FormatReadError::code(self)
    }

    fn message(&self) -> String {
        self.to_string()
    }

    fn location(&self) -> Option<ErrorLocation> {
        match self {
            Self::FastqParse(error) => error.location(),
            Self::Io(_) => None,
        }
    }
}

impl From<FastqParseError> for FormatReadError {
    fn from(error: FastqParseError) -> Self {
        Self::FastqParse(error)
    }
}

impl From<std::io::Error> for FormatReadError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}

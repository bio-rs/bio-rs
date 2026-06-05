use serde::{Deserialize, Serialize};

use crate::error::Diagnostic;
use crate::sequence::SequenceKind;

use super::super::records::{BioFormat, FormatField, FormatMetadata, FormatRecord};

/// Result of parsing FASTQ from a reader.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ParsedFastqInput {
    /// Stable hash of the raw input stream.
    pub input_hash: String,
    /// Parsed FASTQ records.
    pub records: Vec<FastqRecord>,
}

/// Result of validating FASTQ from a reader.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValidatedFastqInput {
    /// Stable hash of the raw input stream.
    pub input_hash: String,
    /// Aggregate validation report.
    pub report: FastqValidationReport,
}

/// Parsed FASTQ read record.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FastqRecord {
    /// Identifier after `@`, excluding the optional description.
    pub id: String,
    /// Optional header description after the first whitespace-delimited token.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Normalized uppercase sequence symbols.
    pub sequence: String,
    /// Raw Phred+33 quality string.
    pub quality: String,
    /// Source record metadata.
    pub metadata: FormatMetadata,
}

impl FastqRecord {
    /// Project this FASTQ read into the shared format-record contract.
    pub fn to_format_record(&self) -> FormatRecord {
        let mut fields = vec![
            FormatField::new("sequence", self.sequence.clone()),
            FormatField::new("quality", self.quality.clone()),
        ];
        if let Some(description) = &self.description {
            fields.push(FormatField::new("description", description.clone()));
        }
        FormatRecord::new(
            BioFormat::Fastq,
            self.id.clone(),
            self.metadata.clone(),
            fields,
        )
    }
}

/// Aggregate validation report for FASTQ records.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FastqValidationReport {
    /// Format family.
    pub format: BioFormat,
    /// Sequence policy used for FASTQ sequence validation.
    pub sequence_kind: SequenceKind,
    /// Number of records.
    pub records: usize,
    /// Number of records with no warnings and no errors.
    pub valid_records: usize,
    /// Total warning count.
    pub warning_count: usize,
    /// Total error count.
    pub error_count: usize,
    /// Per-record validation details without raw quality payload duplication.
    pub record_reports: Vec<FastqRecordValidation>,
}

/// Validation details for one FASTQ read.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FastqRecordValidation {
    /// FASTQ record identifier.
    pub id: String,
    /// Optional header description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Normalized sequence length.
    pub sequence_length: usize,
    /// Quality string length.
    pub quality_length: usize,
    /// Source record metadata.
    pub metadata: FormatMetadata,
    /// True when this read has no warnings and no errors.
    pub valid: bool,
    /// Non-fatal biological warnings such as ambiguous DNA symbols.
    pub warnings: Vec<FastqValidationIssue>,
    /// Structural or biological validation errors.
    pub errors: Vec<FastqValidationIssue>,
}

/// Stable FASTQ validation issue code.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FastqValidationIssueCode {
    /// Supported ambiguous DNA/IUPAC symbol.
    AmbiguousSymbol,
    /// Unsupported DNA sequence symbol.
    InvalidSymbol,
    /// Quality character is outside printable Phred+33 ASCII.
    InvalidQualityCharacter,
}

impl FastqValidationIssueCode {
    /// Stable code string.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::AmbiguousSymbol => "ambiguous_symbol",
            Self::InvalidSymbol => "invalid_symbol",
            Self::InvalidQualityCharacter => "invalid_quality_character",
        }
    }
}

/// FASTQ sequence or quality validation issue.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FastqValidationIssue {
    /// Offending symbol.
    pub symbol: char,
    /// One-based position in sequence or quality string.
    pub position: usize,
    /// Stable issue code.
    pub code: FastqValidationIssueCode,
    /// Human-readable message.
    pub message: String,
}

impl FastqValidationIssue {
    pub(crate) fn ambiguous(symbol: char, position: usize) -> Self {
        Self {
            symbol,
            position,
            code: FastqValidationIssueCode::AmbiguousSymbol,
            message: format!(
                "DNA symbol '{symbol}' at position {position} is ambiguous IUPAC code"
            ),
        }
    }

    pub(crate) fn invalid_symbol(symbol: char, position: usize) -> Self {
        Self {
            symbol,
            position,
            code: FastqValidationIssueCode::InvalidSymbol,
            message: format!(
                "DNA symbol '{symbol}' at position {position} is not supported by dna-iupac"
            ),
        }
    }

    pub(crate) fn invalid_quality_character(symbol: char, position: usize) -> Self {
        Self {
            symbol,
            position,
            code: FastqValidationIssueCode::InvalidQualityCharacter,
            message: format!(
                "FASTQ quality symbol '{symbol}' at position {position} is outside printable Phred+33 ASCII"
            ),
        }
    }
}

impl Diagnostic for FastqValidationIssue {
    fn code(&self) -> &'static str {
        self.code.as_str()
    }

    fn message(&self) -> String {
        self.message.clone()
    }
}

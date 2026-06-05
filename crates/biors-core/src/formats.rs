//! Shared biological file-format contracts and format-specific parsers.
//!
//! The format layer starts narrow on purpose. `0.48.0` introduces shared record
//! metadata and a production-oriented FASTQ validator while recording explicit
//! requirements for other common bioinformatics formats before they become
//! executable contracts.

mod capabilities;
mod fastq;
mod records;

pub use capabilities::{format_capabilities, FormatCapability, FormatSupportStatus};
pub(crate) use fastq::{fastq_quality_symbol_count, validate_fastq_quality};
pub use fastq::{
    parse_fastq_records, parse_fastq_records_reader, validate_fastq_reader,
    validate_fastq_reader_with_hash, FastqParseError, FastqRecord, FastqRecordValidation,
    FastqValidationIssue, FastqValidationIssueCode, FastqValidationReport, FormatReadError,
    ParsedFastqInput, ValidatedFastqInput,
};
pub use records::{BioFormat, FormatField, FormatMetadata, FormatRecord};

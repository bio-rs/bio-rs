mod error;
mod parse;
#[cfg(test)]
mod parse_tests;
mod types;
mod validation;

pub use error::{FastqParseError, FormatReadError};
pub use parse::{parse_fastq_records, parse_fastq_records_reader};
pub use types::{
    FastqRecord, FastqRecordValidation, FastqValidationIssue, FastqValidationIssueCode,
    FastqValidationReport, ParsedFastqInput, ValidatedFastqInput,
};
pub(crate) use validation::{fastq_quality_symbol_count, validate_fastq_quality};
pub use validation::{validate_fastq_reader, validate_fastq_reader_with_hash};

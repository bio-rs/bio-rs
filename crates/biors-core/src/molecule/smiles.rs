//! SMILES parser and validation entrypoints.

mod error;
mod parse;
mod validation;

pub use error::SmilesParseError;
pub use parse::{parse_smiles_records, parse_smiles_records_reader};
pub use validation::{validate_smiles_reader, validate_smiles_reader_with_hash};

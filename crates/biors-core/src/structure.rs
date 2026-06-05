//! Structure parsing, validation, and sequence extraction APIs.

pub mod pdb;
mod residue_codes;
mod types;
mod validation;

pub use pdb::{
    parse_pdb_record, parse_pdb_record_reader, validate_pdb_reader, validate_pdb_reader_with_hash,
    PdbParseError, StructureReadError,
};
pub use types::{
    Atom, Chain, Coordinate, MissingResidue, ParsedStructureInput, ProteinStructureMapping,
    ProteinStructureMappingStatus, Residue3D, StructureChainReport, StructureMetadata,
    StructureRecord, StructureSequenceChain, StructureSequenceOutput, StructureValidationIssue,
    StructureValidationIssueCode, StructureValidationReport, ValidatedStructureInput,
};
pub use validation::{
    extract_structure_sequences, summarize_structure_record, validate_structure_record,
};

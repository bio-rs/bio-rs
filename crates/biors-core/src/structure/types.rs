use serde::{Deserialize, Serialize};

use crate::error::Diagnostic;
use crate::formats::BioFormat;

/// Result of parsing a structure file from a reader.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ParsedStructureInput {
    /// Stable hash of the raw input stream.
    pub input_hash: String,
    /// Parsed structure record.
    pub record: StructureRecord,
}

/// Result of validating a structure file from a reader.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ValidatedStructureInput {
    /// Stable hash of the raw input stream.
    pub input_hash: String,
    /// Aggregate validation report.
    pub report: StructureValidationReport,
}

/// Parsed macromolecular structure record.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StructureRecord {
    /// Source file format.
    pub format: BioFormat,
    /// Optional stable accession from the source file.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// Source and aggregate metadata.
    pub metadata: StructureMetadata,
    /// Extracted chains in source order.
    pub chains: Vec<Chain>,
}

/// Aggregate metadata for a structure record.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StructureMetadata {
    /// Optional title from source metadata.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// Number of input lines consumed.
    pub line_count: usize,
    /// Number of MODEL blocks observed; defaults to one for flat PDB files.
    pub model_count: usize,
    /// Number of ATOM records parsed.
    pub atom_count: usize,
    /// Number of HETATM records parsed.
    pub hetero_atom_count: usize,
    /// Number of chains with SEQRES data.
    pub seqres_chain_count: usize,
    /// Number of residues listed in REMARK 465.
    pub missing_residue_count: usize,
}

/// One chain extracted from a structure record.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Chain {
    /// Chain identifier; blank PDB chain IDs are normalized to `_`.
    pub id: String,
    /// Residues with at least one coordinate-bearing atom.
    pub residues: Vec<Residue3D>,
    /// Coordinate-derived protein sequence, using `X` for unknown residues.
    pub coordinate_sequence: String,
    /// SEQRES-derived protein sequence when available.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seqres_sequence: Option<String>,
    /// Missing residues annotated by REMARK 465 for this chain.
    pub missing_residues: Vec<MissingResidue>,
}

/// One residue and its coordinate-bearing atoms.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Residue3D {
    /// Three-letter residue name.
    pub name: String,
    /// Residue sequence number.
    pub sequence_number: i32,
    /// Optional insertion code.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub insertion_code: Option<char>,
    /// True when the residue was sourced from HETATM records.
    pub hetero: bool,
    /// Protein one-letter code; unknown coordinate residues use `X`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub one_letter_code: Option<char>,
    /// Atoms belonging to this residue.
    pub atoms: Vec<Atom>,
}

/// One coordinate-bearing atom.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Atom {
    /// Atom serial number.
    pub serial: i32,
    /// Atom name.
    pub name: String,
    /// Optional alternate location indicator.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alternate_location: Option<char>,
    /// Optional element symbol.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub element: Option<String>,
    /// Cartesian coordinate.
    pub coordinate: Coordinate,
    /// Occupancy when present.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub occupancy: Option<f64>,
    /// Isotropic temperature factor when present.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature_factor: Option<f64>,
}

/// Cartesian coordinate in Angstroms.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Coordinate {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

/// Residue listed as present in sequence but absent from coordinates.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MissingResidue {
    /// Three-letter residue name.
    pub name: String,
    /// Chain identifier; blank PDB chain IDs are normalized to `_`.
    pub chain_id: String,
    /// Residue sequence number.
    pub sequence_number: i32,
    /// Optional insertion code.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub insertion_code: Option<char>,
}

/// Aggregate structure validation report.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StructureValidationReport {
    pub format: BioFormat,
    pub valid: bool,
    pub chains: usize,
    pub residues: usize,
    pub atoms: usize,
    pub hetero_atoms: usize,
    pub missing_residues: usize,
    pub warning_count: usize,
    pub error_count: usize,
    pub chain_reports: Vec<StructureChainReport>,
    pub warnings: Vec<StructureValidationIssue>,
    pub errors: Vec<StructureValidationIssue>,
}

/// Validation report for one structure chain.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StructureChainReport {
    pub chain_id: String,
    pub residues: usize,
    pub atoms: usize,
    pub coordinate_sequence_length: usize,
    pub seqres_sequence_length: Option<usize>,
    pub missing_residues: usize,
    pub sequence_mapping: ProteinStructureMapping,
}

/// Structure sequence extraction output.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StructureSequenceOutput {
    pub format: BioFormat,
    pub chains: Vec<StructureSequenceChain>,
}

/// Extracted sequence and mapping data for one chain.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StructureSequenceChain {
    pub chain_id: String,
    pub coordinate_sequence: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seqres_sequence: Option<String>,
    pub coordinate_residues: usize,
    pub seqres_residues: Option<usize>,
    pub missing_residues: Vec<MissingResidue>,
    pub mapping: ProteinStructureMapping,
}

/// Relationship between coordinate-derived protein sequence and SEQRES.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProteinStructureMapping {
    pub status: ProteinStructureMappingStatus,
    pub message: String,
    /// One-based SEQRES positions for each coordinate residue, or null when unmapped.
    pub coordinate_to_seqres_positions: Vec<Option<usize>>,
}

/// Protein sequence mapping status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProteinStructureMappingStatus {
    Exact,
    CoordinateSubsequence,
    MissingSeqres,
    Mismatch,
}

/// Stable structure validation issue code.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StructureValidationIssueCode {
    NoCoordinateChains,
    InvalidCoordinate,
    InvalidOccupancy,
    SuspiciousOccupancy,
    MissingElement,
    MissingResidue,
    UnknownResidue,
    SequenceMismatch,
}

impl StructureValidationIssueCode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::NoCoordinateChains => "no_coordinate_chains",
            Self::InvalidCoordinate => "invalid_coordinate",
            Self::InvalidOccupancy => "invalid_occupancy",
            Self::SuspiciousOccupancy => "suspicious_occupancy",
            Self::MissingElement => "missing_element",
            Self::MissingResidue => "missing_residue",
            Self::UnknownResidue => "unknown_residue",
            Self::SequenceMismatch => "sequence_mismatch",
        }
    }
}

/// Structure validation warning or error.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StructureValidationIssue {
    pub code: StructureValidationIssueCode,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chain_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub residue_number: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub atom_serial: Option<i32>,
}

impl StructureValidationIssue {
    pub(crate) fn new(code: StructureValidationIssueCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            chain_id: None,
            residue_number: None,
            atom_serial: None,
        }
    }

    pub(crate) fn with_chain(mut self, chain_id: impl Into<String>) -> Self {
        self.chain_id = Some(chain_id.into());
        self
    }

    pub(crate) fn with_residue(mut self, residue_number: i32) -> Self {
        self.residue_number = Some(residue_number);
        self
    }

    pub(crate) fn with_atom(mut self, atom_serial: i32) -> Self {
        self.atom_serial = Some(atom_serial);
        self
    }
}

impl Diagnostic for StructureValidationIssue {
    fn code(&self) -> &'static str {
        self.code.as_str()
    }

    fn message(&self) -> String {
        self.message.clone()
    }
}

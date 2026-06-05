use serde::{Deserialize, Serialize};

use crate::error::Diagnostic;
use crate::formats::{BioFormat, FormatField, FormatMetadata, FormatRecord};

/// Parsed molecular record.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MoleculeRecord {
    /// Source file format.
    pub format: BioFormat,
    /// Optional identifier or title from the source line.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// Raw source SMILES string.
    pub source: String,
    /// Source and aggregate metadata.
    pub metadata: MoleculeMetadata,
    /// Molecular graph parsed from the source string.
    pub graph: MolecularGraph,
    /// Format-specific metadata/properties preserved from SDF/MOL2 sources.
    pub properties: Vec<MoleculeProperty>,
}

impl MoleculeRecord {
    /// Project this molecule into the shared format-record contract.
    pub fn to_format_record(&self) -> FormatRecord {
        let mut fields = vec![
            FormatField::new("source", self.source.clone()),
            FormatField::new("atoms", self.metadata.atom_count.to_string()),
            FormatField::new("bonds", self.metadata.bond_count.to_string()),
        ];
        if let Some(id) = &self.id {
            fields.push(FormatField::new("id", id.clone()));
        }
        FormatRecord::new(
            self.format,
            self.id
                .clone()
                .unwrap_or_else(|| format!("record-{}", self.metadata.source.record_index + 1)),
            self.metadata.source.clone(),
            fields,
        )
    }
}

/// Result of parsing molecule records from a reader.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ParsedMoleculeInput {
    /// Stable hash of the raw input stream.
    pub input_hash: String,
    /// Parsed molecule records.
    pub records: Vec<MoleculeRecord>,
}

/// Result of validating molecule records from a reader.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ValidatedMoleculeInput {
    /// Stable hash of the raw input stream.
    pub input_hash: String,
    /// Aggregate validation report.
    pub report: MoleculeValidationReport,
}

/// Name/value property preserved from molecule file metadata.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MoleculeProperty {
    pub name: String,
    pub value: String,
}

impl MoleculeProperty {
    pub fn new(name: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            value: value.into(),
        }
    }
}

/// Aggregate source metadata for a molecular record.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MoleculeMetadata {
    /// Source line and record index.
    pub source: FormatMetadata,
    /// Number of parsed atoms.
    pub atom_count: usize,
    /// Number of parsed bonds.
    pub bond_count: usize,
    /// Number of branch starts in the source string.
    pub branch_count: usize,
    /// Number of completed ring-closure bonds.
    pub ring_closure_count: usize,
    /// Number of disconnected graph components.
    pub disconnected_component_count: usize,
    /// Number of atoms marked aromatic in source notation.
    pub aromatic_atom_count: usize,
}

/// Molecular graph split into stable atom and bond graph sections.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MolecularGraph {
    pub atoms: AtomGraph,
    pub bonds: BondGraph,
}

/// Atom graph section.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AtomGraph {
    pub atoms: Vec<MoleculeAtom>,
}

/// Bond graph section.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BondGraph {
    pub bonds: Vec<MoleculeBond>,
}

/// One molecular atom.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MoleculeAtom {
    /// Zero-based atom index.
    pub index: usize,
    /// Chemical element symbol, or `*` for wildcard atoms.
    pub element: String,
    /// Original atom token, including brackets when present.
    pub token: String,
    /// True when the source atom token was aromatic lower-case notation.
    pub aromatic: bool,
    /// True when the source atom came from bracket notation.
    pub bracketed: bool,
    /// Optional isotope mass number from bracket notation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub isotope: Option<u16>,
    /// Explicit hydrogens declared in bracket notation.
    pub explicit_hydrogens: u8,
    /// Formal charge parsed from bracket notation.
    pub charge: i8,
    /// Optional chirality marker from bracket notation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chirality: Option<String>,
    /// Optional atom class from bracket notation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub atom_class: Option<u32>,
    /// Optional 3D coordinate from SDF/MOL2 records.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub coordinate: Option<MoleculeCoordinate>,
    /// Optional source atom type such as a MOL2 atom type.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub atom_type: Option<String>,
    /// Optional partial charge from MOL2 or source properties.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub partial_charge: Option<f64>,
    /// Optional source substructure id from MOL2.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub substructure_id: Option<i32>,
    /// Optional source substructure name from MOL2.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub substructure_name: Option<String>,
}

/// Optional molecule-space coordinate in Angstroms.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct MoleculeCoordinate {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

/// One molecular bond.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MoleculeBond {
    /// Zero-based bond index.
    pub index: usize,
    /// Zero-based source atom index.
    pub source_atom: usize,
    /// Zero-based target atom index.
    pub target_atom: usize,
    /// Parsed bond order.
    pub order: BondOrder,
    /// True when this bond was created by a ring-closure token.
    pub ring_closure: bool,
    /// Optional directional marker from `/` or `\`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stereochemistry: Option<String>,
}

/// SMILES bond order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BondOrder {
    Single,
    Double,
    Triple,
    Quadruple,
    Aromatic,
}

impl BondOrder {
    /// Stable JSON string.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Single => "single",
            Self::Double => "double",
            Self::Triple => "triple",
            Self::Quadruple => "quadruple",
            Self::Aromatic => "aromatic",
        }
    }
}

/// Aggregate molecule validation report.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MoleculeValidationReport {
    pub format: BioFormat,
    pub valid: bool,
    pub records: usize,
    pub valid_records: usize,
    pub atom_count: usize,
    pub bond_count: usize,
    pub warning_count: usize,
    pub error_count: usize,
    pub record_reports: Vec<MoleculeValidationRecord>,
}

/// Validation report for one molecular record.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MoleculeValidationRecord {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub source: String,
    pub metadata: MoleculeMetadata,
    pub valid: bool,
    pub atoms: usize,
    pub bonds: usize,
    pub derived: MoleculeDerivedFeatures,
    pub warnings: Vec<MoleculeValidationIssue>,
    pub errors: Vec<MoleculeValidationIssue>,
}

/// Deterministic molecule features derived from the parsed graph.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MoleculeDerivedFeatures {
    pub canonical_graph_key: String,
    pub formula: String,
    pub exact_mass: f64,
    pub heavy_atom_count: usize,
    pub hetero_atom_count: usize,
    pub ring_bond_count: usize,
    pub rotatable_bond_count: usize,
    pub hydrogen_bond_donor_count: usize,
    pub hydrogen_bond_acceptor_count: usize,
    pub formal_charge: i32,
    pub fingerprint: MoleculeFingerprint,
}

/// Deterministic hashed molecular fingerprint.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MoleculeFingerprint {
    pub algorithm: String,
    pub bits: usize,
    pub set_bits: Vec<usize>,
}

/// Stable molecule validation issue code.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MoleculeValidationIssueCode {
    AromaticityNotVerified,
    ValenceExceeded,
    UnknownValenceModel,
}

impl MoleculeValidationIssueCode {
    /// Stable code string.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::AromaticityNotVerified => "aromaticity_not_verified",
            Self::ValenceExceeded => "valence_exceeded",
            Self::UnknownValenceModel => "unknown_valence_model",
        }
    }
}

/// Molecule validation warning or error.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MoleculeValidationIssue {
    pub code: MoleculeValidationIssueCode,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub atom_index: Option<usize>,
}

impl MoleculeValidationIssue {
    pub(crate) fn new(code: MoleculeValidationIssueCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            atom_index: None,
        }
    }

    pub(crate) fn with_atom(mut self, atom_index: usize) -> Self {
        self.atom_index = Some(atom_index);
        self
    }
}

impl Diagnostic for MoleculeValidationIssue {
    fn code(&self) -> &'static str {
        self.code.as_str()
    }

    fn message(&self) -> String {
        self.message.clone()
    }
}

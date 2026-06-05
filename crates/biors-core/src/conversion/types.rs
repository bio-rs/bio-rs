use serde::{Deserialize, Serialize};

use crate::formats::{BioFormat, FormatMetadata, FormatRecord};
use crate::molecule::{MoleculeDerivedFeatures, MoleculeRecord};
use crate::sequence::{SequenceKindDetection, SequenceRecord};
use crate::structure::{StructureRecord, StructureSequenceOutput};

/// Stable schema identifier for unified conversion JSON exports.
pub const CONVERSION_SCHEMA_VERSION: &str = "biors.conversion.v0";

/// Biological entity family represented by a unified conversion record.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BioEntityType {
    Sequence,
    Structure,
    Molecule,
}

/// Source format and optional source-record location for a converted entity.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConversionSource {
    pub format: BioFormat,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<FormatMetadata>,
}

impl ConversionSource {
    pub const fn new(format: BioFormat, metadata: Option<FormatMetadata>) -> Self {
        Self { format, metadata }
    }
}

/// Stable conversion-layer issue severity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConversionIssueSeverity {
    Warning,
    Error,
}

/// Stable conversion-layer issue code.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConversionIssueCode {
    SequenceAmbiguousSymbol,
    SequenceInvalidSymbol,
    EmptySequence,
    FastqQualityLengthMismatch,
    StructureValidationWarning,
    StructureValidationError,
    MoleculeValidationWarning,
    MoleculeValidationError,
}

impl ConversionIssueCode {
    /// Stable string used by JSON consumers and tests.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::SequenceAmbiguousSymbol => "sequence_ambiguous_symbol",
            Self::SequenceInvalidSymbol => "sequence_invalid_symbol",
            Self::EmptySequence => "empty_sequence",
            Self::FastqQualityLengthMismatch => "fastq_quality_length_mismatch",
            Self::StructureValidationWarning => "structure_validation_warning",
            Self::StructureValidationError => "structure_validation_error",
            Self::MoleculeValidationWarning => "molecule_validation_warning",
            Self::MoleculeValidationError => "molecule_validation_error",
        }
    }
}

/// Conversion warning or error with optional source-specific location metadata.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConversionIssue {
    pub severity: ConversionIssueSeverity,
    pub code: ConversionIssueCode,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_code: Option<String>,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chain_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub residue_number: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub atom_serial: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub atom_index: Option<usize>,
}

impl ConversionIssue {
    pub fn new(
        severity: ConversionIssueSeverity,
        code: ConversionIssueCode,
        message: impl Into<String>,
    ) -> Self {
        Self {
            severity,
            code,
            source_code: None,
            message: message.into(),
            position: None,
            chain_id: None,
            residue_number: None,
            atom_serial: None,
            atom_index: None,
        }
    }

    pub fn with_source_code(mut self, source_code: impl Into<String>) -> Self {
        self.source_code = Some(source_code.into());
        self
    }

    pub fn with_position(mut self, position: usize) -> Self {
        self.position = Some(position);
        self
    }

    pub fn with_chain_id(mut self, chain_id: impl Into<String>) -> Self {
        self.chain_id = Some(chain_id.into());
        self
    }

    pub fn with_residue_number(mut self, residue_number: i32) -> Self {
        self.residue_number = Some(residue_number);
        self
    }

    pub fn with_atom_serial(mut self, atom_serial: i32) -> Self {
        self.atom_serial = Some(atom_serial);
        self
    }

    pub fn with_atom_index(mut self, atom_index: usize) -> Self {
        self.atom_index = Some(atom_index);
        self
    }
}

/// Per-entity conversion validation and model-readiness state.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConversionValidation {
    pub valid: bool,
    pub model_ready: bool,
    pub warning_count: usize,
    pub error_count: usize,
    pub warnings: Vec<ConversionIssue>,
    pub errors: Vec<ConversionIssue>,
}

impl ConversionValidation {
    pub fn new(
        valid: bool,
        model_ready: bool,
        warnings: Vec<ConversionIssue>,
        errors: Vec<ConversionIssue>,
    ) -> Self {
        Self {
            valid,
            model_ready,
            warning_count: warnings.len(),
            error_count: errors.len(),
            warnings,
            errors,
        }
    }
}

/// Sequence payload projected into the existing kind-aware sequence record.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConvertedSequenceRecord {
    pub sequence: SequenceRecord,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quality: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auto_detection: Option<SequenceKindDetection>,
}

/// Structure payload preserving the parsed record and extracted chain sequences.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConvertedStructureRecord {
    pub record: StructureRecord,
    pub sequences: StructureSequenceOutput,
}

/// Molecule payload preserving the parsed record, shared projection, and features.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConvertedMoleculeRecord {
    pub record: MoleculeRecord,
    pub format_record: FormatRecord,
    pub derived: MoleculeDerivedFeatures,
}

/// Unified conversion payload.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "data", rename_all = "snake_case")]
pub enum ConversionRecord {
    Sequence(ConvertedSequenceRecord),
    Structure(Box<ConvertedStructureRecord>),
    Molecule(Box<ConvertedMoleculeRecord>),
}

/// Unified biological entity produced by the conversion layer.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BioEntity {
    pub id: String,
    pub entity_type: BioEntityType,
    pub source: ConversionSource,
    pub record: ConversionRecord,
    pub validation: ConversionValidation,
}

/// JSON-ready export wrapper for converted biological entities.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BioEntityJsonExport {
    pub schema_version: String,
    pub records: usize,
    pub valid_records: usize,
    pub model_ready_records: usize,
    pub warning_count: usize,
    pub error_count: usize,
    pub entities: Vec<BioEntity>,
}

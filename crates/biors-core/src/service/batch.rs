use crate::error::FastaReadError;
use crate::sequence::kind_validation::validate_fasta_reader_summary_with_kind_and_hash;
use crate::sequence::{KindAwareSequenceValidationSummary, SequenceKind, SequenceKindSelection};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::io::Cursor;

pub const SERVICE_BATCH_SEQUENCE_VALIDATE_SCHEMA_VERSION: &str =
    "biors.service_batch_sequence_validate.v0";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ServiceBatchSequenceValidateRequest {
    #[serde(default)]
    pub kind: ServiceSequenceKindSelection,
    pub inputs: Vec<ServiceBatchSequenceInput>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ServiceBatchSequenceInput {
    pub id: String,
    pub fasta_text: String,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ServiceSequenceKindSelection {
    #[default]
    Auto,
    Protein,
    Dna,
    Rna,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ServiceBatchSequenceValidateOutput {
    pub schema_version: String,
    pub inputs: usize,
    pub summary: ServiceBatchSequenceSummary,
    pub items: Vec<ServiceBatchSequenceItemReport>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ServiceBatchSequenceSummary {
    pub items: usize,
    pub fasta_records: usize,
    pub valid_records: usize,
    pub warning_count: usize,
    pub error_count: usize,
    pub kind_counts: crate::sequence::SequenceKindCounts,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ServiceBatchSequenceItemReport {
    pub id: String,
    pub input_hash: String,
    pub fasta_records: usize,
    pub valid_records: usize,
    pub warning_count: usize,
    pub error_count: usize,
    pub kind_counts: crate::sequence::SequenceKindCounts,
}

#[derive(Debug)]
pub enum ServiceBatchValidationError {
    EmptyInputs,
    EmptyInputId { index: usize },
    InvalidInputId { id: String },
    DuplicateInputId { id: String },
    EmptyFastaText { id: String },
    FastaRead { id: String, source: FastaReadError },
}

pub fn validate_service_batch_sequence_request(
    request: ServiceBatchSequenceValidateRequest,
) -> Result<ServiceBatchSequenceValidateOutput, ServiceBatchValidationError> {
    if request.inputs.is_empty() {
        return Err(ServiceBatchValidationError::EmptyInputs);
    }

    let mut seen = BTreeSet::new();
    let mut output = ServiceBatchSequenceValidateOutput {
        schema_version: SERVICE_BATCH_SEQUENCE_VALIDATE_SCHEMA_VERSION.to_string(),
        inputs: request.inputs.len(),
        summary: ServiceBatchSequenceSummary::default(),
        items: Vec::with_capacity(request.inputs.len()),
    };
    let selection = SequenceKindSelection::from(request.kind);

    for (index, input) in request.inputs.into_iter().enumerate() {
        if input.id.trim().is_empty() {
            return Err(ServiceBatchValidationError::EmptyInputId { index });
        }
        if !is_service_batch_input_id(&input.id) {
            return Err(ServiceBatchValidationError::InvalidInputId { id: input.id });
        }
        let id = input.id;
        if !seen.insert(id.clone()) {
            return Err(ServiceBatchValidationError::DuplicateInputId { id });
        }
        if input.fasta_text.trim().is_empty() {
            return Err(ServiceBatchValidationError::EmptyFastaText { id });
        }

        let reader = Cursor::new(input.fasta_text.into_bytes());
        let validated = validate_fasta_reader_summary_with_kind_and_hash(reader, selection)
            .map_err(|source| ServiceBatchValidationError::FastaRead {
                id: id.clone(),
                source,
            })?;
        output.summary.add_item(&validated.summary);
        output
            .items
            .push(ServiceBatchSequenceItemReport::from_summary(
                id,
                validated.input_hash,
                validated.summary,
            ));
    }

    Ok(output)
}

fn is_service_batch_input_id(id: &str) -> bool {
    id.chars()
        .all(|symbol| symbol.is_ascii_alphanumeric() || matches!(symbol, '.' | '_' | ':' | '-'))
}

impl From<ServiceSequenceKindSelection> for SequenceKindSelection {
    fn from(value: ServiceSequenceKindSelection) -> Self {
        match value {
            ServiceSequenceKindSelection::Auto => Self::Auto,
            ServiceSequenceKindSelection::Protein => Self::Explicit(SequenceKind::Protein),
            ServiceSequenceKindSelection::Dna => Self::Explicit(SequenceKind::Dna),
            ServiceSequenceKindSelection::Rna => Self::Explicit(SequenceKind::Rna),
        }
    }
}

impl ServiceBatchSequenceSummary {
    fn add_item(&mut self, summary: &KindAwareSequenceValidationSummary) {
        self.items += 1;
        self.fasta_records += summary.records;
        self.valid_records += summary.valid_records;
        self.warning_count += summary.warning_count;
        self.error_count += summary.error_count;
        self.kind_counts.protein += summary.kind_counts.protein;
        self.kind_counts.dna += summary.kind_counts.dna;
        self.kind_counts.rna += summary.kind_counts.rna;
    }
}

impl ServiceBatchSequenceItemReport {
    fn from_summary(
        id: String,
        input_hash: String,
        summary: KindAwareSequenceValidationSummary,
    ) -> Self {
        Self {
            id,
            input_hash,
            fasta_records: summary.records,
            valid_records: summary.valid_records,
            warning_count: summary.warning_count,
            error_count: summary.error_count,
            kind_counts: summary.kind_counts,
        }
    }
}

impl ServiceBatchValidationError {
    pub const fn code(&self) -> &'static str {
        match self {
            Self::EmptyInputs => "service.batch.no_inputs",
            Self::EmptyInputId { .. } => "service.batch.empty_input_id",
            Self::InvalidInputId { .. } => "service.batch.invalid_input_id",
            Self::DuplicateInputId { .. } => "service.batch.duplicate_input_id",
            Self::EmptyFastaText { .. } => "service.batch.empty_fasta_text",
            Self::FastaRead { .. } => "service.batch.invalid_fasta",
        }
    }

    pub fn location(&self) -> Option<String> {
        match self {
            Self::EmptyInputs => None,
            Self::EmptyInputId { index } => Some(format!("inputs[{index}].id")),
            Self::InvalidInputId { id } => Some(id.clone()),
            Self::DuplicateInputId { id }
            | Self::EmptyFastaText { id }
            | Self::FastaRead { id, .. } => Some(id.clone()),
        }
    }
}

impl std::fmt::Display for ServiceBatchValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyInputs => write!(f, "batch validation requires at least one input"),
            Self::EmptyInputId { index } => {
                write!(f, "batch input at index {index} has an empty id")
            }
            Self::InvalidInputId { id } => {
                write!(f, "batch input id '{id}' must match ^[A-Za-z0-9._:-]+$")
            }
            Self::DuplicateInputId { id } => write!(f, "batch input id '{id}' is duplicated"),
            Self::EmptyFastaText { id } => write!(f, "batch input '{id}' has empty FASTA text"),
            Self::FastaRead { id, source } => {
                write!(f, "batch input '{id}' is not valid FASTA: {source}")
            }
        }
    }
}

impl std::error::Error for ServiceBatchValidationError {}

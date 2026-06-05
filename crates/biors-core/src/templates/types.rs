use serde::Serialize;

use crate::formats::BioFormat;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskTemplateKind {
    ProteinClassification,
    ProteinEmbeddingGeneration,
    VariantEffectPrediction,
    MoleculePropertyPrediction,
    StructureValidation,
    SequenceSimilarityPreprocess,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TemplateEntity {
    ProteinSequence,
    ProteinVariant,
    Molecule,
    ProteinStructure,
    SequenceSet,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CoreReaderSupport {
    /// bio-rs has a deterministic parser or validator for this format.
    Executable,
    /// The template documents normalized fields but does not claim a core reader.
    ContractOnly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct TemplateInputFormat {
    pub format: BioFormat,
    pub core_reader: CoreReaderSupport,
}

impl TemplateInputFormat {
    pub const fn executable(format: BioFormat) -> Self {
        Self {
            format,
            core_reader: CoreReaderSupport::Executable,
        }
    }

    pub const fn contract_only(format: BioFormat) -> Self {
        Self {
            format,
            core_reader: CoreReaderSupport::ContractOnly,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct TemplateInput {
    pub entity: TemplateEntity,
    pub formats: &'static [TemplateInputFormat],
    pub required_fields: &'static [&'static str],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct TemplateValidation {
    pub id: &'static str,
    pub description: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct TemplateModelField {
    pub name: &'static str,
    pub description: &'static str,
    pub required: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct TemplateOutputExpectation {
    pub name: &'static str,
    pub description: &'static str,
    pub required: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct TemplateExecutionAssumptions {
    pub execution_mode: &'static str,
    pub network_access: &'static str,
    pub external_model_calls: bool,
    pub uploads_input_data: bool,
    pub persistence: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct TaskTemplate {
    pub schema_version: &'static str,
    pub id: &'static str,
    pub kind: TaskTemplateKind,
    pub title: &'static str,
    pub summary: &'static str,
    pub supported_inputs: &'static [TemplateInput],
    pub required_validations: &'static [TemplateValidation],
    pub model_ready_fields: &'static [TemplateModelField],
    pub output_expectations: &'static [TemplateOutputExpectation],
    pub execution: TemplateExecutionAssumptions,
}

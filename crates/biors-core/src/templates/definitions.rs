use super::fields::{
    CLASSIFICATION_FIELDS, EMBEDDING_FIELDS, MOLECULE_FIELDS, SEARCH_FIELDS, STRUCTURE_FIELDS,
    VARIANT_FIELDS,
};
use super::inputs::{
    MOLECULE_INPUT, PROTEIN_SEQUENCE_INPUT, SEARCH_INPUT, STRUCTURE_INPUT, VARIANT_INPUTS,
};
use super::outputs::{
    CLASSIFICATION_OUTPUTS, EMBEDDING_OUTPUTS, MOLECULE_OUTPUTS, SEARCH_OUTPUTS, STRUCTURE_OUTPUTS,
    VARIANT_OUTPUTS,
};
use super::validations::{
    MOLECULE_VALIDATIONS, PROTEIN_VALIDATIONS, SEARCH_VALIDATIONS, STRUCTURE_VALIDATIONS,
    VARIANT_VALIDATIONS,
};
use super::{
    TaskTemplate, TaskTemplateKind, TemplateExecutionAssumptions, TemplateInput,
    TemplateModelField, TemplateOutputExpectation, TemplateValidation,
};

pub const TASK_TEMPLATE_SCHEMA_VERSION: &str = "biors.task_template.v0";

const LOCAL_CONTRACT_EXECUTION: TemplateExecutionAssumptions = TemplateExecutionAssumptions {
    execution_mode: "local_template_contract",
    network_access: "none",
    external_model_calls: false,
    uploads_input_data: false,
    persistence: "caller_controlled",
};

static TASK_TEMPLATE_IDS: [&str; 6] = [
    "protein-classification-v0",
    "protein-embedding-generation-v0",
    "variant-effect-prediction-v0",
    "molecule-property-prediction-v0",
    "structure-validation-v0",
    "sequence-similarity-preprocess-v0",
];

static TASK_TEMPLATES: [TaskTemplate; 6] = [
    template(TemplateSpec {
        id: "protein-classification-v0",
        kind: TaskTemplateKind::ProteinClassification,
        title: "Protein classification",
        summary: "Contract for local protein sequence classification inputs and outputs.",
        supported_inputs: &PROTEIN_SEQUENCE_INPUT,
        required_validations: &PROTEIN_VALIDATIONS,
        model_ready_fields: &CLASSIFICATION_FIELDS,
        output_expectations: &CLASSIFICATION_OUTPUTS,
    }),
    template(TemplateSpec {
        id: "protein-embedding-generation-v0",
        kind: TaskTemplateKind::ProteinEmbeddingGeneration,
        title: "Protein embedding generation",
        summary: "Contract for local protein embedding input tensors and vector outputs.",
        supported_inputs: &PROTEIN_SEQUENCE_INPUT,
        required_validations: &PROTEIN_VALIDATIONS,
        model_ready_fields: &EMBEDDING_FIELDS,
        output_expectations: &EMBEDDING_OUTPUTS,
    }),
    template(TemplateSpec {
        id: "variant-effect-prediction-v0",
        kind: TaskTemplateKind::VariantEffectPrediction,
        title: "Variant effect prediction",
        summary: "Contract for protein variant effect inputs after reference validation.",
        supported_inputs: &VARIANT_INPUTS,
        required_validations: &VARIANT_VALIDATIONS,
        model_ready_fields: &VARIANT_FIELDS,
        output_expectations: &VARIANT_OUTPUTS,
    }),
    template(TemplateSpec {
        id: "molecule-property-prediction-v0",
        kind: TaskTemplateKind::MoleculePropertyPrediction,
        title: "Molecule property prediction",
        summary: "Contract for molecule graph features and caller-provided property outputs.",
        supported_inputs: &MOLECULE_INPUT,
        required_validations: &MOLECULE_VALIDATIONS,
        model_ready_fields: &MOLECULE_FIELDS,
        output_expectations: &MOLECULE_OUTPUTS,
    }),
    template(TemplateSpec {
        id: "structure-validation-v0",
        kind: TaskTemplateKind::StructureValidation,
        title: "Structure validation",
        summary: "Contract for deterministic PDB structure validation and chain summaries.",
        supported_inputs: &STRUCTURE_INPUT,
        required_validations: &STRUCTURE_VALIDATIONS,
        model_ready_fields: &STRUCTURE_FIELDS,
        output_expectations: &STRUCTURE_OUTPUTS,
    }),
    template(TemplateSpec {
        id: "sequence-similarity-preprocess-v0",
        kind: TaskTemplateKind::SequenceSimilarityPreprocess,
        title: "Sequence similarity preprocessing",
        summary: "Contract for local sequence normalization before caller-side search.",
        supported_inputs: &SEARCH_INPUT,
        required_validations: &SEARCH_VALIDATIONS,
        model_ready_fields: &SEARCH_FIELDS,
        output_expectations: &SEARCH_OUTPUTS,
    }),
];

pub fn task_templates() -> &'static [TaskTemplate] {
    &TASK_TEMPLATES
}

pub fn task_template_ids() -> &'static [&'static str] {
    &TASK_TEMPLATE_IDS
}

pub fn find_task_template(id: &str) -> Option<&'static TaskTemplate> {
    TASK_TEMPLATES.iter().find(|template| template.id == id)
}

struct TemplateSpec {
    id: &'static str,
    kind: TaskTemplateKind,
    title: &'static str,
    summary: &'static str,
    supported_inputs: &'static [TemplateInput],
    required_validations: &'static [TemplateValidation],
    model_ready_fields: &'static [TemplateModelField],
    output_expectations: &'static [TemplateOutputExpectation],
}

const fn template(spec: TemplateSpec) -> TaskTemplate {
    TaskTemplate {
        schema_version: TASK_TEMPLATE_SCHEMA_VERSION,
        id: spec.id,
        kind: spec.kind,
        title: spec.title,
        summary: spec.summary,
        supported_inputs: spec.supported_inputs,
        required_validations: spec.required_validations,
        model_ready_fields: spec.model_ready_fields,
        output_expectations: spec.output_expectations,
        execution: LOCAL_CONTRACT_EXECUTION,
    }
}

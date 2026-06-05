//! Deterministic task template contracts for local bio-AI workflows.
//!
//! Templates describe required inputs, validations, model-ready fields, output
//! expectations, and execution assumptions. They do not call models, open
//! network connections, or claim hosted inference behavior.

mod definitions;
mod fields;
mod inputs;
mod outputs;
mod types;
mod validations;

pub use definitions::{
    find_task_template, task_template_ids, task_templates, TASK_TEMPLATE_SCHEMA_VERSION,
};
pub use types::{
    CoreReaderSupport, TaskTemplate, TaskTemplateKind, TemplateEntity,
    TemplateExecutionAssumptions, TemplateInput, TemplateInputFormat, TemplateModelField,
    TemplateOutputExpectation, TemplateValidation,
};

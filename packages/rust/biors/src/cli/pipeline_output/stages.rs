use crate::cli::pipeline_config::ResolvedPipelineConfig;
use biors_core::workflow::SequenceWorkflowOutput;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub(super) struct PipelineStep {
    name: &'static str,
    status: &'static str,
    records: usize,
    warning_count: usize,
    error_count: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    output_sha256: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub(super) struct PipelinePlan {
    schema_version: String,
    name: String,
    input: String,
    stages: Vec<PipelinePlanStage>,
}

#[derive(Debug, Clone, Serialize)]
struct PipelinePlanStage {
    name: &'static str,
    operation: &'static str,
    detail: String,
}

#[derive(Debug, Clone, Copy)]
enum PipelineStage {
    Parse,
    Normalize,
    Validate,
    Tokenize,
    Export,
}

const LEGACY_STAGES: &[PipelineStage] = &[
    PipelineStage::Validate,
    PipelineStage::Tokenize,
    PipelineStage::Export,
];

const CONFIG_STAGES: &[PipelineStage] = &[
    PipelineStage::Parse,
    PipelineStage::Normalize,
    PipelineStage::Validate,
    PipelineStage::Tokenize,
    PipelineStage::Export,
];

pub(super) fn legacy_steps(workflow: &SequenceWorkflowOutput) -> Vec<PipelineStep> {
    executed_steps(LEGACY_STAGES, workflow)
}

pub(super) fn planned_steps() -> Vec<PipelineStep> {
    CONFIG_STAGES
        .iter()
        .map(|stage| stage.planned_step())
        .collect()
}

pub(super) fn config_steps(workflow: &SequenceWorkflowOutput) -> Vec<PipelineStep> {
    executed_steps(CONFIG_STAGES, workflow)
}

pub(super) fn pipeline_plan(resolved: &ResolvedPipelineConfig) -> PipelinePlan {
    PipelinePlan {
        schema_version: resolved.config.schema_version.to_string(),
        name: resolved.config.name.clone(),
        input: resolved.input_path.display().to_string(),
        stages: CONFIG_STAGES
            .iter()
            .map(|stage| stage.plan_stage(resolved))
            .collect(),
    }
}

fn executed_steps(
    stages: &[PipelineStage],
    workflow: &SequenceWorkflowOutput,
) -> Vec<PipelineStep> {
    stages
        .iter()
        .map(|stage| stage.executed_step(workflow))
        .collect()
}

fn status_from_errors(error_count: usize) -> &'static str {
    if error_count == 0 {
        "passed"
    } else {
        "failed"
    }
}

impl PipelineStage {
    fn name(self) -> &'static str {
        match self {
            Self::Parse => "parse",
            Self::Normalize => "normalize",
            Self::Validate => "validate",
            Self::Tokenize => "tokenize",
            Self::Export => "export",
        }
    }

    fn operation(self) -> &'static str {
        match self {
            Self::Parse => "parse FASTA input",
            Self::Normalize => "normalize sequence records",
            Self::Validate => "validate biological alphabet",
            Self::Tokenize => "tokenize normalized records",
            Self::Export => "export model-ready JSON",
        }
    }

    fn plan_detail(self, resolved: &ResolvedPipelineConfig) -> String {
        match self {
            Self::Parse => format!("read {}", resolved.declared_input_path),
            Self::Normalize => resolved.config.normalize.policy.clone(),
            Self::Validate => resolved.config.validate.kind.clone(),
            Self::Tokenize => resolved.config.tokenize.profile.clone(),
            Self::Export => format!(
                "max_length={}, padding={}",
                resolved.config.export.max_length, resolved.config.export.padding
            ),
        }
    }

    fn planned_step(self) -> PipelineStep {
        PipelineStep {
            name: self.name(),
            status: "planned",
            records: 0,
            warning_count: 0,
            error_count: 0,
            output_sha256: None,
        }
    }

    fn executed_step(self, workflow: &SequenceWorkflowOutput) -> PipelineStep {
        match self {
            Self::Parse | Self::Normalize => PipelineStep {
                name: self.name(),
                status: "passed",
                records: workflow.validation.records,
                warning_count: 0,
                error_count: 0,
                output_sha256: None,
            },
            Self::Validate => PipelineStep {
                name: self.name(),
                status: status_from_errors(workflow.validation.error_count),
                records: workflow.validation.records,
                warning_count: workflow.validation.warning_count,
                error_count: workflow.validation.error_count,
                output_sha256: None,
            },
            Self::Tokenize => {
                let summary = &workflow.tokenization.summary;
                PipelineStep {
                    name: self.name(),
                    status: status_from_errors(summary.error_count),
                    records: summary.records,
                    warning_count: summary.warning_count,
                    error_count: summary.error_count,
                    output_sha256: None,
                }
            }
            Self::Export => export_step(workflow),
        }
    }

    fn plan_stage(self, resolved: &ResolvedPipelineConfig) -> PipelinePlanStage {
        PipelinePlanStage {
            name: self.name(),
            operation: self.operation(),
            detail: self.plan_detail(resolved),
        }
    }
}

fn export_step(workflow: &SequenceWorkflowOutput) -> PipelineStep {
    PipelineStep {
        name: PipelineStage::Export.name(),
        status: if workflow.model_ready {
            "passed"
        } else {
            "blocked"
        },
        records: workflow
            .model_input
            .as_ref()
            .map(|input| input.records.len())
            .unwrap_or(0),
        warning_count: 0,
        error_count: if workflow.model_ready {
            0
        } else {
            workflow.readiness_issues.len()
        },
        output_sha256: workflow
            .model_ready
            .then(|| workflow.provenance.hashes.output_data_sha256.clone()),
    }
}

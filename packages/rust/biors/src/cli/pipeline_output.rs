use super::pipeline_config::{PipelineConfig, ResolvedPipelineConfig};
use biors_core::workflow::SequenceWorkflowOutput;
use serde::Serialize;
use std::path::Path;

#[derive(Debug, Serialize)]
pub(crate) struct PipelineOutput {
    pipeline: &'static str,
    ready: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    dry_run: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    explain_plan: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    config: Option<PipelineConfigSummary>,
    steps: Vec<PipelineStep>,
    #[serde(skip_serializing_if = "Option::is_none")]
    plan: Option<PipelinePlan>,
    pub(crate) workflow: Option<SequenceWorkflowOutput>,
}

#[derive(Debug, Serialize)]
struct PipelineStep {
    name: String,
    status: &'static str,
    records: usize,
    warning_count: usize,
    error_count: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    output_sha256: Option<String>,
}

#[derive(Debug, Serialize)]
struct PipelineConfigSummary {
    schema_version: String,
    name: String,
    input: String,
    export: String,
}

#[derive(Debug, Clone, Serialize)]
struct PipelinePlan {
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

impl PipelineOutput {
    pub(crate) fn from_workflow(workflow: SequenceWorkflowOutput) -> Self {
        Self {
            pipeline: "validate_tokenize_export.v0",
            ready: workflow.model_ready,
            dry_run: None,
            explain_plan: None,
            config: None,
            steps: legacy_steps(&workflow),
            plan: None,
            workflow: Some(workflow),
        }
    }

    pub(crate) fn dry_run(resolved: ResolvedPipelineConfig, explain_plan: bool) -> Self {
        Self {
            pipeline: "config_pipeline.v0",
            ready: true,
            dry_run: Some(true),
            explain_plan: Some(explain_plan),
            config: Some(PipelineConfigSummary::from_config(
                &resolved.config,
                &resolved.input_path,
            )),
            steps: planned_steps(),
            plan: Some(pipeline_plan(&resolved)),
            workflow: None,
        }
    }

    pub(crate) fn from_config_workflow(
        resolved: ResolvedPipelineConfig,
        explain_plan: bool,
        workflow: SequenceWorkflowOutput,
    ) -> Self {
        let ready = workflow.model_ready;
        Self {
            pipeline: "config_pipeline.v0",
            ready,
            dry_run: Some(false),
            explain_plan: Some(explain_plan),
            config: Some(PipelineConfigSummary::from_config(
                &resolved.config,
                &resolved.input_path,
            )),
            steps: config_steps(&workflow),
            plan: explain_plan.then(|| pipeline_plan(&resolved)),
            workflow: Some(workflow),
        }
    }
}

impl PipelineConfigSummary {
    fn from_config(config: &PipelineConfig, input_path: &Path) -> Self {
        Self {
            schema_version: config.schema_version.to_string(),
            name: config.name.clone(),
            input: input_path.display().to_string(),
            export: config.export.format.clone(),
        }
    }
}

fn legacy_steps(workflow: &SequenceWorkflowOutput) -> Vec<PipelineStep> {
    let validation = &workflow.validation;
    let tokenization = &workflow.tokenization.summary;
    let export_status = if workflow.model_ready {
        "passed"
    } else {
        "blocked"
    };

    vec![
        PipelineStep {
            name: "validate".to_string(),
            status: if validation.error_count == 0 {
                "passed"
            } else {
                "failed"
            },
            records: validation.records,
            warning_count: validation.warning_count,
            error_count: validation.error_count,
            output_sha256: None,
        },
        PipelineStep {
            name: "tokenize".to_string(),
            status: if tokenization.error_count == 0 {
                "passed"
            } else {
                "failed"
            },
            records: tokenization.records,
            warning_count: tokenization.warning_count,
            error_count: tokenization.error_count,
            output_sha256: None,
        },
        export_step(workflow, export_status),
    ]
}

fn planned_steps() -> Vec<PipelineStep> {
    ["parse", "normalize", "validate", "tokenize", "export"]
        .into_iter()
        .map(|name| PipelineStep {
            name: name.to_string(),
            status: "planned",
            records: 0,
            warning_count: 0,
            error_count: 0,
            output_sha256: None,
        })
        .collect()
}

fn config_steps(workflow: &SequenceWorkflowOutput) -> Vec<PipelineStep> {
    let validation = &workflow.validation;
    let tokenization = &workflow.tokenization.summary;
    let export_status = if workflow.model_ready {
        "passed"
    } else {
        "blocked"
    };

    vec![
        PipelineStep {
            name: "parse".to_string(),
            status: "passed",
            records: validation.records,
            warning_count: 0,
            error_count: 0,
            output_sha256: None,
        },
        PipelineStep {
            name: "normalize".to_string(),
            status: "passed",
            records: validation.records,
            warning_count: 0,
            error_count: 0,
            output_sha256: None,
        },
        PipelineStep {
            name: "validate".to_string(),
            status: if validation.error_count == 0 {
                "passed"
            } else {
                "failed"
            },
            records: validation.records,
            warning_count: validation.warning_count,
            error_count: validation.error_count,
            output_sha256: None,
        },
        PipelineStep {
            name: "tokenize".to_string(),
            status: if tokenization.error_count == 0 {
                "passed"
            } else {
                "failed"
            },
            records: tokenization.records,
            warning_count: tokenization.warning_count,
            error_count: tokenization.error_count,
            output_sha256: None,
        },
        export_step(workflow, export_status),
    ]
}

fn export_step(workflow: &SequenceWorkflowOutput, status: &'static str) -> PipelineStep {
    PipelineStep {
        name: "export".to_string(),
        status,
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

fn pipeline_plan(resolved: &ResolvedPipelineConfig) -> PipelinePlan {
    PipelinePlan {
        schema_version: resolved.config.schema_version.to_string(),
        name: resolved.config.name.clone(),
        input: resolved.input_path.display().to_string(),
        stages: vec![
            PipelinePlanStage {
                name: "parse",
                operation: "parse FASTA input",
                detail: format!("read {}", resolved.input_path.display()),
            },
            PipelinePlanStage {
                name: "normalize",
                operation: "normalize sequence records",
                detail: resolved.config.normalize.policy.clone(),
            },
            PipelinePlanStage {
                name: "validate",
                operation: "validate biological alphabet",
                detail: resolved.config.validate.kind.clone(),
            },
            PipelinePlanStage {
                name: "tokenize",
                operation: "tokenize normalized records",
                detail: resolved.config.tokenize.profile.clone(),
            },
            PipelinePlanStage {
                name: "export",
                operation: "export model-ready JSON",
                detail: format!(
                    "max_length={}, padding={}",
                    resolved.config.export.max_length, resolved.config.export.padding
                ),
            },
        ],
    }
}

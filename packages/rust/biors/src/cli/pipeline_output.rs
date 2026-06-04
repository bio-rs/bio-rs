use self::stages::{
    config_steps, legacy_steps, pipeline_plan, planned_steps, PipelinePlan, PipelineStep,
};
use super::pipeline_config::ResolvedPipelineConfig;
use biors_core::workflow::SequenceWorkflowOutput;
use serde::Serialize;

mod stages;

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
struct PipelineConfigSummary {
    schema_version: String,
    name: String,
    input: String,
    export: String,
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
            ready: false,
            dry_run: Some(true),
            explain_plan: Some(explain_plan),
            config: Some(PipelineConfigSummary::from_config(&resolved)),
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
            config: Some(PipelineConfigSummary::from_config(&resolved)),
            steps: config_steps(&workflow),
            plan: explain_plan.then(|| pipeline_plan(&resolved)),
            workflow: Some(workflow),
        }
    }
}

impl PipelineConfigSummary {
    fn from_config(resolved: &ResolvedPipelineConfig) -> Self {
        Self {
            schema_version: resolved.config.schema_version.to_string(),
            name: resolved.config.name.clone(),
            input: resolved.declared_input_path.clone(),
            export: resolved.config.export.format.clone(),
        }
    }
}

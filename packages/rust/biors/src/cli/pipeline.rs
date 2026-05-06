use super::{workflow::workflow_output, PaddingArg};
use crate::errors::CliError;
use crate::output::print_success;
use biors_core::workflow::SequenceWorkflowOutput;
use serde::Serialize;
use std::path::PathBuf;

pub(crate) fn run_pipeline(
    max_length: usize,
    pad_token_id: u8,
    padding: PaddingArg,
    path: PathBuf,
) -> Result<(), CliError> {
    let output = workflow_output("biors pipeline", max_length, pad_token_id, padding, path)?;
    let pipeline = PipelineOutput::from_workflow(output);
    print_success(
        Some(pipeline.workflow.provenance.input_hash.clone()),
        pipeline,
    )
}

#[derive(Debug, Serialize)]
struct PipelineOutput {
    pipeline: &'static str,
    ready: bool,
    steps: Vec<PipelineStep>,
    workflow: SequenceWorkflowOutput,
}

#[derive(Debug, Serialize)]
struct PipelineStep {
    name: &'static str,
    status: &'static str,
    records: usize,
    warning_count: usize,
    error_count: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    output_sha256: Option<String>,
}

impl PipelineOutput {
    fn from_workflow(workflow: SequenceWorkflowOutput) -> Self {
        let validation = &workflow.validation;
        let tokenization = &workflow.tokenization.summary;
        let export_status = if workflow.model_ready {
            "passed"
        } else {
            "blocked"
        };
        Self {
            pipeline: "validate_tokenize_export.v0",
            ready: workflow.model_ready,
            steps: vec![
                PipelineStep {
                    name: "validate",
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
                    name: "tokenize",
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
                PipelineStep {
                    name: "export",
                    status: export_status,
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
                },
            ],
            workflow,
        }
    }
}

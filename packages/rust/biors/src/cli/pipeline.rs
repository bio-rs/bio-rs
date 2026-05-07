use super::{
    pipeline_config::load_pipeline_config, pipeline_output::PipelineOutput,
    workflow::workflow_output, PaddingArg,
};
use crate::errors::CliError;
use crate::output::print_success;
use std::path::PathBuf;

pub(crate) fn run_pipeline(
    config: Option<PathBuf>,
    dry_run: bool,
    explain_plan: bool,
    max_length: Option<usize>,
    pad_token_id: u8,
    padding: PaddingArg,
    path: Option<PathBuf>,
) -> Result<(), CliError> {
    let pipeline = match config {
        Some(config_path) => {
            if max_length.is_some() || path.is_some() {
                return Err(CliError::Validation {
                    code: "pipeline.invalid_config",
                    message: "--config cannot be combined with --max-length or positional input"
                        .to_string(),
                    location: Some("pipeline".to_string()),
                });
            }
            run_config_pipeline(config_path, dry_run, explain_plan)?
        }
        None => run_legacy_pipeline(max_length, pad_token_id, padding, path)?,
    };

    print_success(
        pipeline
            .workflow
            .as_ref()
            .map(|workflow| workflow.provenance.input_hash.clone()),
        pipeline,
    )
}

fn run_legacy_pipeline(
    max_length: Option<usize>,
    pad_token_id: u8,
    padding: PaddingArg,
    path: Option<PathBuf>,
) -> Result<PipelineOutput, CliError> {
    let max_length = max_length.ok_or_else(|| CliError::Validation {
        code: "pipeline.invalid_config",
        message: "--max-length is required when --config is not used".to_string(),
        location: Some("max_length".to_string()),
    })?;
    let path = path.ok_or_else(|| CliError::Validation {
        code: "pipeline.invalid_config",
        message: "input path is required when --config is not used".to_string(),
        location: Some("path".to_string()),
    })?;
    let output = workflow_output("biors pipeline", max_length, pad_token_id, padding, path)?;
    Ok(PipelineOutput::from_workflow(output))
}

fn run_config_pipeline(
    config_path: PathBuf,
    dry_run: bool,
    explain_plan: bool,
) -> Result<PipelineOutput, CliError> {
    let resolved = load_pipeline_config(&config_path)?;
    if dry_run {
        return Ok(PipelineOutput::dry_run(resolved, explain_plan));
    }
    let workflow = workflow_output(
        "biors pipeline --config",
        resolved.config.export.max_length,
        resolved.config.export.pad_token_id,
        resolved.padding,
        resolved.input_path.clone(),
    )?;
    Ok(PipelineOutput::from_config_workflow(
        resolved,
        explain_plan,
        workflow,
    ))
}

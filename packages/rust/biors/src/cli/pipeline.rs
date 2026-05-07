use super::{
    pipeline_config::load_pipeline_config,
    pipeline_lock::{write_pipeline_lock, PipelineLockPackage},
    pipeline_output::PipelineOutput,
    workflow::workflow_output,
    PaddingArg,
};
use crate::errors::CliError;
use crate::input::read_package_manifest;
use crate::output::print_success;
use biors_core::package::validate_package_manifest_artifacts;
use std::path::PathBuf;

pub(crate) struct PipelineRunOptions {
    pub(crate) config: Option<PathBuf>,
    pub(crate) dry_run: bool,
    pub(crate) explain_plan: bool,
    pub(crate) package: Option<PathBuf>,
    pub(crate) write_lock: Option<PathBuf>,
    pub(crate) max_length: Option<usize>,
    pub(crate) pad_token_id: u8,
    pub(crate) padding: PaddingArg,
    pub(crate) path: Option<PathBuf>,
}

pub(crate) fn run_pipeline(options: PipelineRunOptions) -> Result<(), CliError> {
    let pipeline = match options.config {
        Some(config_path) => {
            if options.max_length.is_some() || options.path.is_some() {
                return Err(CliError::Validation {
                    code: "pipeline.invalid_config",
                    message: "--config cannot be combined with --max-length or positional input"
                        .to_string(),
                    location: Some("pipeline".to_string()),
                });
            }
            let package = load_lock_package(options.package, options.write_lock.is_some())?;
            run_config_pipeline(
                config_path,
                options.dry_run,
                options.explain_plan,
                options.write_lock,
                package,
            )?
        }
        None => {
            if options.write_lock.is_some() || options.package.is_some() {
                return Err(CliError::Validation {
                    code: "pipeline.invalid_config",
                    message: "--write-lock and --package require --config".to_string(),
                    location: Some("pipeline".to_string()),
                });
            }
            run_legacy_pipeline(
                options.max_length,
                options.pad_token_id,
                options.padding,
                options.path,
            )?
        }
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
    write_lock: Option<PathBuf>,
    package: Option<PipelineLockPackage>,
) -> Result<PipelineOutput, CliError> {
    if dry_run && write_lock.is_some() {
        return Err(CliError::Validation {
            code: "pipeline.invalid_config",
            message:
                "--write-lock requires an executed pipeline and cannot be combined with --dry-run"
                    .to_string(),
            location: Some("pipeline".to_string()),
        });
    }

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
    if let Some(lock_path) = write_lock {
        write_pipeline_lock(
            &lock_path,
            &config_path,
            &resolved,
            &workflow,
            package.as_ref(),
        )?;
    }
    Ok(PipelineOutput::from_config_workflow(
        resolved,
        explain_plan,
        workflow,
    ))
}

fn load_lock_package(
    package_path: Option<PathBuf>,
    lock_requested: bool,
) -> Result<Option<PipelineLockPackage>, CliError> {
    let Some(path) = package_path else {
        return Ok(None);
    };
    if !lock_requested {
        return Err(CliError::Validation {
            code: "pipeline.invalid_config",
            message: "--package is only used when --write-lock is supplied".to_string(),
            location: Some("pipeline".to_string()),
        });
    }

    let (manifest, base_dir) = read_package_manifest(path.clone())?;
    let validation = validate_package_manifest_artifacts(&manifest, &base_dir);
    if !validation.valid {
        return Err(CliError::Validation {
            code: "pipeline.invalid_lock_package",
            message: format!("{:?}", validation.issues),
            location: Some(path.display().to_string()),
        });
    }

    Ok(Some(PipelineLockPackage {
        manifest_path: path,
        manifest,
    }))
}

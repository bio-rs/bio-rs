use super::{
    package::validate_cli_package_manifest_artifacts,
    pipeline_config::load_pipeline_config,
    pipeline_lock::{write_pipeline_lock, PipelineLockPackage},
    pipeline_output::PipelineOutput,
    workflow::{workflow_output, workflow_output_with_invocation_path},
    PaddingArg, TokenizerProfileArg,
};
use crate::errors::CliError;
use crate::input::read_package_manifest;
use crate::output::print_success;
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
            run_config_pipeline(
                config_path,
                options.dry_run,
                options.explain_plan,
                options.write_lock,
                options.package,
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
    let output = workflow_output(
        "biors pipeline",
        TokenizerProfileArg::Protein20,
        max_length,
        pad_token_id,
        padding,
        path,
    )?;
    Ok(PipelineOutput::from_workflow(output))
}

fn run_config_pipeline(
    config_path: PathBuf,
    dry_run: bool,
    explain_plan: bool,
    write_lock: Option<PathBuf>,
    package_path: Option<PathBuf>,
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
    let package = load_lock_package(package_path, write_lock.is_some(), &config_path)?;
    if dry_run {
        return Ok(PipelineOutput::dry_run(resolved, explain_plan));
    }
    let workflow = workflow_output_with_invocation_path(
        "biors pipeline --config",
        resolved.profile,
        resolved.config.export.max_length,
        resolved.config.export.pad_token_id,
        resolved.padding,
        resolved.input_path.clone(),
        PathBuf::from(&resolved.declared_input_path),
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
    config_path: &std::path::Path,
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
    let validation = validate_cli_package_manifest_artifacts(&manifest, &base_dir, &path);
    if !validation.valid {
        return Err(CliError::Validation {
            code: "pipeline.invalid_lock_package",
            message: format!("{:?}", validation.issues),
            location: Some(path.display().to_string()),
        });
    }
    let pipeline_config_path = manifest_declared_config_path(&manifest, &base_dir, config_path)?;

    Ok(Some(PipelineLockPackage {
        base_dir,
        manifest_path: path,
        pipeline_config_path,
        manifest,
    }))
}

fn manifest_declared_config_path(
    manifest: &biors_core::package::PackageManifest,
    base_dir: &std::path::Path,
    config_path: &std::path::Path,
) -> Result<String, CliError> {
    let config_canonical = canonicalize_lock_path(config_path)?;
    let declared_config = manifest
        .preprocessing
        .iter()
        .chain(manifest.postprocessing.iter())
        .filter_map(|step| step.config.as_ref())
        .find(|config| {
            canonicalize_lock_path(&base_dir.join(&config.path))
                .is_ok_and(|declared| declared == config_canonical)
        });

    if let Some(config) = declared_config {
        return Ok(config.path.clone());
    }

    Err(CliError::Validation {
        code: "pipeline.lock_config_not_in_package",
        message: format!(
            "--config '{}' is not declared by the supplied package manifest",
            config_path.display()
        ),
        location: Some("pipeline.config".to_string()),
    })
}

fn canonicalize_lock_path(path: &std::path::Path) -> Result<PathBuf, CliError> {
    path.canonicalize().map_err(|source| CliError::Read {
        path: path.to_path_buf(),
        source,
    })
}

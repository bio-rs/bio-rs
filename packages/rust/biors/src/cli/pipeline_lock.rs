use super::pipeline_config::ResolvedPipelineConfig;
use crate::errors::CliError;
use biors_core::{
    package::{sha256_digest, PackageManifest},
    workflow::SequenceWorkflowOutput,
};
use serde::Serialize;
use std::path::{Path, PathBuf};

pub(crate) struct PipelineLockPackage {
    pub(crate) manifest_path: PathBuf,
    pub(crate) manifest: PackageManifest,
}

#[derive(Debug, Serialize)]
struct PipelineLockfile {
    schema_version: &'static str,
    generated_by: PipelineLockGenerator,
    pipeline_config: PipelineLockConfig,
    package: Option<PipelineLockPackageSection>,
    execution: PipelineLockExecution,
    hashes: PipelineLockHashes,
    python_baseline: PipelineLockPythonBaseline,
}

#[derive(Debug, Serialize)]
struct PipelineLockGenerator {
    biors_version: &'static str,
    biors_core_version: String,
}

#[derive(Debug, Serialize)]
struct PipelineLockConfig {
    schema_version: String,
    name: String,
    path: String,
    sha256: String,
}

#[derive(Debug, Serialize)]
struct PipelineLockPackageSection {
    name: String,
    schema_version: String,
    manifest_path: String,
    model_sha256: String,
    runtime_backend: String,
    runtime_target: String,
    backend_version: String,
}

#[derive(Debug, Serialize)]
struct PipelineLockExecution {
    command: String,
    arguments: Vec<String>,
    input_path: String,
    input_hash: String,
    ready: bool,
}

#[derive(Debug, Serialize)]
struct PipelineLockHashes {
    vocabulary_sha256: String,
    output_data_sha256: String,
}

#[derive(Debug, Serialize)]
struct PipelineLockPythonBaseline {
    comparison: &'static str,
    reference: &'static str,
    status: &'static str,
}

pub(crate) fn write_pipeline_lock(
    lock_path: &Path,
    config_path: &Path,
    resolved: &ResolvedPipelineConfig,
    workflow: &SequenceWorkflowOutput,
    package: Option<&PipelineLockPackage>,
) -> Result<(), CliError> {
    let lockfile = build_pipeline_lock(config_path, resolved, workflow, package)?;
    let json = serde_json::to_string_pretty(&lockfile).map_err(CliError::Serialization)?;
    if let Some(parent) = lock_path
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
    {
        std::fs::create_dir_all(parent).map_err(CliError::Write)?;
    }
    std::fs::write(lock_path, format!("{json}\n")).map_err(CliError::Write)
}

fn build_pipeline_lock(
    config_path: &Path,
    resolved: &ResolvedPipelineConfig,
    workflow: &SequenceWorkflowOutput,
    package: Option<&PipelineLockPackage>,
) -> Result<PipelineLockfile, CliError> {
    Ok(PipelineLockfile {
        schema_version: "biors.pipeline.lock.v0",
        generated_by: PipelineLockGenerator {
            biors_version: env!("CARGO_PKG_VERSION"),
            biors_core_version: workflow.provenance.biors_core_version.clone(),
        },
        pipeline_config: PipelineLockConfig {
            schema_version: resolved.config.schema_version.to_string(),
            name: resolved.config.name.clone(),
            path: config_path.display().to_string(),
            sha256: file_sha256(config_path)?,
        },
        package: package.map(package_lock_section).transpose()?,
        execution: PipelineLockExecution {
            command: workflow.provenance.invocation.command.clone(),
            arguments: workflow.provenance.invocation.arguments.clone(),
            input_path: resolved.input_path.display().to_string(),
            input_hash: workflow.provenance.input_hash.clone(),
            ready: workflow.model_ready,
        },
        hashes: PipelineLockHashes {
            vocabulary_sha256: workflow.provenance.hashes.vocabulary_sha256.clone(),
            output_data_sha256: workflow.provenance.hashes.output_data_sha256.clone(),
        },
        python_baseline: PipelineLockPythonBaseline {
            comparison: "normalized_records_and_protein20_tokens",
            reference: "examples/model-input-contract/reference-python-parity.json",
            status: "strategy_recorded",
        },
    })
}

fn package_lock_section(
    package: &PipelineLockPackage,
) -> Result<PipelineLockPackageSection, CliError> {
    let model_sha256 =
        package
            .manifest
            .model
            .checksum
            .clone()
            .ok_or_else(|| CliError::Validation {
                code: "pipeline.lock_requires_model_checksum",
                message: "package model.checksum is required for pipeline.lock".to_string(),
                location: Some("package.model.checksum".to_string()),
            })?;
    let backend_version = package
        .manifest
        .runtime
        .version
        .clone()
        .unwrap_or_else(|| format!("{}.v0", package.manifest.runtime.backend));

    Ok(PipelineLockPackageSection {
        name: package.manifest.name.clone(),
        schema_version: package.manifest.schema_version.to_string(),
        manifest_path: package.manifest_path.display().to_string(),
        model_sha256,
        runtime_backend: package.manifest.runtime.backend.to_string(),
        runtime_target: package.manifest.runtime.target.to_string(),
        backend_version,
    })
}

fn file_sha256(path: &Path) -> Result<String, CliError> {
    let bytes = std::fs::read(path).map_err(|source| CliError::Read {
        path: path.to_path_buf(),
        source,
    })?;
    Ok(sha256_digest(&bytes))
}

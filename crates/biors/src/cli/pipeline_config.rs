use crate::cli::{PaddingArg, TokenizerProfileArg};
use crate::errors::CliError;
use biors_core::package::{
    parse_pipeline_config, validate_pipeline_config, PipelineConfig, ReferencedConfigError,
};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ResolvedPipelineConfig {
    pub config: PipelineConfig,
    pub declared_input_path: String,
    pub input_path: PathBuf,
    pub padding: PaddingArg,
    pub profile: TokenizerProfileArg,
}

pub(crate) fn load_pipeline_config(path: &Path) -> Result<ResolvedPipelineConfig, CliError> {
    let contents = std::fs::read_to_string(path).map_err(|source| CliError::Read {
        path: path.to_path_buf(),
        source,
    })?;
    let config = parse_pipeline_config(path, &contents).map_err(referenced_config_to_cli)?;
    let profile = validate_pipeline_config(&config).map_err(referenced_config_to_cli)?;
    Ok(ResolvedPipelineConfig {
        declared_input_path: config.input.path.clone(),
        input_path: resolve_input_path(path, &config.input.path),
        padding: parse_padding(&config.export.padding)?,
        profile: profile.into(),
        config,
    })
}

fn parse_padding(value: &str) -> Result<PaddingArg, CliError> {
    match value {
        "fixed_length" => Ok(PaddingArg::FixedLength),
        "no_padding" => Ok(PaddingArg::NoPadding),
        _ => Err(CliError::Validation {
            code: "pipeline.invalid_config",
            message: "export.padding must be 'fixed_length' or 'no_padding'".to_string(),
            location: Some("export.padding".to_string()),
        }),
    }
}

fn resolve_input_path(config_path: &Path, input_path: &str) -> PathBuf {
    let path = PathBuf::from(input_path);
    if path.is_absolute() {
        return path;
    }
    config_path
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join(path)
}

fn referenced_config_to_cli(error: ReferencedConfigError) -> CliError {
    CliError::Validation {
        code: "pipeline.invalid_config",
        message: error.message,
        location: error.location,
    }
}

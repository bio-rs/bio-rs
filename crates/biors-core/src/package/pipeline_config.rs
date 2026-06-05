use super::{PipelineConfigVersion, ReferencedConfigError};
use crate::tokenizer::ProteinTokenizerProfile;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

mod toml;
mod validation;

const PIPELINE_ERROR_CODE: &str = "pipeline.invalid_config";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PipelineConfig {
    pub schema_version: PipelineConfigVersion,
    pub name: String,
    pub input: PipelineInputConfig,
    pub normalize: PipelineNormalizeConfig,
    pub validate: PipelineValidateConfig,
    pub tokenize: PipelineTokenizeConfig,
    pub export: PipelineExportConfig,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PipelineInputConfig {
    pub format: String,
    pub path: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PipelineNormalizeConfig {
    pub policy: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PipelineValidateConfig {
    pub kind: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PipelineTokenizeConfig {
    pub profile: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PipelineExportConfig {
    pub format: String,
    pub max_length: usize,
    #[serde(default)]
    pub pad_token_id: u8,
    #[serde(default = "default_padding")]
    pub padding: String,
}

pub fn parse_pipeline_config(
    path: &Path,
    contents: &str,
) -> Result<PipelineConfig, ReferencedConfigError> {
    match extension(path).as_deref() {
        Some("json") => serde_json::from_str(contents).map_err(|error| {
            referenced_config_error(
                format!("invalid pipeline config JSON: {error}"),
                Some(path.display().to_string()),
            )
        }),
        Some("toml") => toml::parse_toml_pipeline_config(contents),
        Some(extension) => Err(referenced_config_error(
            format!("unsupported pipeline config extension: {extension}"),
            Some(path.display().to_string()),
        )),
        None => Err(referenced_config_error(
            "pipeline config path must have .json or .toml extension",
            Some(path.display().to_string()),
        )),
    }
}

pub fn validate_pipeline_config(
    config: &PipelineConfig,
) -> Result<ProteinTokenizerProfile, ReferencedConfigError> {
    validation::validate_pipeline_config_fields(config)
}

pub fn validate_pipeline_config_artifact(
    package_root: &Path,
    config_path: &Path,
) -> Result<(), ReferencedConfigError> {
    let contents = std::fs::read_to_string(config_path).map_err(|source| {
        referenced_config_error(
            format!("failed to read pipeline config: {source}"),
            Some(config_path.display().to_string()),
        )
    })?;
    let config = parse_pipeline_config(config_path, &contents)?;
    validate_pipeline_config(&config)?;
    validate_pipeline_input_path(package_root, config_path, &config.input.path)
}

fn validate_pipeline_input_path(
    package_root: &Path,
    config_path: &Path,
    input_path: &str,
) -> Result<(), ReferencedConfigError> {
    let declared = Path::new(input_path);
    if declared.is_absolute() {
        return Err(referenced_config_error(
            "package pipeline input.path must be package-relative",
            Some("input.path".to_string()),
        ));
    }
    let config_dir = config_path.parent().unwrap_or_else(|| Path::new("."));
    let package_root = canonicalize_pipeline_path(package_root)?;
    let input = canonicalize_pipeline_path(&config_dir.join(declared))?;
    if !input.starts_with(&package_root) {
        return Err(referenced_config_error(
            "package pipeline input.path must stay inside the package root",
            Some("input.path".to_string()),
        ));
    }
    Ok(())
}

fn canonicalize_pipeline_path(path: &Path) -> Result<PathBuf, ReferencedConfigError> {
    std::fs::canonicalize(path).map_err(|source| {
        referenced_config_error(
            format!("failed to canonicalize '{}': {source}", path.display()),
            Some(path.display().to_string()),
        )
    })
}

fn extension(path: &Path) -> Option<String> {
    path.extension()
        .and_then(|value| value.to_str())
        .map(str::to_ascii_lowercase)
}

pub(super) fn referenced_config_error(
    message: impl Into<String>,
    location: Option<String>,
) -> ReferencedConfigError {
    ReferencedConfigError::new(PIPELINE_ERROR_CODE, message, location)
}

pub(super) fn default_padding() -> String {
    "fixed_length".to_string()
}

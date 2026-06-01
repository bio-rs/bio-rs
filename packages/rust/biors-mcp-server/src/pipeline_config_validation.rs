use crate::pipeline_config_parse::{parse_pipeline_config, McpPipelineConfig};
use biors_core::package::{PipelineConfigVersion, ReferencedConfigError};
use std::path::{Path, PathBuf};

pub(crate) fn validate_pipeline_config_artifact(
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
    validate_pipeline_input_path(package_root, config_path, &config.input.path)?;
    Ok(())
}

fn validate_pipeline_config(config: &McpPipelineConfig) -> Result<(), ReferencedConfigError> {
    require_pipeline_version(config.schema_version)?;
    require_pipeline_non_empty("name", &config.name)?;
    require_pipeline_value("input.format", &config.input.format, "fasta")?;
    require_pipeline_non_empty("input.path", &config.input.path)?;
    require_pipeline_value(
        "normalize.policy",
        &config.normalize.policy,
        "strip_ascii_whitespace_uppercase",
    )?;
    require_pipeline_value("validate.kind", &config.validate.kind, "protein")?;
    require_pipeline_value("tokenize.profile", &config.tokenize.profile, "protein-20")?;
    require_pipeline_value("export.format", &config.export.format, "model-input-json")?;
    require_pipeline_max_length(config.export.max_length)?;
    require_pipeline_padding(&config.export.padding)?;
    let _ = config.export.pad_token_id;
    Ok(())
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

fn require_pipeline_version(version: PipelineConfigVersion) -> Result<(), ReferencedConfigError> {
    if version != PipelineConfigVersion::BiorsPipelineV0 {
        return Err(referenced_config_error(
            "schema_version must be biors.pipeline.v0",
            Some("schema_version".to_string()),
        ));
    }
    Ok(())
}

fn require_pipeline_max_length(max_length: usize) -> Result<(), ReferencedConfigError> {
    if max_length == 0 {
        return Err(referenced_config_error(
            "export.max_length must be greater than zero",
            Some("export.max_length".to_string()),
        ));
    }
    Ok(())
}

fn require_pipeline_padding(padding: &str) -> Result<(), ReferencedConfigError> {
    if !matches!(padding, "fixed_length" | "no_padding") {
        return Err(referenced_config_error(
            format!("export.padding must be fixed_length or no_padding, got {padding}"),
            Some("export.padding".to_string()),
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

fn require_pipeline_non_empty(field: &str, value: &str) -> Result<(), ReferencedConfigError> {
    if value.trim().is_empty() {
        return Err(referenced_config_error(
            format!("{field} is required"),
            Some(field.to_string()),
        ));
    }
    Ok(())
}

fn require_pipeline_value(
    field: &str,
    actual: &str,
    expected: &str,
) -> Result<(), ReferencedConfigError> {
    if actual != expected {
        return Err(referenced_config_error(
            format!("{field} must be {expected}, got {actual}"),
            Some(field.to_string()),
        ));
    }
    Ok(())
}

fn referenced_config_error(
    message: impl Into<String>,
    location: Option<String>,
) -> ReferencedConfigError {
    ReferencedConfigError::new("pipeline.invalid_config", message, location)
}

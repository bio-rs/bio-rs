use biors_core::package::{PipelineConfigVersion, ReferencedConfigError};
use serde::Deserialize;
use std::collections::BTreeMap;
use std::path::Path;

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct McpPipelineConfig {
    pub(crate) schema_version: PipelineConfigVersion,
    pub(crate) name: String,
    pub(crate) input: McpPipelineInputConfig,
    pub(crate) normalize: McpPipelineNormalizeConfig,
    pub(crate) validate: McpPipelineValidateConfig,
    pub(crate) tokenize: McpPipelineTokenizeConfig,
    pub(crate) export: McpPipelineExportConfig,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct McpPipelineInputConfig {
    pub(crate) format: String,
    pub(crate) path: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct McpPipelineNormalizeConfig {
    pub(crate) policy: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct McpPipelineValidateConfig {
    pub(crate) kind: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct McpPipelineTokenizeConfig {
    pub(crate) profile: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct McpPipelineExportConfig {
    pub(crate) format: String,
    pub(crate) max_length: usize,
    #[serde(default)]
    pub(crate) pad_token_id: u8,
    #[serde(default = "default_padding")]
    pub(crate) padding: String,
}

pub(crate) fn parse_pipeline_config(
    path: &Path,
    contents: &str,
) -> Result<McpPipelineConfig, ReferencedConfigError> {
    match path.extension().and_then(|value| value.to_str()) {
        Some("json") => serde_json::from_str(contents).map_err(|error| {
            referenced_config_error(
                format!("invalid pipeline config JSON: {error}"),
                Some(path.display().to_string()),
            )
        }),
        Some("toml") => parse_toml_pipeline_config(contents),
        Some("yaml") | Some("yml") => parse_yaml_pipeline_config(contents),
        Some(extension) => Err(referenced_config_error(
            format!("unsupported pipeline config extension: {extension}"),
            Some(path.display().to_string()),
        )),
        None => Err(referenced_config_error(
            "pipeline config path must have .json, .toml, .yaml, or .yml extension",
            Some(path.display().to_string()),
        )),
    }
}

fn parse_toml_pipeline_config(contents: &str) -> Result<McpPipelineConfig, ReferencedConfigError> {
    let mut values = BTreeMap::new();
    let mut section = String::new();

    for raw_line in contents.lines() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some(name) = line
            .strip_prefix('[')
            .and_then(|value| value.strip_suffix(']'))
        {
            section = name.trim().to_string();
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            return Err(referenced_config_error(
                format!("invalid pipeline config TOML line: {line}"),
                None,
            ));
        };
        let field = field_name(&section, key.trim());
        values.insert(field, unquote_value(value.trim()).to_string());
    }

    pipeline_config_from_values(values)
}

fn parse_yaml_pipeline_config(contents: &str) -> Result<McpPipelineConfig, ReferencedConfigError> {
    let mut values = BTreeMap::new();
    let mut section = String::new();

    for raw_line in contents.lines() {
        let trimmed = raw_line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        if !raw_line.starts_with(' ') && trimmed.ends_with(':') {
            section = trimmed.trim_end_matches(':').to_string();
            continue;
        }
        let Some((key, value)) = trimmed.split_once(':') else {
            return Err(referenced_config_error(
                format!("invalid pipeline config YAML line: {trimmed}"),
                None,
            ));
        };
        let field = field_name(&section, key.trim());
        values.insert(field, unquote_value(value.trim()).to_string());
    }

    pipeline_config_from_values(values)
}

fn pipeline_config_from_values(
    values: BTreeMap<String, String>,
) -> Result<McpPipelineConfig, ReferencedConfigError> {
    require_schema_version(&values)?;
    Ok(McpPipelineConfig {
        schema_version: PipelineConfigVersion::BiorsPipelineV0,
        name: required_value(&values, "name")?.to_string(),
        input: McpPipelineInputConfig {
            format: required_value(&values, "input.format")?.to_string(),
            path: required_value(&values, "input.path")?.to_string(),
        },
        normalize: McpPipelineNormalizeConfig {
            policy: required_value(&values, "normalize.policy")?.to_string(),
        },
        validate: McpPipelineValidateConfig {
            kind: required_value(&values, "validate.kind")?.to_string(),
        },
        tokenize: McpPipelineTokenizeConfig {
            profile: required_value(&values, "tokenize.profile")?.to_string(),
        },
        export: McpPipelineExportConfig {
            format: required_value(&values, "export.format")?.to_string(),
            max_length: parse_usize(&values, "export.max_length")?,
            pad_token_id: parse_optional_u8(&values, "export.pad_token_id")?,
            padding: values
                .get("export.padding")
                .cloned()
                .unwrap_or_else(default_padding),
        },
    })
}

fn require_schema_version(values: &BTreeMap<String, String>) -> Result<(), ReferencedConfigError> {
    let schema_version = required_value(values, "schema_version")?;
    if schema_version != "biors.pipeline.v0" {
        return Err(referenced_config_error(
            "schema_version must be biors.pipeline.v0",
            Some("schema_version".to_string()),
        ));
    }
    Ok(())
}

fn parse_usize(
    values: &BTreeMap<String, String>,
    field: &str,
) -> Result<usize, ReferencedConfigError> {
    required_value(values, field)?
        .parse::<usize>()
        .map_err(|_| {
            referenced_config_error(
                format!("{field} must be an unsigned integer"),
                Some(field.to_string()),
            )
        })
}

fn parse_optional_u8(
    values: &BTreeMap<String, String>,
    field: &str,
) -> Result<u8, ReferencedConfigError> {
    values
        .get(field)
        .map(|value| value.parse::<u8>())
        .transpose()
        .map_err(|_| {
            referenced_config_error(
                format!("{field} must be an unsigned 8-bit integer"),
                Some(field.to_string()),
            )
        })
        .map(Option::unwrap_or_default)
}

fn required_value<'a>(
    values: &'a BTreeMap<String, String>,
    field: &str,
) -> Result<&'a str, ReferencedConfigError> {
    values
        .get(field)
        .map(String::as_str)
        .ok_or_else(|| referenced_config_error(format!("{field} is required"), Some(field.into())))
}

fn field_name(section: &str, key: &str) -> String {
    if section.is_empty() {
        key.to_string()
    } else {
        format!("{section}.{key}")
    }
}

fn unquote_value(value: &str) -> &str {
    value
        .split_once(" #")
        .map(|(value, _)| value)
        .unwrap_or(value)
        .trim()
        .trim_matches('"')
        .trim_matches('\'')
}

fn referenced_config_error(
    message: impl Into<String>,
    location: Option<String>,
) -> ReferencedConfigError {
    ReferencedConfigError::new("pipeline.invalid_config", message, location)
}

fn default_padding() -> String {
    "fixed_length".to_string()
}

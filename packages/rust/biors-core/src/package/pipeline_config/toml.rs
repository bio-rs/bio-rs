use super::{
    default_padding, referenced_config_error, PipelineConfig, PipelineExportConfig,
    PipelineInputConfig, PipelineNormalizeConfig, PipelineTokenizeConfig, PipelineValidateConfig,
};
use crate::package::{PipelineConfigVersion, ReferencedConfigError};
use std::collections::BTreeMap;

pub(super) fn parse_toml_pipeline_config(
    contents: &str,
) -> Result<PipelineConfig, ReferencedConfigError> {
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
            if !matches!(
                section.as_str(),
                "input" | "normalize" | "validate" | "tokenize" | "export"
            ) {
                return Err(referenced_config_error(
                    format!("unknown section '{section}'"),
                    Some(section.clone()),
                ));
            }
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            return Err(referenced_config_error(
                format!("invalid pipeline config TOML line: {line}"),
                None,
            ));
        };
        let field = field_name(&section, key.trim());
        if !is_known_field(&field) {
            return Err(referenced_config_error(
                format!("unknown field '{field}'"),
                Some(field),
            ));
        }
        if values
            .insert(field.clone(), unquote_value(value.trim()).to_string())
            .is_some()
        {
            return Err(referenced_config_error(
                format!("duplicate field '{field}'"),
                Some(field),
            ));
        }
    }

    pipeline_config_from_values(values)
}

fn pipeline_config_from_values(
    values: BTreeMap<String, String>,
) -> Result<PipelineConfig, ReferencedConfigError> {
    require_schema_version(&values)?;
    Ok(PipelineConfig {
        schema_version: PipelineConfigVersion::BiorsPipelineV0,
        name: required_value(&values, "name")?.to_string(),
        input: PipelineInputConfig {
            format: required_value(&values, "input.format")?.to_string(),
            path: required_value(&values, "input.path")?.to_string(),
        },
        normalize: PipelineNormalizeConfig {
            policy: required_value(&values, "normalize.policy")?.to_string(),
        },
        validate: PipelineValidateConfig {
            kind: required_value(&values, "validate.kind")?.to_string(),
        },
        tokenize: PipelineTokenizeConfig {
            profile: required_value(&values, "tokenize.profile")?.to_string(),
        },
        export: PipelineExportConfig {
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

fn is_known_field(field: &str) -> bool {
    matches!(
        field,
        "schema_version"
            | "name"
            | "input.format"
            | "input.path"
            | "normalize.policy"
            | "validate.kind"
            | "tokenize.profile"
            | "export.format"
            | "export.max_length"
            | "export.pad_token_id"
            | "export.padding"
    )
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

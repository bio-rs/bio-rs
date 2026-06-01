use crate::cli::{PaddingArg, TokenizerProfileArg};
use crate::errors::CliError;
use biors_core::package::PipelineConfigVersion;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

const FASTA_FORMAT: &str = "fasta";
const NORMALIZE_POLICY: &str = "strip_ascii_whitespace_uppercase";
const EXPORT_FORMAT: &str = "model-input-json";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct PipelineConfig {
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
pub(crate) struct PipelineInputConfig {
    pub format: String,
    pub path: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct PipelineNormalizeConfig {
    pub policy: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct PipelineValidateConfig {
    pub kind: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct PipelineTokenizeConfig {
    pub profile: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct PipelineExportConfig {
    pub format: String,
    pub max_length: usize,
    #[serde(default)]
    pub pad_token_id: u8,
    #[serde(default = "default_padding")]
    pub padding: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ResolvedPipelineConfig {
    pub config: PipelineConfig,
    pub input_path: PathBuf,
    pub padding: PaddingArg,
    pub profile: TokenizerProfileArg,
}

pub(crate) fn load_pipeline_config(path: &Path) -> Result<ResolvedPipelineConfig, CliError> {
    let contents = std::fs::read_to_string(path).map_err(|source| CliError::Read {
        path: path.to_path_buf(),
        source,
    })?;
    let config = parse_pipeline_config(path, &contents)?;
    let profile = validate_pipeline_config(&config)?;
    Ok(ResolvedPipelineConfig {
        input_path: resolve_input_path(path, &config.input.path),
        padding: parse_padding(&config.export.padding)?,
        profile,
        config,
    })
}

fn parse_pipeline_config(path: &Path, contents: &str) -> Result<PipelineConfig, CliError> {
    match extension(path).as_deref() {
        Some("json") => serde_json::from_str(contents).map_err(invalid_config),
        Some("toml") => toml::from_str(contents).map_err(invalid_config),
        Some(extension) => Err(CliError::Validation {
            code: "pipeline.invalid_config",
            message: format!("unsupported pipeline config extension: {extension}"),
            location: Some(path.display().to_string()),
        }),
        None => Err(CliError::Validation {
            code: "pipeline.invalid_config",
            message: "pipeline config path must have .json or .toml extension".to_string(),
            location: Some(path.display().to_string()),
        }),
    }
}

fn validate_pipeline_config(config: &PipelineConfig) -> Result<TokenizerProfileArg, CliError> {
    require_non_empty("name", &config.name)?;
    require_value("input.format", &config.input.format, FASTA_FORMAT)?;
    require_non_empty("input.path", &config.input.path)?;
    require_value(
        "normalize.policy",
        &config.normalize.policy,
        NORMALIZE_POLICY,
    )?;
    let profile = parse_profile(&config.tokenize.profile)?;
    validate_kind_matches_profile(&config.validate.kind, profile)?;
    require_value("export.format", &config.export.format, EXPORT_FORMAT)?;
    if config.export.max_length == 0 {
        return Err(CliError::Validation {
            code: "pipeline.invalid_config",
            message: "export.max_length must be greater than zero".to_string(),
            location: Some("export.max_length".to_string()),
        });
    }
    parse_padding(&config.export.padding)?;
    Ok(profile)
}

fn require_non_empty(field: &str, value: &str) -> Result<(), CliError> {
    if value.trim().is_empty() {
        return Err(CliError::Validation {
            code: "pipeline.invalid_config",
            message: format!("{field} is required"),
            location: Some(field.to_string()),
        });
    }
    Ok(())
}

fn require_value(field: &str, actual: &str, expected: &str) -> Result<(), CliError> {
    if actual != expected {
        return Err(CliError::Validation {
            code: "pipeline.invalid_config",
            message: format!("{field} must be '{expected}'"),
            location: Some(field.to_string()),
        });
    }
    Ok(())
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

fn parse_profile(value: &str) -> Result<TokenizerProfileArg, CliError> {
    match value {
        "protein-20" => Ok(TokenizerProfileArg::Protein20),
        "protein-20-special" => Ok(TokenizerProfileArg::Protein20Special),
        "dna-iupac" => Ok(TokenizerProfileArg::DnaIupac),
        "dna-iupac-special" => Ok(TokenizerProfileArg::DnaIupacSpecial),
        "rna-iupac" => Ok(TokenizerProfileArg::RnaIupac),
        "rna-iupac-special" => Ok(TokenizerProfileArg::RnaIupacSpecial),
        _ => Err(CliError::Validation {
            code: "pipeline.invalid_config",
            message: "tokenize.profile must be one of protein-20, protein-20-special, dna-iupac, dna-iupac-special, rna-iupac, rna-iupac-special".to_string(),
            location: Some("tokenize.profile".to_string()),
        }),
    }
}

fn validate_kind_matches_profile(kind: &str, profile: TokenizerProfileArg) -> Result<(), CliError> {
    let expected = match profile {
        TokenizerProfileArg::Protein20 | TokenizerProfileArg::Protein20Special => "protein",
        TokenizerProfileArg::DnaIupac | TokenizerProfileArg::DnaIupacSpecial => "dna",
        TokenizerProfileArg::RnaIupac | TokenizerProfileArg::RnaIupacSpecial => "rna",
    };
    if kind == expected {
        return Ok(());
    }
    if !matches!(kind, "protein" | "dna" | "rna") {
        return Err(CliError::Validation {
            code: "pipeline.invalid_config",
            message: "validate.kind must be one of protein, dna, rna".to_string(),
            location: Some("validate.kind".to_string()),
        });
    }
    Err(CliError::Validation {
        code: "pipeline.invalid_config",
        message: format!(
            "validate.kind must be '{expected}' for profile '{}'",
            profile_name(profile)
        ),
        location: Some("validate.kind".to_string()),
    })
}

fn profile_name(profile: TokenizerProfileArg) -> &'static str {
    match profile {
        TokenizerProfileArg::Protein20 => "protein-20",
        TokenizerProfileArg::Protein20Special => "protein-20-special",
        TokenizerProfileArg::DnaIupac => "dna-iupac",
        TokenizerProfileArg::DnaIupacSpecial => "dna-iupac-special",
        TokenizerProfileArg::RnaIupac => "rna-iupac",
        TokenizerProfileArg::RnaIupacSpecial => "rna-iupac-special",
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

fn extension(path: &Path) -> Option<String> {
    path.extension()
        .and_then(|value| value.to_str())
        .map(str::to_ascii_lowercase)
}

fn invalid_config(error: impl std::fmt::Display) -> CliError {
    CliError::Validation {
        code: "pipeline.invalid_config",
        message: error.to_string(),
        location: Some("config".to_string()),
    }
}

fn default_padding() -> String {
    "fixed_length".to_string()
}

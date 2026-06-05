use crate::errors::CliError;
use crate::output::print_success;
use biors_core::{
    hash::sha256_bytes_digest,
    tokenizer::{
        protein_tokenizer_config_for_profile, ProteinTokenizerConfig, ProteinTokenizerProfile,
    },
};
use serde::Serialize;
use serde_json::Value;
use std::path::{Component, Path, PathBuf};

#[derive(Debug, Serialize)]
struct TokenizerConversionOutput {
    source_format: &'static str,
    source_path: String,
    config: ProteinTokenizerConfig,
    conversion_status: TokenizerConversionStatus,
    preview_tokenizer_asset: PackageTokenizerAsset,
    preview_preprocessing_step: PackagePreprocessingStep,
    assumptions: Vec<String>,
    warnings: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    output_path: Option<String>,
    config_sha256: String,
}

#[derive(Debug, Serialize)]
struct TokenizerConversionStatus {
    status: &'static str,
    package_ready: bool,
    fixture_parity_required: bool,
    message: &'static str,
}

#[derive(Debug, Serialize)]
struct PackageTokenizerAsset {
    name: String,
    path: String,
    checksum: String,
    contract_version: String,
}

#[derive(Debug, Serialize)]
struct PackagePreprocessingStep {
    name: String,
    implementation: String,
    contract: String,
    contract_version: String,
}

pub(crate) fn run_tokenizer_convert_hf(
    path: PathBuf,
    output: Option<PathBuf>,
) -> Result<(), CliError> {
    let input = std::fs::read_to_string(&path).map_err(|source| CliError::Read {
        path: path.clone(),
        source,
    })?;
    let value: Value = serde_json::from_str(&input).map_err(CliError::Json)?;
    let converted = convert_huggingface_tokenizer_config(&value, &path, output.as_ref())?;

    if let Some(output_path) = &output {
        let config_json = serialized_tokenizer_config(&converted.config)?;
        if let Some(parent) = output_path
            .parent()
            .filter(|path| !path.as_os_str().is_empty())
        {
            std::fs::create_dir_all(parent).map_err(CliError::Write)?;
        }
        std::fs::write(output_path, config_json).map_err(CliError::Write)?;
    }

    print_success(None, converted)
}

fn convert_huggingface_tokenizer_config(
    value: &Value,
    source_path: &std::path::Path,
    output_path: Option<&PathBuf>,
) -> Result<TokenizerConversionOutput, CliError> {
    let (config, assumptions, mut warnings) = hf_config_to_biors_config(value, source_path)?;
    warnings.push(
        "preview conversion only: bio-rs does not read the Hugging Face vocab, token IDs, normalizer, or pre-tokenizer; validate fixture parity before using these fragments in a package"
            .to_string(),
    );
    let config_bytes = serialized_tokenizer_config(&config)?.into_bytes();
    let config_sha256 = sha256_bytes_digest(&config_bytes);
    let config_name = config.profile.as_str().to_string();
    let tokenizer_path = preview_tokenizer_path(output_path, &config_name, &mut warnings);

    Ok(TokenizerConversionOutput {
        source_format: "huggingface.tokenizer_config",
        source_path: source_path.display().to_string(),
        conversion_status: TokenizerConversionStatus {
            status: "preview",
            package_ready: false,
            fixture_parity_required: true,
            message:
                "heuristic tokenizer_config mapping; not package-ready until fixture parity is validated",
        },
        preview_tokenizer_asset: PackageTokenizerAsset {
            name: config_name.clone(),
            path: tokenizer_path,
            checksum: config_sha256.clone(),
            contract_version: format!("{config_name}.v0"),
        },
        preview_preprocessing_step: PackagePreprocessingStep {
            name: "protein_fasta_tokenize".to_string(),
            implementation: "biors-core".to_string(),
            contract: config_name.clone(),
            contract_version: format!("{config_name}.v0"),
        },
        config,
        assumptions,
        warnings,
        output_path: output_path.map(|path| path.display().to_string()),
        config_sha256,
    })
}

fn serialized_tokenizer_config(config: &ProteinTokenizerConfig) -> Result<String, CliError> {
    let config_json = serde_json::to_string_pretty(config).map_err(CliError::Serialization)?;
    Ok(format!("{config_json}\n"))
}

fn preview_tokenizer_path(
    output_path: Option<&PathBuf>,
    config_name: &str,
    warnings: &mut Vec<String>,
) -> String {
    let default_path = format!("tokenizers/{config_name}.json");
    let Some(output_path) = output_path else {
        return default_path;
    };

    if let Some(path) = tokenizer_dir_relative_path(output_path) {
        return path;
    }

    warnings.push(format!(
        "output path '{}' is outside a package tokenizers/ directory; copy the written config to {default_path} before using preview_tokenizer_asset",
        output_path.display()
    ));
    default_path
}

fn tokenizer_dir_relative_path(path: &Path) -> Option<String> {
    let components: Vec<_> = path
        .components()
        .filter_map(|component| match component {
            Component::Normal(value) => Some(value.to_string_lossy().into_owned()),
            _ => None,
        })
        .collect();
    let start = components
        .iter()
        .position(|component| component == "tokenizers")?;
    let relative = &components[start..];
    if relative.len() < 2 {
        return None;
    }
    Some(relative.join("/"))
}

pub(crate) fn hf_config_to_biors_config(
    value: &Value,
    source_path: &std::path::Path,
) -> Result<(ProteinTokenizerConfig, Vec<String>, Vec<String>), CliError> {
    let object = value.as_object().ok_or_else(|| CliError::Validation {
        code: "tokenizer.conversion_invalid_config",
        message: "Hugging Face tokenizer config must be a JSON object".to_string(),
        location: Some(source_path.display().to_string()),
    })?;

    let mut assumptions = Vec::new();
    let mut warnings = Vec::new();
    let has_special_tokens = [
        "unk_token",
        "pad_token",
        "cls_token",
        "sep_token",
        "mask_token",
    ]
    .iter()
    .any(|key| object.get(*key).is_some())
        || object
            .get("special_tokens_map")
            .and_then(Value::as_object)
            .is_some_and(|tokens| !tokens.is_empty());

    if let Some(tokenizer_class) = object.get("tokenizer_class").and_then(Value::as_str) {
        assumptions.push(format!(
            "source tokenizer_class '{tokenizer_class}' is mapped to the built-in protein-20 residue vocabulary"
        ));
    } else {
        warnings
            .push("tokenizer_class is absent; conversion assumes a protein tokenizer".to_string());
    }

    if object
        .get("do_lower_case")
        .and_then(Value::as_bool)
        .unwrap_or(false)
    {
        warnings.push(
            "do_lower_case=true is ignored because bio-rs normalizes biological sequences by uppercasing residues".to_string(),
        );
    }

    if let Some(max_length) = object.get("model_max_length").and_then(Value::as_i64) {
        assumptions.push(format!(
            "model_max_length {max_length} is not stored in the tokenizer config; set export.max_length in a pipeline config"
        ));
    }

    let profile = if has_special_tokens {
        ProteinTokenizerProfile::Protein20Special
    } else {
        ProteinTokenizerProfile::Protein20
    };
    let config = protein_tokenizer_config_for_profile(profile);
    Ok((config, assumptions, warnings))
}

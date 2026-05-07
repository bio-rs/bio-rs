use crate::cli::package_skeleton::PackageSkeletonRequest;
use crate::cli::tokenizer_convert::hf_config_to_biors_config;
use crate::errors::CliError;
use biors_core::{
    package::{
        sha256_digest, CitationMetadata, DocumentArtifact, LicenseMetadata, ModelCardMetadata,
        PackageMetadata, TokenAsset,
    },
    tokenizer::{ProteinTokenizerConfig, ProteinTokenizerProfile},
};
use serde_json::Value;
use std::path::{Path, PathBuf};

pub(crate) fn write_tokenizer_config(
    request: &PackageSkeletonRequest,
    created_files: &mut Vec<String>,
) -> Result<(TokenAsset, ProteinTokenizerProfile, Vec<String>), CliError> {
    let (config, notes) = match &request.tokenizer_config {
        Some(path) => read_tokenizer_config(path)?,
        None => (
            biors_core::tokenizer::protein_tokenizer_config_for_profile(
                ProteinTokenizerProfile::Protein20,
            ),
            vec!["no tokenizer config supplied; using built-in protein-20".to_string()],
        ),
    };
    let rel = format!("tokenizers/{}.json", config.profile.as_str());
    let path = request.output_dir.join(&rel);
    let config_json = serde_json::to_string_pretty(&config).map_err(CliError::Serialization)?;
    std::fs::write(&path, format!("{config_json}\n")).map_err(CliError::Write)?;
    created_files.push(path.display().to_string());

    Ok((
        TokenAsset {
            name: config.profile.as_str().to_string(),
            path: rel,
            checksum: Some(file_sha256(&path)?),
            contract_version: Some(format!("{}.v0", config.profile.as_str())),
        },
        config.profile,
        notes,
    ))
}

pub(crate) fn write_pipeline_config(
    output_dir: &Path,
    fixture_input_rel: &str,
    profile: ProteinTokenizerProfile,
    created_files: &mut Vec<String>,
) -> Result<String, CliError> {
    let rel = "pipelines/protein.toml".to_string();
    let path = output_dir.join(&rel);
    let input_name = Path::new(fixture_input_rel)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("input.fasta");
    let contents = format!(
        r#"schema_version = "biors.pipeline.v0"
name = "package-preprocessing"

[input]
format = "fasta"
path = "../fixtures/{input_name}"

[normalize]
policy = "strip_ascii_whitespace_uppercase"

[validate]
kind = "protein"

[tokenize]
profile = "{}"

[export]
format = "model-input-json"
max_length = 512
pad_token_id = 0
padding = "fixed_length"
"#,
        profile.as_str()
    );
    std::fs::write(&path, contents).map_err(CliError::Write)?;
    created_files.push(path.display().to_string());
    Ok(rel)
}

pub(crate) fn write_docs(
    request: &PackageSkeletonRequest,
    created_files: &mut Vec<String>,
) -> Result<PackageMetadata, CliError> {
    let license_rel = "docs/LICENSE.txt";
    let citation_rel = "docs/CITATION.cff";
    let model_card_rel = "docs/model-card.md";
    let license_path = request.output_dir.join(license_rel);
    let citation_path = request.output_dir.join(citation_rel);
    let model_card_path = request.output_dir.join(model_card_rel);

    std::fs::write(&license_path, format!("{}\n", request.license)).map_err(CliError::Write)?;
    std::fs::write(&citation_path, format!("{}\n", request.citation)).map_err(CliError::Write)?;
    std::fs::write(
        &model_card_path,
        format!(
            "# {}\n\n{}\n\n## Intended Use\n\n{}\n\n## Limitations\n\n{}\n",
            request.name,
            request.model_card_summary,
            markdown_list(&request.intended_use),
            markdown_list(&request.limitations)
        ),
    )
    .map_err(CliError::Write)?;

    created_files.push(license_path.display().to_string());
    created_files.push(citation_path.display().to_string());
    created_files.push(model_card_path.display().to_string());

    Ok(PackageMetadata {
        license: LicenseMetadata {
            expression: request.license.clone(),
            file: Some(DocumentArtifact {
                path: license_rel.to_string(),
                checksum: Some(file_sha256(&license_path)?),
            }),
        },
        citation: CitationMetadata {
            preferred_citation: request.citation.clone(),
            doi: request.doi.clone(),
            file: Some(DocumentArtifact {
                path: citation_rel.to_string(),
                checksum: Some(file_sha256(&citation_path)?),
            }),
        },
        model_card: ModelCardMetadata {
            path: model_card_rel.to_string(),
            checksum: Some(file_sha256(&model_card_path)?),
            summary: request.model_card_summary.clone(),
            intended_use: request.intended_use.clone(),
            limitations: request.limitations.clone(),
        },
    })
}

pub(crate) fn copy_asset(
    source: &Path,
    output_dir: &Path,
    target_dir: &str,
    created_files: &mut Vec<String>,
) -> Result<String, CliError> {
    let file_name = source
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| CliError::Validation {
            code: "package.init_invalid_path",
            message: "asset path must have a UTF-8 file name".to_string(),
            location: Some(source.display().to_string()),
        })?;
    let rel = format!("{target_dir}/{file_name}");
    let target = output_dir.join(&rel);
    std::fs::copy(source, &target).map_err(CliError::Write)?;
    created_files.push(target.display().to_string());
    Ok(rel)
}

pub(crate) fn file_sha256(path: &Path) -> Result<String, CliError> {
    let bytes = std::fs::read(path).map_err(|source| CliError::Read {
        path: path.to_path_buf(),
        source,
    })?;
    Ok(sha256_digest(&bytes))
}

pub(crate) fn validate_required_list(option: &str, values: &[String]) -> Result<(), CliError> {
    if values.iter().any(|value| value.trim().is_empty()) || values.is_empty() {
        return Err(CliError::Validation {
            code: "package.init_missing_metadata",
            message: format!("{option} must be supplied at least once"),
            location: Some(option.to_string()),
        });
    }
    Ok(())
}

pub(crate) fn fixture_name(path: &Path) -> String {
    path.file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or("fixture")
        .to_string()
}

fn read_tokenizer_config(
    path: &PathBuf,
) -> Result<(ProteinTokenizerConfig, Vec<String>), CliError> {
    let input = std::fs::read_to_string(path).map_err(|source| CliError::Read {
        path: path.clone(),
        source,
    })?;
    match serde_json::from_str::<ProteinTokenizerConfig>(&input) {
        Ok(config) => Ok((config, Vec::new())),
        Err(_) => {
            let value: Value = serde_json::from_str(&input).map_err(CliError::Json)?;
            let (config, assumptions, warnings) = hf_config_to_biors_config(&value, path)?;
            let mut notes = assumptions;
            notes.extend(warnings);
            Ok((config, notes))
        }
    }
}

fn markdown_list(values: &[String]) -> String {
    values
        .iter()
        .map(|value| format!("- {value}"))
        .collect::<Vec<_>>()
        .join("\n")
}

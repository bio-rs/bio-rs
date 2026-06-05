use super::{referenced_config_error, PipelineConfig};
use crate::{
    package::{PipelineConfigVersion, ReferencedConfigError},
    sequence::SequenceKind,
    tokenizer::ProteinTokenizerProfile,
};

const FASTA_FORMAT: &str = "fasta";
const NORMALIZE_POLICY: &str = "strip_ascii_whitespace_uppercase";
const EXPORT_FORMAT: &str = "model-input-json";

pub(super) fn validate_pipeline_config_fields(
    config: &PipelineConfig,
) -> Result<ProteinTokenizerProfile, ReferencedConfigError> {
    require_pipeline_version(config.schema_version)?;
    require_non_empty("name", &config.name)?;
    require_value("input.format", &config.input.format, FASTA_FORMAT)?;
    require_non_empty("input.path", &config.input.path)?;
    require_value(
        "normalize.policy",
        &config.normalize.policy,
        NORMALIZE_POLICY,
    )?;
    let kind = parse_sequence_kind(&config.validate.kind)?;
    let profile = parse_tokenizer_profile(&config.tokenize.profile)?;
    if kind != profile.sequence_kind() {
        return Err(referenced_config_error(
            format!(
                "validate.kind must be '{}' for profile '{}'",
                profile.sequence_kind(),
                profile.as_str()
            ),
            Some("validate.kind".to_string()),
        ));
    }
    require_value("export.format", &config.export.format, EXPORT_FORMAT)?;
    if config.export.max_length == 0 {
        return Err(referenced_config_error(
            "export.max_length must be greater than zero",
            Some("export.max_length".to_string()),
        ));
    }
    validate_padding(&config.export.padding)?;
    Ok(profile)
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

fn require_non_empty(field: &str, value: &str) -> Result<(), ReferencedConfigError> {
    if value.trim().is_empty() {
        return Err(referenced_config_error(
            format!("{field} is required"),
            Some(field.to_string()),
        ));
    }
    Ok(())
}

fn require_value(field: &str, actual: &str, expected: &str) -> Result<(), ReferencedConfigError> {
    if actual != expected {
        return Err(referenced_config_error(
            format!("{field} must be '{expected}'"),
            Some(field.to_string()),
        ));
    }
    Ok(())
}

fn parse_sequence_kind(value: &str) -> Result<SequenceKind, ReferencedConfigError> {
    match value {
        "protein" => Ok(SequenceKind::Protein),
        "dna" => Ok(SequenceKind::Dna),
        "rna" => Ok(SequenceKind::Rna),
        _ => Err(referenced_config_error(
            "validate.kind must be one of protein, dna, rna",
            Some("validate.kind".to_string()),
        )),
    }
}

fn parse_tokenizer_profile(value: &str) -> Result<ProteinTokenizerProfile, ReferencedConfigError> {
    match value {
        "protein-20" => Ok(ProteinTokenizerProfile::Protein20),
        "protein-20-special" => Ok(ProteinTokenizerProfile::Protein20Special),
        "dna-iupac" => Ok(ProteinTokenizerProfile::DnaIupac),
        "dna-iupac-special" => Ok(ProteinTokenizerProfile::DnaIupacSpecial),
        "rna-iupac" => Ok(ProteinTokenizerProfile::RnaIupac),
        "rna-iupac-special" => Ok(ProteinTokenizerProfile::RnaIupacSpecial),
        _ => Err(referenced_config_error(
            "tokenize.profile must be one of protein-20, protein-20-special, dna-iupac, dna-iupac-special, rna-iupac, rna-iupac-special",
            Some("tokenize.profile".to_string()),
        )),
    }
}

fn validate_padding(value: &str) -> Result<(), ReferencedConfigError> {
    if matches!(value, "fixed_length" | "no_padding") {
        return Ok(());
    }
    Err(referenced_config_error(
        "export.padding must be 'fixed_length' or 'no_padding'",
        Some("export.padding".to_string()),
    ))
}

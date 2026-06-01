use super::{PackageValidationIssueCode, PackageValidationReport, TokenAsset};
use crate::tokenizer::ProteinTokenizerConfig;
use std::path::Path;

pub type ReferencedConfigValidator<'a> = dyn Fn(&Path) -> Result<(), ReferencedConfigError> + 'a;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReferencedConfigError {
    pub code: String,
    pub message: String,
    pub location: Option<String>,
}

impl ReferencedConfigError {
    pub fn new(
        code: impl Into<String>,
        message: impl Into<String>,
        location: Option<String>,
    ) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            location,
        }
    }
}

pub(crate) fn validate_referenced_pipeline_config(
    report: &mut PackageValidationReport,
    field: &str,
    path: &str,
    base_dir: &Path,
    pipeline_config_validator: Option<&ReferencedConfigValidator<'_>>,
) {
    let Some(validator) = pipeline_config_validator else {
        return;
    };
    let config_path = base_dir.join(path);
    if let Err(error) = validator(&config_path) {
        let mut message = format!(
            "{field}: pipeline config '{path}' is invalid: {}: {}",
            error.code, error.message
        );
        if let Some(location) = error.location {
            message.push_str(&format!(" at {location}"));
        }
        report.push_issue(
            PackageValidationIssueCode::InvalidPipelineConfig,
            field,
            &message,
        );
    }
}

pub(crate) fn validate_tokenizer_config(
    report: &mut PackageValidationReport,
    tokenizer: &TokenAsset,
    bytes: &[u8],
) {
    let config = match serde_json::from_slice::<ProteinTokenizerConfig>(bytes) {
        Ok(config) => config,
        Err(error) => {
            report.push_issue(
                PackageValidationIssueCode::InvalidTokenizerConfig,
                "tokenizer",
                &format!("tokenizer: invalid tokenizer config JSON: {error}"),
            );
            return;
        }
    };

    let expected_name = config.profile.as_str();
    if !tokenizer.name.trim().is_empty() && tokenizer.name != expected_name {
        report.push_issue(
            PackageValidationIssueCode::InvalidTokenizerConfig,
            "tokenizer.name",
            &format!("tokenizer.name must match tokenizer config profile '{expected_name}'"),
        );
    }

    let expected_contract_version = format!("{expected_name}.v0");
    if tokenizer
        .contract_version
        .as_deref()
        .is_some_and(|contract_version| {
            !contract_version.trim().is_empty()
                && contract_version != expected_contract_version.as_str()
        })
    {
        report.push_issue(
            PackageValidationIssueCode::InvalidTokenizerConfig,
            "tokenizer.contract_version",
            &format!(
                "tokenizer.contract_version must match tokenizer config profile version '{expected_contract_version}'"
            ),
        );
    }

    if config.add_special_tokens != config.profile.default_add_special_tokens() {
        report.push_issue(
            PackageValidationIssueCode::InvalidTokenizerConfig,
            "tokenizer.add_special_tokens",
            &format!(
                "tokenizer.add_special_tokens must be {} for profile '{}'",
                config.profile.default_add_special_tokens(),
                expected_name
            ),
        );
    }
}

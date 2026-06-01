use super::{PackageValidationIssueCode, PackageValidationReport, TokenAsset};
use crate::tokenizer::{
    dna_iupac_vocab_tokens, load_vocab_json, protein_20_vocab_tokens, rna_iupac_vocab_tokens,
    ProteinTokenizerConfig, UnknownTokenPolicy, VocabToken, Vocabulary,
    NUCLEOTIDE_UNKNOWN_TOKEN_ID, PROTEIN_20_UNKNOWN_TOKEN_ID,
};
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

pub(crate) fn validate_vocab_config(
    report: &mut PackageValidationReport,
    vocab_asset: &TokenAsset,
    bytes: &[u8],
) {
    let input = match std::str::from_utf8(bytes) {
        Ok(input) => input,
        Err(error) => {
            report.push_issue(
                PackageValidationIssueCode::InvalidVocabConfig,
                "vocab",
                &format!("vocab: invalid UTF-8 vocabulary JSON: {error}"),
            );
            return;
        }
    };
    let vocab = match load_vocab_json(input) {
        Ok(vocab) => vocab,
        Err(error) => {
            report.push_issue(
                PackageValidationIssueCode::InvalidVocabConfig,
                "vocab",
                &format!("vocab: {error}"),
            );
            return;
        }
    };

    if !vocab_asset.name.trim().is_empty() && vocab_asset.name != vocab.name {
        report.push_issue(
            PackageValidationIssueCode::InvalidVocabConfig,
            "vocab.name",
            &format!("vocab.name must match vocabulary name '{}'", vocab.name),
        );
    }

    let expected_contract_version = format!("{}.v0", vocab.name);
    if vocab_asset
        .contract_version
        .as_deref()
        .is_some_and(|contract_version| {
            !contract_version.trim().is_empty()
                && contract_version != expected_contract_version.as_str()
        })
    {
        report.push_issue(
            PackageValidationIssueCode::InvalidVocabConfig,
            "vocab.contract_version",
            &format!(
                "vocab.contract_version must match vocabulary version '{expected_contract_version}'"
            ),
        );
    }

    match vocab.name.as_str() {
        "protein-20" => validate_builtin_vocab(
            report,
            &vocab,
            PROTEIN_20_UNKNOWN_TOKEN_ID,
            protein_20_vocab_tokens().as_slice(),
        ),
        "dna-iupac" => validate_builtin_vocab(
            report,
            &vocab,
            NUCLEOTIDE_UNKNOWN_TOKEN_ID,
            dna_iupac_vocab_tokens().as_slice(),
        ),
        "rna-iupac" => validate_builtin_vocab(
            report,
            &vocab,
            NUCLEOTIDE_UNKNOWN_TOKEN_ID,
            rna_iupac_vocab_tokens().as_slice(),
        ),
        _ => {}
    }
}

fn validate_builtin_vocab(
    report: &mut PackageValidationReport,
    vocab: &Vocabulary,
    expected_unknown_token_id: u8,
    expected_tokens: &[VocabToken],
) {
    if vocab.unknown_token_id != expected_unknown_token_id {
        report.push_issue(
            PackageValidationIssueCode::InvalidVocabConfig,
            "vocab.unknown_token_id",
            &format!(
                "vocab.unknown_token_id must be {} for {}",
                expected_unknown_token_id, vocab.name
            ),
        );
    }
    if vocab.unknown_token_policy != UnknownTokenPolicy::WarnOrErrorWithUnknownToken {
        report.push_issue(
            PackageValidationIssueCode::InvalidVocabConfig,
            "vocab.unknown_token_policy",
            &format!(
                "vocab.unknown_token_policy must be warn_or_error_with_unknown_token for {}",
                vocab.name
            ),
        );
    }
    if vocab.tokens.as_slice() != expected_tokens {
        report.push_issue(
            PackageValidationIssueCode::InvalidVocabConfig,
            "vocab.tokens",
            &format!(
                "vocab.tokens must match the built-in {} token order and IDs",
                vocab.name
            ),
        );
    }
}

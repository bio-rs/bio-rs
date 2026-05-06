use super::{protein_20_vocabulary, UnknownTokenPolicy, Vocabulary};
use serde::{Deserialize, Serialize};

/// Built-in protein tokenizer profiles.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ProteinTokenizerProfile {
    /// Standard protein-20 residue tokens with unknown token `20`.
    #[serde(rename = "protein-20")]
    Protein20,
    /// Protein-20 residue tokens plus explicit PAD/CLS/SEP/MASK token policy.
    #[serde(rename = "protein-20-special")]
    Protein20Special,
}

impl ProteinTokenizerProfile {
    /// Stable profile name used in JSON and CLI output.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Protein20 => "protein-20",
            Self::Protein20Special => "protein-20-special",
        }
    }

    /// Whether profile defaults to emitting sequence boundary tokens.
    pub const fn default_add_special_tokens(self) -> bool {
        match self {
            Self::Protein20 => false,
            Self::Protein20Special => true,
        }
    }
}

/// JSON tokenizer configuration for protein preprocessing.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProteinTokenizerConfig {
    /// Tokenizer profile.
    pub profile: ProteinTokenizerProfile,
    /// Whether tokenization should emit profile boundary special tokens.
    pub add_special_tokens: bool,
}

impl Default for ProteinTokenizerConfig {
    fn default() -> Self {
        protein_tokenizer_config_for_profile(ProteinTokenizerProfile::Protein20)
    }
}

/// One named special token.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpecialToken {
    /// Stable token string.
    pub token: String,
    /// Token ID.
    pub token_id: u8,
}

/// Special token policy for profile-aware tokenization.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpecialTokenSet {
    /// Unknown residue token.
    pub unk: SpecialToken,
    /// Padding token.
    pub pad: SpecialToken,
    /// Sequence start token.
    pub cls: SpecialToken,
    /// Sequence end token.
    pub sep: SpecialToken,
    /// Mask token for downstream model examples.
    pub mask: SpecialToken,
}

/// Machine-readable profile inspection output.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProteinTokenizerInspection {
    /// Tokenizer profile.
    pub profile: ProteinTokenizerProfile,
    /// Config used for this inspection.
    pub config: ProteinTokenizerConfig,
    /// Residue vocabulary.
    pub vocabulary: Vocabulary,
    /// Unknown residue policy.
    pub unknown_token_policy: UnknownTokenPolicy,
    /// Unknown token ID.
    pub unknown_token_id: u8,
    /// Special token policy.
    pub special_tokens: SpecialTokenSet,
}

/// Return the default config for a built-in tokenizer profile.
pub fn protein_tokenizer_config_for_profile(
    profile: ProteinTokenizerProfile,
) -> ProteinTokenizerConfig {
    ProteinTokenizerConfig {
        profile,
        add_special_tokens: profile.default_add_special_tokens(),
    }
}

/// Load a protein tokenizer config from JSON.
pub fn load_protein_tokenizer_config_json(
    input: &str,
) -> Result<ProteinTokenizerConfig, serde_json::Error> {
    serde_json::from_str(input)
}

/// Build machine-readable tokenizer inspection output for a config.
pub fn inspect_protein_tokenizer_config(
    config: &ProteinTokenizerConfig,
) -> ProteinTokenizerInspection {
    let vocabulary = protein_20_vocabulary().clone();
    ProteinTokenizerInspection {
        profile: config.profile,
        config: config.clone(),
        unknown_token_policy: vocabulary.unknown_token_policy.clone(),
        unknown_token_id: vocabulary.unknown_token_id,
        vocabulary,
        special_tokens: protein_special_tokens(),
    }
}

pub(crate) fn profile_vocabulary_name(profile: ProteinTokenizerProfile) -> String {
    profile.as_str().to_string()
}

pub(crate) fn protein_special_tokens() -> SpecialTokenSet {
    SpecialTokenSet {
        unk: special("UNK", 20),
        pad: special("PAD", 21),
        cls: special("CLS", 22),
        sep: special("SEP", 23),
        mask: special("MASK", 24),
    }
}

fn special(name: &str, token_id: u8) -> SpecialToken {
    SpecialToken {
        token: format!("<{name}>"),
        token_id,
    }
}

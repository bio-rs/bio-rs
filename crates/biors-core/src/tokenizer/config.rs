use super::{
    dna_iupac_vocabulary, protein_20_vocabulary, rna_iupac_vocabulary, UnknownTokenPolicy,
    Vocabulary,
};
use crate::sequence::SequenceKind;
use serde::{Deserialize, Serialize};

/// Built-in sequence tokenizer profiles.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ProteinTokenizerProfile {
    /// Standard protein-20 residue tokens with unknown token `20`.
    #[serde(rename = "protein-20")]
    Protein20,
    /// Protein-20 residue tokens plus explicit PAD/CLS/SEP/MASK token policy.
    #[serde(rename = "protein-20-special")]
    Protein20Special,
    /// DNA IUPAC canonical base tokens with ambiguous bases emitted as unknown.
    #[serde(rename = "dna-iupac")]
    DnaIupac,
    /// DNA IUPAC canonical base tokens plus explicit PAD/CLS/SEP/MASK token policy.
    #[serde(rename = "dna-iupac-special")]
    DnaIupacSpecial,
    /// RNA IUPAC canonical base tokens with ambiguous bases emitted as unknown.
    #[serde(rename = "rna-iupac")]
    RnaIupac,
    /// RNA IUPAC canonical base tokens plus explicit PAD/CLS/SEP/MASK token policy.
    #[serde(rename = "rna-iupac-special")]
    RnaIupacSpecial,
}

impl ProteinTokenizerProfile {
    /// Stable profile name used in JSON and CLI output.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Protein20 => "protein-20",
            Self::Protein20Special => "protein-20-special",
            Self::DnaIupac => "dna-iupac",
            Self::DnaIupacSpecial => "dna-iupac-special",
            Self::RnaIupac => "rna-iupac",
            Self::RnaIupacSpecial => "rna-iupac-special",
        }
    }

    /// Whether profile defaults to emitting sequence boundary tokens.
    pub const fn default_add_special_tokens(self) -> bool {
        match self {
            Self::Protein20 | Self::DnaIupac | Self::RnaIupac => false,
            Self::Protein20Special | Self::DnaIupacSpecial | Self::RnaIupacSpecial => true,
        }
    }

    /// Biological sequence kind this tokenizer profile accepts.
    pub const fn sequence_kind(self) -> SequenceKind {
        match self {
            Self::Protein20 | Self::Protein20Special => SequenceKind::Protein,
            Self::DnaIupac | Self::DnaIupacSpecial => SequenceKind::Dna,
            Self::RnaIupac | Self::RnaIupacSpecial => SequenceKind::Rna,
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
    let vocabulary = profile_vocabulary(config.profile).clone();
    ProteinTokenizerInspection {
        profile: config.profile,
        config: config.clone(),
        unknown_token_policy: vocabulary.unknown_token_policy.clone(),
        unknown_token_id: vocabulary.unknown_token_id,
        vocabulary,
        special_tokens: special_tokens_for_profile(config.profile),
    }
}

pub(crate) fn profile_vocabulary_name(profile: ProteinTokenizerProfile) -> String {
    profile.as_str().to_string()
}

pub(crate) fn profile_vocabulary(profile: ProteinTokenizerProfile) -> &'static Vocabulary {
    match profile {
        ProteinTokenizerProfile::Protein20 | ProteinTokenizerProfile::Protein20Special => {
            protein_20_vocabulary()
        }
        ProteinTokenizerProfile::DnaIupac | ProteinTokenizerProfile::DnaIupacSpecial => {
            dna_iupac_vocabulary()
        }
        ProteinTokenizerProfile::RnaIupac | ProteinTokenizerProfile::RnaIupacSpecial => {
            rna_iupac_vocabulary()
        }
    }
}

pub(crate) fn special_tokens_for_profile(profile: ProteinTokenizerProfile) -> SpecialTokenSet {
    let unknown_token_id = profile_vocabulary(profile).unknown_token_id;
    SpecialTokenSet {
        unk: special("UNK", unknown_token_id),
        pad: special("PAD", unknown_token_id + 1),
        cls: special("CLS", unknown_token_id + 2),
        sep: special("SEP", unknown_token_id + 3),
        mask: special("MASK", unknown_token_id + 4),
    }
}

fn special(name: &str, token_id: u8) -> SpecialToken {
    SpecialToken {
        token: format!("<{name}>"),
        token_id,
    }
}

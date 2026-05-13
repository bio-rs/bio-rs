use crate::sequence::PROTEIN_20;
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;

/// Errors that can occur during tokenizer vocabulary operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenizerError {
    /// Failed to parse vocabulary JSON.
    InvalidVocabJson(String),
}

impl std::fmt::Display for TokenizerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidVocabJson(msg) => write!(f, "invalid vocabulary JSON: {msg}"),
        }
    }
}

impl std::error::Error for TokenizerError {}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Token vocabulary and unknown-token policy.
pub struct Vocabulary {
    /// Vocabulary name.
    pub name: String,
    /// Ordered token definitions.
    pub tokens: Vec<VocabToken>,
    /// Token ID emitted for unresolved residues.
    pub unknown_token_id: u8,
    /// Policy for unknown or unsupported residues.
    pub unknown_token_policy: UnknownTokenPolicy,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Mapping from one residue to one token ID.
pub struct VocabToken {
    /// Residue symbol.
    pub residue: char,
    /// Token ID.
    pub token_id: u8,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
/// Policy used when tokenization encounters ambiguous or invalid residues.
pub enum UnknownTokenPolicy {
    /// Emit the unknown token and record the residue as a warning or error.
    WarnOrErrorWithUnknownToken,
}

/// Borrow the cached built-in `protein-20` vocabulary.
pub fn load_protein_20_vocab() -> &'static Vocabulary {
    protein_20_vocabulary()
}

/// Borrow the cached built-in `protein-20` vocabulary.
pub fn protein_20_vocabulary() -> &'static Vocabulary {
    static VOCABULARY: OnceLock<Vocabulary> = OnceLock::new();
    VOCABULARY.get_or_init(|| Vocabulary {
        name: PROTEIN_20.to_string(),
        tokens: protein_20_vocab_tokens().to_vec(),
        unknown_token_id: PROTEIN_20_UNKNOWN_TOKEN_ID,
        unknown_token_policy: protein_20_unknown_token_policy(),
    })
}

/// Load a vocabulary from its JSON representation.
pub fn load_vocab_json(input: &str) -> Result<Vocabulary, TokenizerError> {
    serde_json::from_str(input).map_err(|e| TokenizerError::InvalidVocabJson(e.to_string()))
}

/// Return the unknown-token policy used by the built-in `protein-20` vocabulary.
pub const fn protein_20_unknown_token_policy() -> UnknownTokenPolicy {
    UnknownTokenPolicy::WarnOrErrorWithUnknownToken
}

/// Unknown token ID emitted for unresolved residues in the built-in vocabulary.
pub const PROTEIN_20_UNKNOWN_TOKEN_ID: u8 = 20;
pub(crate) const TOKEN_LOOKUP_MISSING: u8 = u8::MAX;

/// Borrow the static built-in `protein-20` token definitions.
pub fn protein_20_vocab_tokens() -> &'static [VocabToken; 20] {
    &PROTEIN_20_VOCAB_TOKENS
}

const PROTEIN_20_VOCAB_TOKENS: [VocabToken; 20] = [
    VocabToken {
        residue: 'A',
        token_id: 0,
    },
    VocabToken {
        residue: 'C',
        token_id: 1,
    },
    VocabToken {
        residue: 'D',
        token_id: 2,
    },
    VocabToken {
        residue: 'E',
        token_id: 3,
    },
    VocabToken {
        residue: 'F',
        token_id: 4,
    },
    VocabToken {
        residue: 'G',
        token_id: 5,
    },
    VocabToken {
        residue: 'H',
        token_id: 6,
    },
    VocabToken {
        residue: 'I',
        token_id: 7,
    },
    VocabToken {
        residue: 'K',
        token_id: 8,
    },
    VocabToken {
        residue: 'L',
        token_id: 9,
    },
    VocabToken {
        residue: 'M',
        token_id: 10,
    },
    VocabToken {
        residue: 'N',
        token_id: 11,
    },
    VocabToken {
        residue: 'P',
        token_id: 12,
    },
    VocabToken {
        residue: 'Q',
        token_id: 13,
    },
    VocabToken {
        residue: 'R',
        token_id: 14,
    },
    VocabToken {
        residue: 'S',
        token_id: 15,
    },
    VocabToken {
        residue: 'T',
        token_id: 16,
    },
    VocabToken {
        residue: 'V',
        token_id: 17,
    },
    VocabToken {
        residue: 'W',
        token_id: 18,
    },
    VocabToken {
        residue: 'Y',
        token_id: 19,
    },
];

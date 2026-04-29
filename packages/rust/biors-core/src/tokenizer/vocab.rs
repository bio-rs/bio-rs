use crate::sequence::PROTEIN_20;
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Vocabulary {
    pub name: String,
    pub tokens: Vec<VocabToken>,
    pub unknown_token_id: u8,
    pub unknown_token_policy: UnknownTokenPolicy,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VocabToken {
    pub residue: char,
    pub token_id: u8,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UnknownTokenPolicy {
    WarnOrErrorWithUnknownToken,
}

pub fn load_protein_20_vocab() -> Vocabulary {
    protein_20_vocabulary().clone()
}

pub fn protein_20_vocabulary() -> &'static Vocabulary {
    static VOCABULARY: OnceLock<Vocabulary> = OnceLock::new();
    VOCABULARY.get_or_init(|| Vocabulary {
        name: PROTEIN_20.to_string(),
        tokens: protein_20_vocab_tokens().to_vec(),
        unknown_token_id: PROTEIN_20_UNKNOWN_TOKEN_ID,
        unknown_token_policy: protein_20_unknown_token_policy(),
    })
}

pub fn load_vocab_json(input: &str) -> Result<Vocabulary, serde_json::Error> {
    serde_json::from_str(input)
}

pub const fn protein_20_unknown_token_policy() -> UnknownTokenPolicy {
    UnknownTokenPolicy::WarnOrErrorWithUnknownToken
}

pub const PROTEIN_20_UNKNOWN_TOKEN_ID: u8 = 20;
pub(crate) const TOKEN_LOOKUP_MISSING: u8 = u8::MAX;

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

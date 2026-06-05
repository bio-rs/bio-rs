use super::config::{profile_vocabulary_name, special_tokens_for_profile};
use super::lookup::{push_tokenized_residue, push_tokenized_residue_byte};
use super::{
    load_protein_20_vocab, protein_20_vocabulary, ProteinTokenizerConfig, TokenizedProtein,
    Vocabulary,
};
use crate::sequence::{NormalizedResidue, ProteinSequence, PROTEIN_20};

pub trait Tokenizer {
    fn alphabet(&self) -> &'static str;
    /// Owned vocabulary for callers that need a serializable value.
    fn vocabulary(&self) -> Vocabulary;
    fn tokenize(&self, protein: &ProteinSequence) -> TokenizedProtein;
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct ProteinTokenizer;

impl Tokenizer for ProteinTokenizer {
    fn alphabet(&self) -> &'static str {
        PROTEIN_20
    }

    fn vocabulary(&self) -> Vocabulary {
        load_protein_20_vocab().clone()
    }

    fn tokenize(&self, protein: &ProteinSequence) -> TokenizedProtein {
        tokenize_protein(protein)
    }
}

impl ProteinTokenizer {
    /// Borrow the cached built-in vocabulary without allocating a new token vector.
    pub fn vocabulary_ref(&self) -> &'static Vocabulary {
        protein_20_vocabulary()
    }
}

pub fn tokenize_protein(protein: &ProteinSequence) -> TokenizedProtein {
    tokenize_protein_with_config(
        protein,
        &ProteinTokenizerConfig {
            profile: super::ProteinTokenizerProfile::Protein20,
            add_special_tokens: false,
        },
    )
}

pub fn tokenize_protein_with_config(
    protein: &ProteinSequence,
    config: &ProteinTokenizerConfig,
) -> TokenizedProtein {
    let sequence = crate::sequence::normalize_sequence_bytes(&protein.sequence);
    let special_token_count = if config.add_special_tokens { 2 } else { 0 };
    let mut tokens = Vec::with_capacity(sequence.len() + special_token_count);
    let mut warnings = Vec::new();
    let mut errors = Vec::new();

    if config.add_special_tokens {
        tokens.push(special_tokens_for_profile(config.profile).cls.token_id);
    }

    crate::sequence::for_each_normalized_residue(&sequence, |residue| match residue {
        NormalizedResidue::Byte { value, position } => {
            push_tokenized_residue_byte(
                config.profile,
                value,
                position,
                &mut tokens,
                &mut warnings,
                &mut errors,
            );
        }
        NormalizedResidue::Char { value, position } => {
            push_tokenized_residue(
                config.profile,
                value,
                position,
                &mut tokens,
                &mut warnings,
                &mut errors,
            );
        }
    });

    if config.add_special_tokens {
        tokens.push(special_tokens_for_profile(config.profile).sep.token_id);
    }

    TokenizedProtein {
        id: protein.id.clone(),
        length: tokens.len(),
        alphabet: profile_vocabulary_name(config.profile),
        valid: warnings.is_empty() && errors.is_empty(),
        tokens,
        warnings,
        errors,
    }
}

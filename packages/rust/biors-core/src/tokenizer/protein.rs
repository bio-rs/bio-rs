use super::config::{profile_vocabulary_name, protein_special_tokens};
use super::lookup::{push_tokenized_residue, push_tokenized_residue_byte};
use super::{
    load_protein_20_vocab, protein_20_vocabulary, ProteinTokenizerConfig, TokenizedProtein,
    Vocabulary,
};
use crate::sequence::{ProteinSequence, PROTEIN_20};

/// Trait implemented by protein tokenizers.
pub trait Tokenizer {
    /// Alphabet name supported by this tokenizer.
    fn alphabet(&self) -> &'static str;
    /// Owned vocabulary for callers that need a serializable value.
    fn vocabulary(&self) -> Vocabulary;
    /// Tokenize one protein sequence.
    fn tokenize(&self, protein: &ProteinSequence) -> TokenizedProtein;
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
/// Built-in tokenizer for the `protein-20` residue vocabulary.
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

/// Tokenize one protein sequence with the built-in `protein-20` tokenizer.
pub fn tokenize_protein(protein: &ProteinSequence) -> TokenizedProtein {
    tokenize_protein_with_config(
        protein,
        &ProteinTokenizerConfig {
            profile: super::ProteinTokenizerProfile::Protein20,
            add_special_tokens: false,
        },
    )
}

/// Tokenize one protein sequence with a profile-aware tokenizer config.
pub fn tokenize_protein_with_config(
    protein: &ProteinSequence,
    config: &ProteinTokenizerConfig,
) -> TokenizedProtein {
    let mut tokens = Vec::with_capacity(protein.sequence.len());
    let mut warnings = Vec::with_capacity(protein.sequence.len());
    let mut errors = Vec::with_capacity(protein.sequence.len());

    if config.add_special_tokens {
        tokens.push(protein_special_tokens().cls.token_id);
    }

    if protein.sequence.is_ascii() {
        for (index, byte) in protein.sequence.iter().enumerate() {
            push_tokenized_residue_byte(*byte, index + 1, &mut tokens, &mut warnings, &mut errors);
        }
    } else if let Ok(s) = std::str::from_utf8(&protein.sequence) {
        for (index, residue) in s.chars().enumerate() {
            push_tokenized_residue(residue, index + 1, &mut tokens, &mut warnings, &mut errors);
        }
    } else {
        for (index, byte) in protein.sequence.iter().enumerate() {
            push_tokenized_residue_byte(*byte, index + 1, &mut tokens, &mut warnings, &mut errors);
        }
    }

    if config.add_special_tokens {
        tokens.push(protein_special_tokens().sep.token_id);
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

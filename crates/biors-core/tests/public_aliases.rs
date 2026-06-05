use biors_core::{
    sequence::{BiologicalSequence, FastaSequence},
    tokenizer::{TokenizedSequence, TokenizerConfig, TokenizerProfile},
};

#[test]
fn sequence_generic_aliases_compile_for_public_api_users() {
    let record = BiologicalSequence::new_normalized("seq1", "ACDE");
    let fasta_record: FastaSequence = record.clone();
    let config = TokenizerConfig {
        profile: TokenizerProfile::Protein20,
        add_special_tokens: false,
    };
    let tokenized: TokenizedSequence =
        biors_core::tokenizer::tokenize_protein_with_config(&fasta_record, &config);

    assert_eq!(record.id, "seq1");
    assert_eq!(tokenized.alphabet, "protein-20");
}

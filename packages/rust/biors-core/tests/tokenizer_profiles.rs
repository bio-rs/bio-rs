use biors_core::sequence::ProteinSequence;
use biors_core::tokenizer::{
    inspect_protein_tokenizer_config, load_protein_tokenizer_config_json,
    protein_tokenizer_config_for_profile, tokenize_fasta_records_reader_with_config,
    tokenize_protein_with_config, ProteinTokenizerProfile,
};
use std::io::Cursor;

#[test]
fn protein_20_special_profile_wraps_sequences_with_cls_and_sep_tokens() {
    let config = protein_tokenizer_config_for_profile(ProteinTokenizerProfile::Protein20Special);
    let tokenized = tokenize_protein_with_config(
        &ProteinSequence {
            id: "seq1".to_string(),
            sequence: b"ACD".to_vec(),
        },
        &config,
    );

    assert_eq!(tokenized.alphabet, "protein-20-special");
    assert_eq!(tokenized.tokens, vec![22, 0, 1, 2, 23]);
    assert_eq!(tokenized.length, 5);
    assert!(tokenized.valid);
}

#[test]
fn tokenizer_config_json_selects_profile_and_special_token_policy() {
    let config = load_protein_tokenizer_config_json(
        r#"{
          "profile": "protein-20-special",
          "add_special_tokens": true
        }"#,
    )
    .expect("valid tokenizer config");

    let output = tokenize_fasta_records_reader_with_config(Cursor::new(">seq1\nACD\n"), &config)
        .expect("reader tokenization");

    assert_eq!(output.records[0].alphabet, "protein-20-special");
    assert_eq!(output.records[0].tokens, vec![22, 0, 1, 2, 23]);
}

#[test]
fn tokenizer_inspection_exposes_vocab_and_special_tokens() {
    let config = protein_tokenizer_config_for_profile(ProteinTokenizerProfile::Protein20Special);
    let inspection = inspect_protein_tokenizer_config(&config);

    assert_eq!(
        inspection.profile,
        ProteinTokenizerProfile::Protein20Special
    );
    assert_eq!(inspection.vocabulary.name, "protein-20");
    assert_eq!(inspection.vocabulary.tokens.len(), 20);
    assert_eq!(inspection.unknown_token_id, 20);
    assert_eq!(inspection.special_tokens.pad.token_id, 21);
    assert_eq!(inspection.special_tokens.cls.token_id, 22);
    assert_eq!(inspection.special_tokens.sep.token_id, 23);
    assert_eq!(inspection.special_tokens.mask.token_id, 24);
}

#[test]
fn nucleotide_profiles_tokenize_iupac_dna_and_rna() {
    let dna_config = protein_tokenizer_config_for_profile(ProteinTokenizerProfile::DnaIupac);
    let dna =
        tokenize_fasta_records_reader_with_config(Cursor::new(">dna1\nACGTN?\n"), &dna_config)
            .expect("dna tokenization");

    assert_eq!(dna.records[0].alphabet, "dna-iupac");
    assert_eq!(dna.records[0].tokens, vec![0, 1, 2, 3, 4, 4]);
    assert_eq!(dna.records[0].warnings.len(), 1);
    assert_eq!(dna.records[0].errors.len(), 1);

    let rna_config = protein_tokenizer_config_for_profile(ProteinTokenizerProfile::RnaIupacSpecial);
    let rna = tokenize_fasta_records_reader_with_config(Cursor::new(">rna1\nACGUN\n"), &rna_config)
        .expect("rna tokenization");

    assert_eq!(rna.records[0].alphabet, "rna-iupac-special");
    assert_eq!(rna.records[0].tokens, vec![6, 0, 1, 2, 3, 4, 7]);
    assert_eq!(rna.records[0].warnings.len(), 1);
    assert_eq!(rna.records[0].errors.len(), 0);
}

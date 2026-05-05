use biors_core::{
    load_vocab_json, protein_20_vocabulary, stable_input_hash, tokenize_fasta_records,
    tokenize_fasta_records_reader, ProteinSequence, ProteinTokenizer, Tokenizer,
    UnknownTokenPolicy, PROTEIN_20_UNKNOWN_TOKEN_ID,
};
use std::io::Cursor;

#[test]
fn protein_tokenizer_trait_matches_public_tokenize_function() {
    let tokenizer = ProteinTokenizer;
    let record = ProteinSequence {
        id: "seq1".to_string(),
        sequence: "ACDE".to_string(),
    };

    assert_eq!(tokenizer.alphabet(), "protein-20");
    assert_eq!(tokenizer.vocabulary().tokens.len(), 20);
    assert_eq!(
        tokenizer.vocabulary().unknown_token_id,
        PROTEIN_20_UNKNOWN_TOKEN_ID
    );
    assert_eq!(
        tokenizer.vocabulary().unknown_token_policy,
        UnknownTokenPolicy::WarnOrErrorWithUnknownToken
    );
    assert_eq!(tokenizer.tokenize(&record).tokens, vec![0, 1, 2, 3]);
}

#[test]
fn protein_20_vocabulary_can_be_borrowed_without_rebuilding_tokens() {
    let first = protein_20_vocabulary();
    let second = ProteinTokenizer.vocabulary_ref();

    assert!(std::ptr::eq(first, second));
    assert_eq!(first.tokens.len(), 20);
    assert_eq!(first.tokens.as_ptr(), second.tokens.as_ptr());
    assert_eq!(first.unknown_token_id, PROTEIN_20_UNKNOWN_TOKEN_ID);
}

#[test]
fn loads_vocab_from_json_contract() {
    let vocab = load_vocab_json(
        r#"{
          "name": "protein-20",
          "tokens": [{ "residue": "A", "token_id": 0 }],
          "unknown_token_id": 20,
          "unknown_token_policy": "warn_or_error_with_unknown_token"
        }"#,
    )
    .expect("valid vocab JSON");

    assert_eq!(vocab.name, "protein-20");
    assert_eq!(vocab.tokens[0].residue, 'A');
    assert_eq!(vocab.unknown_token_id, PROTEIN_20_UNKNOWN_TOKEN_ID);
}

#[test]
fn tokenizer_preserves_sequence_length_with_unknown_tokens() {
    let tokenized = tokenize_fasta_records(">seq1\nAX*\n").expect("valid FASTA envelope");

    assert_eq!(tokenized[0].length, 3);
    assert_eq!(
        tokenized[0].tokens,
        vec![0, PROTEIN_20_UNKNOWN_TOKEN_ID, PROTEIN_20_UNKNOWN_TOKEN_ID]
    );
    assert_eq!(tokenized[0].tokens.len(), tokenized[0].length);
    assert_eq!(tokenized[0].warnings.len(), 1);
    assert_eq!(tokenized[0].errors.len(), 1);
}

#[test]
fn tokenizes_fasta_from_reader_and_reports_input_hash() {
    let raw = ">seq1\nACDE\n";
    let output = tokenize_fasta_records_reader(Cursor::new(raw)).expect("reader tokenization");

    assert_eq!(output.input_hash, stable_input_hash(raw));
    assert_eq!(output.records[0].tokens, vec![0, 1, 2, 3]);
}

#[test]
fn tokenizes_lowercase_reader_input_without_changing_public_issue_residues() {
    let raw = ">seq1\nacx?\n";
    let output = tokenize_fasta_records_reader(Cursor::new(raw)).expect("reader tokenization");
    let record = &output.records[0];

    assert_eq!(
        record.tokens,
        vec![
            0,
            1,
            PROTEIN_20_UNKNOWN_TOKEN_ID,
            PROTEIN_20_UNKNOWN_TOKEN_ID
        ]
    );
    assert_eq!(record.warnings[0].residue, 'X');
    assert_eq!(record.warnings[0].position, 3);
    assert_eq!(record.errors[0].residue, '?');
    assert_eq!(record.errors[0].position, 4);
}

#[test]
fn reader_fasta_path_preserves_unicode_fallback_behavior() {
    let raw = ">seq1\nACΩ\n";
    let output =
        tokenize_fasta_records_reader(Cursor::new(raw)).expect("reader handles UTF-8 FASTA");

    assert_eq!(output.input_hash, stable_input_hash(raw));
    assert_eq!(output.records[0].length, 3);
    assert_eq!(
        output.records[0].tokens,
        vec![0, 1, PROTEIN_20_UNKNOWN_TOKEN_ID]
    );
    assert_eq!(output.records[0].errors[0].residue, 'Ω');
    assert_eq!(output.records[0].errors[0].position, 3);
}

#[test]
fn summarizes_fasta_from_reader_without_materializing_tokens() {
    let raw = ">valid\nACDE\n>warn\nXBZ\n>invalid\nA?\n";
    let output =
        biors_core::summarize_fasta_records_reader(Cursor::new(raw)).expect("reader summary");

    assert_eq!(output.input_hash, stable_input_hash(raw));
    assert_eq!(output.summary.records, 3);
    assert_eq!(output.summary.total_length, 9);
    assert_eq!(output.summary.valid_records, 1);
    assert_eq!(output.summary.warning_count, 3);
    assert_eq!(output.summary.error_count, 1);
}

#[test]
fn tokenizer_invariants_hold_for_ascii_residue_corpus() {
    let records = tokenize_fasta_records(">seq1\nACDEFGHIKLMNPQRSTVWYXBZJUO*\n")
        .expect("valid FASTA envelope");
    let record = &records[0];

    assert_eq!(record.length, record.tokens.len());
    assert_eq!(record.warnings.len(), 6);
    assert_eq!(record.errors.len(), 1);

    for issue in record.warnings.iter().chain(record.errors.iter()) {
        assert_eq!(
            record.tokens[issue.position - 1],
            PROTEIN_20_UNKNOWN_TOKEN_ID
        );
    }

    assert_eq!(&record.tokens[..20], &(0u8..20).collect::<Vec<_>>());
}

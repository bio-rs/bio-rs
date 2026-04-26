use biors_core::{
    build_model_inputs, build_model_inputs_checked, parse_fasta_records, tokenize_fasta_records,
    validate_fasta_input, BioRsError, ModelInputBuildError, ModelInputPolicy, PaddingPolicy,
    ProteinSequence, ProteinTokenizer, Tokenizer, UnknownTokenPolicy,
};

#[test]
fn parses_crlf_and_ignores_empty_lines() {
    let records = parse_fasta_records(">seq1\r\nAC\r\n\r\nDE\r\n").expect("valid FASTA");

    assert_eq!(
        records,
        vec![ProteinSequence {
            id: "seq1".to_string(),
            sequence: "ACDE".to_string(),
        }]
    );
}

#[test]
fn reports_line_and_record_index_for_invalid_fasta() {
    let error = parse_fasta_records(">seq1\nACDE\n>seq2\n").expect_err("empty record fails");

    assert_eq!(
        error,
        BioRsError::MissingSequence {
            id: "seq2".to_string(),
            line: 3,
            record_index: 1,
        }
    );
    assert_eq!(error.code(), "fasta.missing_sequence");
}

#[test]
fn validates_sequences_with_ambiguous_residue_policy() {
    let report = validate_fasta_input(">seq1\nAX*\n").expect("valid FASTA envelope");

    assert_eq!(report.records, 1);
    assert_eq!(report.valid_records, 0);
    assert_eq!(report.warning_count, 1);
    assert_eq!(report.error_count, 1);
}

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
        tokenizer.vocabulary().unknown_token_policy,
        UnknownTokenPolicy::WarnOrErrorWithoutToken
    );
    assert_eq!(tokenizer.tokenize(&record).tokens, vec![0, 1, 2, 3]);
}

#[test]
fn builds_deterministic_model_input_with_attention_mask() {
    let tokenized = tokenize_fasta_records(">seq1\nACDEFG\n").expect("valid FASTA");
    let model_input = build_model_inputs(
        &tokenized,
        ModelInputPolicy {
            max_length: 4,
            pad_token_id: 0,
            padding: PaddingPolicy::FixedLength,
        },
    );

    assert_eq!(model_input.records[0].input_ids, vec![0, 1, 2, 3]);
    assert_eq!(model_input.records[0].attention_mask, vec![1, 1, 1, 1]);
    assert!(model_input.records[0].truncated);
}

#[test]
fn pads_model_input_to_fixed_length() {
    let tokenized = tokenize_fasta_records(">seq1\nAC\n").expect("valid FASTA");
    let model_input = build_model_inputs(
        &tokenized,
        ModelInputPolicy {
            max_length: 4,
            pad_token_id: 255,
            padding: PaddingPolicy::FixedLength,
        },
    );

    assert_eq!(model_input.records[0].input_ids, vec![0, 1, 255, 255]);
    assert_eq!(model_input.records[0].attention_mask, vec![1, 1, 0, 0]);
    assert!(!model_input.records[0].truncated);
}

#[test]
fn preserves_unpadded_model_input_when_no_padding_is_requested() {
    let tokenized = tokenize_fasta_records(">seq1\nAC\n").expect("valid FASTA");
    let model_input = build_model_inputs(
        &tokenized,
        ModelInputPolicy {
            max_length: 4,
            pad_token_id: 255,
            padding: PaddingPolicy::NoPadding,
        },
    );

    assert_eq!(model_input.records[0].input_ids, vec![0, 1]);
    assert_eq!(model_input.records[0].attention_mask, vec![1, 1]);
    assert!(!model_input.records[0].truncated);
}

#[test]
fn rejects_model_input_for_sequences_with_ambiguous_or_invalid_residues() {
    let tokenized = tokenize_fasta_records(">seq1\nAX*\n").expect("valid FASTA envelope");
    let error = build_model_inputs_checked(
        &tokenized,
        ModelInputPolicy {
            max_length: 8,
            pad_token_id: 0,
            padding: PaddingPolicy::FixedLength,
        },
    )
    .expect_err("invalid tokenized sequence should not become model-ready input");

    assert_eq!(
        error,
        ModelInputBuildError::InvalidTokenizedSequence {
            id: "seq1".to_string(),
            warning_count: 1,
            error_count: 1,
        }
    );
}

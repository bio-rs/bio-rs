use biors_core::{
    build_model_inputs_checked, build_model_inputs_unchecked, load_vocab_json, parse_fasta_records,
    parse_fasta_records_reader, protein_20_vocabulary, stable_input_hash, tokenize_fasta_records,
    tokenize_fasta_records_reader, validate_fasta_input, validate_fasta_reader, BioRsError,
    FastaReadError, ModelInputBuildError, ModelInputPolicy, PaddingPolicy, ProteinSequence,
    ProteinTokenizer, Tokenizer, UnknownTokenPolicy, PROTEIN_20_UNKNOWN_TOKEN_ID,
};
use std::io::{Cursor, ErrorKind};
use std::path::Path;

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
fn parses_fasta_from_reader_without_preloading_input() {
    let input = Cursor::new(">seq1\nAC\n>seq2\nDE\n");
    let parsed = parse_fasta_records_reader(input).expect("valid FASTA from reader");

    assert_eq!(parsed.records.len(), 2);
    assert_eq!(parsed.records[0].id, "seq1");
    assert_eq!(
        parsed.input_hash,
        stable_input_hash(">seq1\nAC\n>seq2\nDE\n")
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
fn rejects_empty_fasta_identifier() {
    let error = parse_fasta_records(">\nACDE\n").expect_err("empty FASTA id fails");

    assert_eq!(
        error,
        BioRsError::MissingIdentifier {
            line: 1,
            record_index: 0,
        }
    );
    assert_eq!(error.code(), "fasta.missing_identifier");
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
fn validates_fasta_from_reader_with_same_contract_as_string_api() {
    let input = Cursor::new(">seq1\nAX*\n");
    let report = validate_fasta_reader(input).expect("valid FASTA envelope");

    assert_eq!(report.records, 1);
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
fn reader_fasta_path_reports_invalid_utf8_as_read_failure() {
    let raw = b">seq1\nAC\xff\n";
    let error = parse_fasta_records_reader(Cursor::new(raw))
        .expect_err("invalid UTF-8 should remain an I/O-style read failure");

    match error {
        FastaReadError::Io(error) => assert_eq!(error.kind(), ErrorKind::InvalidData),
        other => panic!("expected invalid UTF-8 read failure, got {other:?}"),
    }
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
fn fixture_corpus_covers_valid_and_invalid_fasta_contracts() {
    let fixture_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/fasta");
    let valid = fixture_dir.join("valid/mixed_case_whitespace.fasta");
    let missing_header = fixture_dir.join("invalid/missing_header.fasta");
    let missing_sequence = fixture_dir.join("invalid/missing_sequence.fasta");
    let empty_identifier = fixture_dir.join("invalid/empty_identifier.fasta");

    for path in [
        &valid,
        &missing_header,
        &missing_sequence,
        &empty_identifier,
    ] {
        assert!(path.exists(), "missing FASTA fixture: {}", path.display());
    }

    let valid_input = std::fs::read_to_string(valid).expect("read valid FASTA fixture");
    let records = parse_fasta_records(&valid_input).expect("valid fixture parses");
    assert_eq!(records[0].id, "seq-valid");
    assert_eq!(records[0].sequence, "ACDEXBZJUO");

    let missing_header_input =
        std::fs::read_to_string(missing_header).expect("read missing-header fixture");
    assert_eq!(
        parse_fasta_records(&missing_header_input).expect_err("missing header fails"),
        BioRsError::MissingHeader { line: 1 }
    );

    let missing_sequence_input =
        std::fs::read_to_string(missing_sequence).expect("read missing-sequence fixture");
    assert_eq!(
        parse_fasta_records(&missing_sequence_input).expect_err("missing sequence fails"),
        BioRsError::MissingSequence {
            id: "seq-empty".to_string(),
            line: 1,
            record_index: 0,
        }
    );

    let empty_identifier_input =
        std::fs::read_to_string(empty_identifier).expect("read empty-identifier fixture");
    assert_eq!(
        parse_fasta_records(&empty_identifier_input).expect_err("empty identifier fails"),
        BioRsError::MissingIdentifier {
            line: 1,
            record_index: 0,
        }
    );
}

#[test]
fn tokenizer_expected_output_fixture_matches_public_contract() {
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/tokenizer/protein_20_expected.json");

    assert!(
        fixture.exists(),
        "missing tokenizer fixture: {}",
        fixture.display()
    );

    let actual = tokenize_fasta_records(">seq-valid\nACDEXBZJUO*\n").expect("tokenize fixture");
    let expected: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(fixture).expect("read tokenizer fixture"))
            .expect("valid tokenizer fixture JSON");
    let actual = serde_json::to_value(actual).expect("serialize tokenization");

    assert_eq!(actual, expected);
}

#[test]
fn deterministic_invalid_fasta_corpus_keeps_error_codes_stable() {
    let cases = [
        ("", BioRsError::EmptyInput),
        ("   \n\t\n", BioRsError::EmptyInput),
        ("ACDE\n", BioRsError::MissingHeader { line: 1 }),
        (
            ">\nACDE\n",
            BioRsError::MissingIdentifier {
                line: 1,
                record_index: 0,
            },
        ),
        (
            ">seq1\n>seq2\nACDE\n",
            BioRsError::MissingSequence {
                id: "seq1".to_string(),
                line: 1,
                record_index: 0,
            },
        ),
        (
            ">seq1\nACDE\n>seq2\n",
            BioRsError::MissingSequence {
                id: "seq2".to_string(),
                line: 3,
                record_index: 1,
            },
        ),
    ];

    for (input, expected) in cases {
        let error = parse_fasta_records(input).expect_err("invalid FASTA must fail");
        assert_eq!(error, expected);
        assert_eq!(error.code(), expected.code());
    }
}

#[test]
fn string_and_reader_fasta_paths_stay_behaviorally_identical() {
    let inputs = [
        ">seq1\nACDE\n",
        "  >seq1 description\r\n ac de \r\n\r\n>seq2\nxbzjuo*\n",
        ">seq1\nACDE\n>seq2\n",
        "ACDE\n",
        ">\nACDE\n",
    ];

    for input in inputs {
        let string_parse = parse_fasta_records(input);
        let reader_parse =
            parse_fasta_records_reader(Cursor::new(input)).map(|parsed| parsed.records);
        assert_eq!(
            reader_parse.map_err(|error| error.code()),
            string_parse.map_err(|error| error.code())
        );

        let string_tokens = tokenize_fasta_records(input);
        let reader_tokens =
            tokenize_fasta_records_reader(Cursor::new(input)).map(|parsed| parsed.records);
        assert_eq!(
            reader_tokens.map_err(|error| error.code()),
            string_tokens.map_err(|error| error.code())
        );
    }
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

#[test]
fn builds_deterministic_model_input_with_attention_mask() {
    let tokenized = tokenize_fasta_records(">seq1\nACDEFG\n").expect("valid FASTA");
    let model_input = build_model_inputs_unchecked(
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
    let model_input = build_model_inputs_unchecked(
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
    let model_input = build_model_inputs_unchecked(
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

#[test]
fn rejects_zero_length_model_input_policy() {
    let tokenized = tokenize_fasta_records(">seq1\nACDE\n").expect("valid FASTA");
    let error = build_model_inputs_checked(
        &tokenized,
        ModelInputPolicy {
            max_length: 0,
            pad_token_id: 0,
            padding: PaddingPolicy::FixedLength,
        },
    )
    .expect_err("zero max_length should fail");

    assert_eq!(
        error,
        ModelInputBuildError::InvalidPolicy {
            message: "max_length must be greater than zero".to_string(),
        }
    );
}

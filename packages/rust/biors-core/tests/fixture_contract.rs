use biors_core::{parse_fasta_records, tokenize_fasta_records, BioRsError};
use std::path::Path;

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
    assert_eq!(records[0].sequence, b"ACDEXBZJUO");

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

use biors_core::error::{BioRsError, FastaReadError};
use biors_core::fasta::{parse_fasta_records, parse_fasta_records_reader};
use biors_core::sequence::ProteinSequence;
use biors_core::verification::stable_input_hash;
use std::io::{Cursor, ErrorKind};

#[test]
fn parses_crlf_and_ignores_empty_lines() {
    let records = parse_fasta_records(">seq1\r\nAC\r\n\r\nDE\r\n").expect("valid FASTA");

    assert_eq!(
        records,
        vec![ProteinSequence {
            id: "seq1".to_string(),
            sequence: b"ACDE".to_vec(),
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
fn reader_fasta_path_reports_invalid_utf8_as_read_failure() {
    let raw = b">seq1\nAC\xff\n";
    let error =
        parse_fasta_records_reader(Cursor::new(raw)).expect_err("invalid UTF-8 read failure");

    match error {
        FastaReadError::Io(error) => assert_eq!(error.kind(), ErrorKind::InvalidData),
        other => panic!("expected invalid UTF-8 read failure, got {other:?}"),
    }
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

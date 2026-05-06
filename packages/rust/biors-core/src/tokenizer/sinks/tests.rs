use super::*;
use crate::fasta_scan::{scan_fasta_reader, scan_fasta_str};
use crate::tokenizer::PROTEIN_20_UNKNOWN_TOKEN_ID;
use std::io::Cursor;

#[test]
fn tokenized_record_sink_tokenizes_ascii_sequence_via_bytes() {
    let mut sink = TokenizedRecordSink::default();
    scan_fasta_reader(Cursor::new(b">seq1\nACDE\n"), &mut sink, |_line| {}).expect("valid FASTA");

    assert_eq!(sink.records.len(), 1);
    assert_eq!(sink.records[0].id, "seq1");
    assert_eq!(sink.records[0].length, 4);
    assert_eq!(sink.records[0].tokens, vec![0, 1, 2, 3]);
    assert!(sink.records[0].valid);
    assert!(sink.records[0].warnings.is_empty());
    assert!(sink.records[0].errors.is_empty());
}

#[test]
fn tokenized_record_sink_tokenizes_non_ascii_sequence_via_str() {
    let mut sink = TokenizedRecordSink::default();
    scan_fasta_str(">seq1\nacde\n", &mut sink).expect("valid FASTA");

    assert_eq!(sink.records.len(), 1);
    assert_eq!(sink.records[0].length, 4);
    assert_eq!(sink.records[0].tokens, vec![0, 1, 2, 3]);
    assert!(sink.records[0].valid);
}

#[test]
fn tokenized_record_sink_warns_for_ambiguous_residue() {
    let mut sink = TokenizedRecordSink::default();
    scan_fasta_reader(Cursor::new(b">seq1\nACXDE\n"), &mut sink, |_line| {}).expect("valid FASTA");

    assert_eq!(
        sink.records[0].tokens,
        vec![0, 1, PROTEIN_20_UNKNOWN_TOKEN_ID, 2, 3]
    );
    assert!(!sink.records[0].valid);
    assert_eq!(sink.records[0].warnings.len(), 1);
    assert_eq!(sink.records[0].warnings[0].residue, 'X');
    assert_eq!(sink.records[0].warnings[0].position, 3);
    assert!(sink.records[0].errors.is_empty());
}

#[test]
fn tokenized_record_sink_errors_for_invalid_residue() {
    let mut sink = TokenizedRecordSink::default();
    scan_fasta_reader(Cursor::new(b">seq1\nAC*DE\n"), &mut sink, |_line| {}).expect("valid FASTA");

    assert_eq!(
        sink.records[0].tokens,
        vec![0, 1, PROTEIN_20_UNKNOWN_TOKEN_ID, 2, 3]
    );
    assert!(!sink.records[0].valid);
    assert!(sink.records[0].warnings.is_empty());
    assert_eq!(sink.records[0].errors.len(), 1);
    assert_eq!(sink.records[0].errors[0].residue, '*');
    assert_eq!(sink.records[0].errors[0].position, 3);
}

#[test]
fn tokenized_record_sink_rejects_empty_sequence() {
    let mut sink = TokenizedRecordSink::default();
    let err = scan_fasta_reader(Cursor::new(b">seq1\n>seq2\nACDE\n"), &mut sink, |_line| {})
        .expect_err("empty record should error");

    assert!(matches!(
        err,
        crate::FastaReadError::Parse(crate::BioRsError::MissingSequence { id, line: 1, record_index: 0 })
        if id == "seq1"
    ));
}

#[test]
fn summary_record_sink_summarizes_ascii_sequence() {
    let mut sink = SummaryRecordSink::default();
    scan_fasta_reader(
        Cursor::new(b">seq1\nACDE\nFGHI\n>seq2\nKLMN\n"),
        &mut sink,
        |_line| {},
    )
    .expect("valid FASTA");

    assert_eq!(sink.summary.records, 2);
    assert_eq!(sink.summary.total_length, 12);
    assert_eq!(sink.summary.valid_records, 2);
    assert_eq!(sink.summary.warning_count, 0);
    assert_eq!(sink.summary.error_count, 0);
}

#[test]
fn summary_record_sink_counts_warnings_and_errors() {
    let mut sink = SummaryRecordSink::default();
    scan_fasta_reader(
        Cursor::new(b">seq1\nACXDE\n>seq2\nAC*DE\n>seq3\nACDE\n"),
        &mut sink,
        |_line| {},
    )
    .expect("valid FASTA");

    assert_eq!(sink.summary.records, 3);
    assert_eq!(sink.summary.total_length, 14);
    assert_eq!(sink.summary.valid_records, 1);
    assert_eq!(sink.summary.warning_count, 1);
    assert_eq!(sink.summary.error_count, 1);
}

#[test]
fn summary_record_sink_rejects_empty_sequence() {
    let mut sink = SummaryRecordSink::default();
    let err = scan_fasta_reader(Cursor::new(b">seq1\n>seq2\nACDE\n"), &mut sink, |_line| {})
        .expect_err("empty record should error");

    assert!(matches!(
        err,
        crate::FastaReadError::Parse(crate::BioRsError::MissingSequence { id, line: 1, record_index: 0 })
        if id == "seq1"
    ));
}

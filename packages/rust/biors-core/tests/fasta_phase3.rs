use biors_core::{
    stable_input_hash, validate_fasta_input_with_kind,
    validate_fasta_reader_summary_with_kind_and_hash, validate_fasta_reader_with_kind_and_hash,
    Diagnostic, SequenceKind, SequenceKindSelection,
};
use std::io::Cursor;

#[test]
fn fasta_auto_detection_assigns_kind_per_record_and_summarizes_mixed_batches() {
    let report = validate_fasta_input_with_kind(
        ">dna\nACGN\n>rna\nACGU\n>protein\nMEEPQSDPSV\n",
        SequenceKindSelection::Auto,
    )
    .expect("valid FASTA envelope");

    assert_eq!(report.records, 3);
    assert_eq!(report.valid_records, 2);
    assert_eq!(report.warning_count, 1);
    assert_eq!(report.error_count, 0);
    assert_eq!(report.kind_counts.dna, 1);
    assert_eq!(report.kind_counts.rna, 1);
    assert_eq!(report.kind_counts.protein, 1);
    assert_eq!(report.sequences[0].kind, SequenceKind::Dna);
    assert_eq!(report.sequences[0].alphabet, "dna-iupac");
    assert_eq!(report.sequences[0].warnings[0].symbol, 'N');
    assert_eq!(report.sequences[1].kind, SequenceKind::Rna);
    assert_eq!(report.sequences[2].kind, SequenceKind::Protein);
}

#[test]
fn explicit_kind_override_uses_kind_specific_errors() {
    let report = validate_fasta_input_with_kind(
        ">rna-looking\nACGU\n",
        SequenceKindSelection::Explicit(SequenceKind::Dna),
    )
    .expect("valid FASTA envelope");

    let record = &report.sequences[0];
    assert_eq!(record.kind, SequenceKind::Dna);
    assert_eq!(record.alphabet, "dna-iupac");
    assert!(!record.valid);
    assert_eq!(record.errors[0].symbol, 'U');
    assert_eq!(record.errors[0].code(), "sequence.invalid_symbol");
    assert!(record.errors[0].message().contains("DNA"));
}

#[test]
fn reader_kind_validation_preserves_input_hash_and_unicode_fallback() {
    let raw = ">unicode\nACΩ\n";
    let output = validate_fasta_reader_with_kind_and_hash(
        Cursor::new(raw),
        SequenceKindSelection::Explicit(SequenceKind::Dna),
    )
    .expect("valid UTF-8 FASTA envelope");

    assert_eq!(output.input_hash, stable_input_hash(raw));
    assert_eq!(output.report.sequences[0].sequence, "ACΩ");
    assert_eq!(output.report.sequences[0].errors[0].symbol, 'Ω');
    assert_eq!(output.report.sequences[0].errors[0].position, 3);
}

#[test]
fn reader_kind_validation_summary_counts_without_record_payloads() {
    let raw = ">dna\nACGN\n>rna\nACGU\n>protein\nMEEPQSDPSV\n";
    let output = validate_fasta_reader_summary_with_kind_and_hash(
        Cursor::new(raw),
        SequenceKindSelection::Auto,
    )
    .expect("valid FASTA envelope");

    assert_eq!(output.input_hash, stable_input_hash(raw));
    assert_eq!(output.summary.records, 3);
    assert_eq!(output.summary.valid_records, 2);
    assert_eq!(output.summary.warning_count, 1);
    assert_eq!(output.summary.error_count, 0);
    assert_eq!(output.summary.kind_counts.dna, 1);
    assert_eq!(output.summary.kind_counts.rna, 1);
    assert_eq!(output.summary.kind_counts.protein, 1);
}

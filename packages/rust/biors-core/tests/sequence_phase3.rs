use biors_core::error::Diagnostic;
use biors_core::sequence::{
    detect_sequence_kind, detect_sequence_kind_with_metadata, validate_fasta_input_with_kind,
    validate_sequence_record, AlphabetPolicy, SequenceKind, SequenceKindSelection, SequenceRecord,
    SymbolClass,
};

#[test]
fn sequence_kind_serializes_as_stable_lowercase_names() {
    assert_eq!(
        serde_json::to_string(&SequenceKind::Protein).unwrap(),
        r#""protein""#
    );
    assert_eq!(
        serde_json::to_string(&SequenceKind::Dna).unwrap(),
        r#""dna""#
    );
    assert_eq!(
        serde_json::to_string(&SequenceKind::Rna).unwrap(),
        r#""rna""#
    );
    assert_eq!(SequenceKind::Dna.alphabet_name(), "dna-iupac");
    assert_eq!(SequenceKind::Rna.alphabet_name(), "rna-iupac");
}

#[test]
fn nucleotide_policies_classify_standard_and_ambiguous_iupac_codes() {
    let dna = AlphabetPolicy::for_kind(SequenceKind::Dna);
    let rna = AlphabetPolicy::for_kind(SequenceKind::Rna);

    for base in ['A', 'C', 'G', 'T'] {
        assert_eq!(dna.classify(base), SymbolClass::Standard, "{base}");
    }
    for base in ['R', 'Y', 'S', 'W', 'K', 'M', 'B', 'D', 'H', 'V', 'N'] {
        assert_eq!(dna.classify(base), SymbolClass::Ambiguous, "{base}");
        assert_eq!(rna.classify(base), SymbolClass::Ambiguous, "{base}");
    }
    assert_eq!(dna.classify('U'), SymbolClass::Invalid);

    for base in ['A', 'C', 'G', 'U'] {
        assert_eq!(rna.classify(base), SymbolClass::Standard, "{base}");
    }
    assert_eq!(rna.classify('T'), SymbolClass::Invalid);
}

#[test]
fn validates_dna_and_rna_sequences_with_kind_specific_diagnostics() {
    let dna = SequenceRecord::new("dna1", "acgtnu?", SequenceKind::Dna);
    let rna = SequenceRecord::new("rna1", "acgun t?", SequenceKind::Rna);

    let dna_report = validate_sequence_record(&dna);
    assert_eq!(dna_report.kind, SequenceKind::Dna);
    assert_eq!(dna_report.alphabet, "dna-iupac");
    assert_eq!(dna_report.sequence, "ACGTNU?");
    assert_eq!(dna_report.warnings.len(), 1);
    assert_eq!(dna_report.warnings[0].symbol, 'N');
    assert_eq!(dna_report.warnings[0].position, 5);
    assert_eq!(dna_report.warnings[0].code(), "sequence.ambiguous_symbol");
    assert_eq!(dna_report.errors.len(), 2);
    assert_eq!(dna_report.errors[0].symbol, 'U');
    assert_eq!(dna_report.errors[0].code(), "sequence.invalid_symbol");

    let rna_report = validate_sequence_record(&rna);
    assert_eq!(rna_report.kind, SequenceKind::Rna);
    assert_eq!(rna_report.alphabet, "rna-iupac");
    assert_eq!(rna_report.sequence, "ACGUNT?");
    assert_eq!(rna_report.warnings[0].symbol, 'N');
    assert_eq!(rna_report.errors[0].symbol, 'T');
    assert!(rna_report.errors[0].message().contains("RNA"));
}

#[test]
fn auto_detection_chooses_fewest_errors_with_dna_tie_default() {
    assert_eq!(detect_sequence_kind("ACGN"), SequenceKind::Dna);
    assert_eq!(detect_sequence_kind("NNNN"), SequenceKind::Dna);
    assert_eq!(detect_sequence_kind("ACGU"), SequenceKind::Rna);
    assert_eq!(detect_sequence_kind("MEEPQSDPSV"), SequenceKind::Protein);
}

#[test]
fn auto_detection_reports_ambiguous_kind_candidates() {
    let acgt = detect_sequence_kind_with_metadata("ACGT");
    assert_eq!(acgt.selected_kind, SequenceKind::Dna);
    assert_eq!(
        acgt.candidate_kinds,
        vec![SequenceKind::Protein, SequenceKind::Dna]
    );
    assert!(acgt.ambiguous);

    let n = detect_sequence_kind_with_metadata("N");
    assert_eq!(n.selected_kind, SequenceKind::Dna);
    assert_eq!(
        n.candidate_kinds,
        vec![SequenceKind::Protein, SequenceKind::Dna, SequenceKind::Rna]
    );
    assert!(n.ambiguous);

    let uuuu = detect_sequence_kind_with_metadata("UUUU");
    assert_eq!(uuuu.selected_kind, SequenceKind::Rna);
    assert_eq!(
        uuuu.candidate_kinds,
        vec![SequenceKind::Protein, SequenceKind::Rna]
    );
    assert!(uuuu.ambiguous);

    let m = detect_sequence_kind_with_metadata("M");
    assert_eq!(m.selected_kind, SequenceKind::Dna);
    assert_eq!(
        m.candidate_kinds,
        vec![SequenceKind::Protein, SequenceKind::Dna, SequenceKind::Rna]
    );
    assert!(m.ambiguous);

    let protein_only = detect_sequence_kind_with_metadata("E");
    assert_eq!(protein_only.selected_kind, SequenceKind::Protein);
    assert_eq!(protein_only.candidate_kinds, vec![SequenceKind::Protein]);
    assert!(!protein_only.ambiguous);
}

#[test]
fn auto_kind_validation_output_includes_detection_metadata() {
    let report = validate_fasta_input_with_kind(
        ">ambiguous\nACGT\n>mixed-invalid\n1\n",
        SequenceKindSelection::Auto,
    )
    .expect("validation report");

    assert_eq!(report.sequences[0].kind, SequenceKind::Dna);
    let detection = report.sequences[0]
        .auto_detection
        .as_ref()
        .expect("auto detection metadata");
    assert!(detection.ambiguous);
    assert_eq!(
        detection.candidate_kinds,
        vec![SequenceKind::Protein, SequenceKind::Dna]
    );

    let mixed_detection = report.sequences[1]
        .auto_detection
        .as_ref()
        .expect("mixed invalid auto detection metadata");
    assert!(mixed_detection.ambiguous);
    assert_eq!(
        mixed_detection.candidate_kinds,
        vec![SequenceKind::Protein, SequenceKind::Dna, SequenceKind::Rna]
    );
}

#[test]
fn explicit_kind_validation_omits_auto_detection_metadata() {
    let report = validate_fasta_input_with_kind(
        ">explicit\nACGT\n",
        SequenceKindSelection::Explicit(SequenceKind::Protein),
    )
    .expect("validation report");

    assert!(report.sequences[0].auto_detection.is_none());
}

#[test]
fn sequence_kind_selection_distinguishes_auto_from_explicit_kinds() {
    assert_eq!(SequenceKindSelection::Auto.explicit_kind(), None);
    assert_eq!(
        SequenceKindSelection::Explicit(SequenceKind::Protein).explicit_kind(),
        Some(SequenceKind::Protein)
    );
}

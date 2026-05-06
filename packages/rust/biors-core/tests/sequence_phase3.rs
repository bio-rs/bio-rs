use biors_core::{
    detect_sequence_kind, validate_sequence_record, AlphabetPolicy, Diagnostic, SequenceKind,
    SequenceKindSelection, SequenceRecord, SymbolClass,
};

#[test]
fn sequence_kind_serializes_as_stable_lowercase_names() {
    assert_eq!(
        serde_json::to_string(&SequenceKind::Protein).expect("serialize protein kind"),
        r#""protein""#
    );
    assert_eq!(
        serde_json::to_string(&SequenceKind::Dna).expect("serialize DNA kind"),
        r#""dna""#
    );
    assert_eq!(
        serde_json::to_string(&SequenceKind::Rna).expect("serialize RNA kind"),
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
fn sequence_kind_selection_distinguishes_auto_from_explicit_kinds() {
    assert_eq!(SequenceKindSelection::Auto.explicit_kind(), None);
    assert_eq!(
        SequenceKindSelection::Explicit(SequenceKind::Protein).explicit_kind(),
        Some(SequenceKind::Protein)
    );
}

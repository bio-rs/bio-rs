use biors_core::fasta::{validate_fasta_input, validate_fasta_reader};
use biors_core::sequence::{ResidueIssue, ValidatedSequence};
use std::io::Cursor;

#[test]
fn validates_sequences_with_ambiguous_residue_policy() {
    let report = validate_fasta_input(">seq1\nAX*\n").expect("valid FASTA envelope");

    assert_eq!(report.records, 1);
    assert_eq!(report.valid_records, 0);
    assert_eq!(report.warning_count, 1);
    assert_eq!(report.error_count, 1);
}

#[test]
fn sequence_validation_report_aggregates_record_validity_and_issue_counts() {
    let report = biors_core::sequence::summarize_validated_sequences(vec![
        ValidatedSequence {
            id: "valid".to_string(),
            sequence: "ACDE".to_string(),
            alphabet: "protein-20".to_string(),
            valid: true,
            warnings: vec![],
            errors: vec![],
        },
        ValidatedSequence {
            id: "warn".to_string(),
            sequence: "XB".to_string(),
            alphabet: "protein-20".to_string(),
            valid: false,
            warnings: vec![
                ResidueIssue {
                    residue: 'X',
                    position: 1,
                },
                ResidueIssue {
                    residue: 'B',
                    position: 2,
                },
            ],
            errors: vec![],
        },
        ValidatedSequence {
            id: "invalid".to_string(),
            sequence: "A?".to_string(),
            alphabet: "protein-20".to_string(),
            valid: false,
            warnings: vec![],
            errors: vec![ResidueIssue {
                residue: '?',
                position: 2,
            }],
        },
    ]);

    assert_eq!(report.records, 3);
    assert_eq!(report.valid_records, 1);
    assert_eq!(report.warning_count, 2);
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

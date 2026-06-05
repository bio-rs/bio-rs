use super::{parse_pdb_record, validate_pdb_reader};

const PDB: &str = "\
HEADER    TEST STRUCTURE                          01-JAN-00   1ABC
TITLE     EXAMPLE STRUCTURE
SEQRES   1 A    4  ALA CYS ASP GLU
REMARK 465     CYS A    2
ATOM      1  N   ALA A   1      11.104  13.207  14.100  1.00 20.00           N
ATOM      2  CA  ALA A   1      12.000  13.500  14.700  1.00 21.00           C
ATOM      3  N   GLU A   4      15.104  17.207  18.100  1.00 20.00           N
HETATM    4  O   HOH A 101      16.104  18.207  19.100  1.00 30.00           O
END
";

#[test]
fn parse_pdb_extracts_chain_residues_atoms_and_sequences() {
    let record = parse_pdb_record(PDB).expect("parse PDB");

    assert_eq!(record.format.as_str(), "pdb");
    assert_eq!(record.id.as_deref(), Some("1ABC"));
    assert_eq!(record.metadata.atom_count, 3);
    assert_eq!(record.metadata.hetero_atom_count, 1);
    assert_eq!(record.metadata.missing_residue_count, 1);
    assert_eq!(record.chains.len(), 1);

    let chain = &record.chains[0];
    assert_eq!(chain.id, "A");
    assert_eq!(chain.coordinate_sequence, "AE");
    assert_eq!(chain.seqres_sequence.as_deref(), Some("ACDE"));
    assert_eq!(chain.missing_residues[0].name, "CYS");
    assert_eq!(chain.residues.len(), 3);
    assert!(chain.residues.iter().any(|residue| residue.hetero));
}

#[test]
fn validate_pdb_reports_subsequence_mapping_and_missing_residue_warning() {
    let report = validate_pdb_reader(PDB.as_bytes()).expect("validate PDB");

    assert!(report.valid);
    assert_eq!(report.warning_count, 1);
    assert_eq!(report.error_count, 0);
    assert_eq!(
        report.chain_reports[0]
            .sequence_mapping
            .coordinate_to_seqres_positions,
        vec![Some(1), Some(4)]
    );
}

#[test]
fn parse_pdb_rejects_invalid_fixed_coordinate_field() {
    let input = "\
ATOM      1  N   ALA A   1      not-num 13.207  14.100  1.00 20.00           N
";
    let error = parse_pdb_record(input).expect_err("invalid coordinate must fail");

    assert_eq!(error.code(), "pdb.invalid_atom_field");
}

#[test]
fn parse_pdb_uses_first_model_coordinates_only() {
    let input = "\
MODEL        1
SEQRES   1 A    1  ALA
ATOM      1  N   ALA A   1      11.104  13.207  14.100  1.00 20.00           N
ENDMDL
MODEL        2
ATOM      2  N   GLY A   2      12.104  13.207  14.100  1.00 20.00           N
ENDMDL
";
    let record = parse_pdb_record(input).expect("parse first model");

    assert_eq!(record.metadata.model_count, 2);
    assert_eq!(record.chains[0].coordinate_sequence, "A");
    assert_eq!(record.metadata.atom_count, 1);
}

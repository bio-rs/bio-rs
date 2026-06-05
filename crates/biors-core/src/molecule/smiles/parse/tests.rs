use super::parse_smiles_records;

#[test]
fn parses_branching_and_double_bonds() {
    let records = parse_smiles_records("CC(=O)O acetate\n").expect("parse smiles");
    let record = &records[0];

    assert_eq!(record.id.as_deref(), Some("acetate"));
    assert_eq!(record.metadata.atom_count, 4);
    assert_eq!(record.metadata.bond_count, 3);
    assert_eq!(record.metadata.branch_count, 1);
}

#[test]
fn parses_aromatic_ring_closures() {
    let records = parse_smiles_records("c1ccccc1 benzene\n").expect("parse benzene");
    let record = &records[0];

    assert_eq!(record.metadata.atom_count, 6);
    assert_eq!(record.metadata.ring_closure_count, 1);
    assert_eq!(record.metadata.aromatic_atom_count, 6);
}

#[test]
fn parses_bracket_atoms_with_charge_and_hydrogen() {
    let records = parse_smiles_records("[NH4+] ammonium\n").expect("parse ammonium");
    let atom = &records[0].graph.atoms.atoms[0];

    assert_eq!(atom.element, "N");
    assert_eq!(atom.explicit_hydrogens, 4);
    assert_eq!(atom.charge, 1);
}

#[test]
fn rejects_unclosed_ring() {
    let error = parse_smiles_records("C1CC\n").expect_err("ring must be closed");

    assert_eq!(error.code(), "smiles.unclosed_ring");
}

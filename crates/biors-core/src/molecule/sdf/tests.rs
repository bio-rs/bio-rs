use super::{parse_sdf_records, SdfParseError};
use crate::formats::BioFormat;

#[test]
fn parses_v2000_sdf_with_properties() {
    let input = "\
ethanol
  bio-rs

  3  2  0  0  0  0            999 V2000
    0.0000    0.0000    0.0000 C   0  0  0  0  0  0  0  0  0  0  0  0
    1.5000    0.0000    0.0000 C   0  0  0  0  0  0  0  0  0  0  0  0
    2.1000    1.2000    0.0000 O   0  0  0  0  0  0  0  0  0  0  0  0
  1  2  1  0  0  0  0
  2  3  1  0  0  0  0
M  END
>  <ASSAY>
active

$$$$
";
    let records = parse_sdf_records(input).expect("parse sdf");

    assert_eq!(records[0].format, BioFormat::Sdf);
    assert_eq!(records[0].metadata.atom_count, 3);
    assert_eq!(records[0].metadata.bond_count, 2);
    assert_eq!(records[0].properties[0].name, "ASSAY");
    assert_eq!(records[0].properties[0].value, "active\n");
}

#[test]
fn rejects_v2000_bond_endpoint_outside_atom_table() {
    let input = "\
bad-bond
  bio-rs

  2  1  0  0  0  0            999 V2000
    0.0000    0.0000    0.0000 C   0  0  0  0  0  0  0  0  0  0  0  0
    1.5000    0.0000    0.0000 O   0  0  0  0  0  0  0  0  0  0  0  0
  1  3  1  0  0  0  0
M  END
$$$$
";

    let error = parse_sdf_records(input).expect_err("invalid endpoint rejected");

    assert!(matches!(error, SdfParseError::InvalidBondLine { .. }));
}

#[test]
fn rejects_v2000_non_finite_coordinate() {
    let input = "\
bad-coordinate
  bio-rs

  1  0  0  0  0  0            999 V2000
       NaN    0.0000    0.0000 C   0  0  0  0  0  0  0  0  0  0  0  0
M  END
$$$$
";

    let error = parse_sdf_records(input).expect_err("non-finite coordinate rejected");

    assert!(matches!(error, SdfParseError::InvalidAtomLine { .. }));
}

#[test]
fn parses_v2000_formal_charges_from_atom_block_and_m_chg_records() {
    let input = "\
charged
  bio-rs

  2  1  0  0  0  0            999 V2000
    0.0000    0.0000    0.0000 N   0  3  0  0  0  0  0  0  0  0  0  0
    1.5000    0.0000    0.0000 O   0  0  0  0  0  0  0  0  0  0  0  0
  1  2  1  0  0  0  0
M  CHG  1   2  -1
M  END
$$$$
";

    let records = parse_sdf_records(input).expect("parse charged sdf");

    assert_eq!(records[0].graph.atoms.atoms[0].charge, 1);
    assert_eq!(records[0].graph.atoms.atoms[1].charge, -1);
}

#[test]
fn parses_v3000_bonds_using_source_atom_ids() {
    let input = "\
v3000-record
  bio-rs

  0  0  0  0  0  0            999 V3000
M  V30 BEGIN CTAB
M  V30 COUNTS 2 1 0 0 0
M  V30 BEGIN ATOM
M  V30 10 C 0.0000 0.0000 0.0000 0
M  V30 20 O 1.5000 0.0000 0.0000 0
M  V30 END ATOM
M  V30 BEGIN BOND
M  V30 1 1 10 20
M  V30 END BOND
M  V30 END CTAB
M  END
$$$$
";

    let records = parse_sdf_records(input).expect("parse v3000");
    let bond = &records[0].graph.bonds.bonds[0];

    assert_eq!(bond.source_atom, 0);
    assert_eq!(bond.target_atom, 1);
    assert_eq!(records[0].metadata.disconnected_component_count, 1);
}

#[test]
fn parses_v3000_atom_charge_property() {
    let input = "\
v3000-record
  bio-rs

  0  0  0  0  0  0            999 V3000
M  V30 BEGIN CTAB
M  V30 COUNTS 1 0 0 0 0
M  V30 BEGIN ATOM
M  V30 10 N 0.0000 0.0000 0.0000 0 CHG=1
M  V30 END ATOM
M  V30 END CTAB
M  END
$$$$
";

    let records = parse_sdf_records(input).expect("parse charged v3000");

    assert_eq!(records[0].graph.atoms.atoms[0].charge, 1);
}

#[test]
fn rejects_v3000_bond_with_unknown_source_atom_id() {
    let input = "\
v3000-record
  bio-rs

  0  0  0  0  0  0            999 V3000
M  V30 BEGIN CTAB
M  V30 COUNTS 2 1 0 0 0
M  V30 BEGIN ATOM
M  V30 10 C 0.0000 0.0000 0.0000 0
M  V30 20 O 1.5000 0.0000 0.0000 0
M  V30 END ATOM
M  V30 BEGIN BOND
M  V30 1 1 10 30
M  V30 END BOND
M  V30 END CTAB
M  END
$$$$
";

    let error = parse_sdf_records(input).expect_err("unknown endpoint rejected");

    assert!(matches!(error, SdfParseError::InvalidBondLine { .. }));
}

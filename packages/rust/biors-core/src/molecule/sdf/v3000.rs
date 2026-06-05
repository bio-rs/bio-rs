use crate::molecule::{MoleculeAtom, MoleculeBond, MoleculeCoordinate, MoleculeProperty};

use super::{parse_one_based_index, parse_sdf_properties, sdf_bond_order, SdfParseError};

pub(super) fn parse_v3000(
    lines: &[(usize, String)],
    record_index: usize,
) -> Result<(Vec<MoleculeAtom>, Vec<MoleculeBond>, Vec<MoleculeProperty>), SdfParseError> {
    let mut atoms = Vec::new();
    let mut bonds = Vec::new();
    let mut in_atom = false;
    let mut in_bond = false;
    for (line_number, line) in lines {
        let Some(payload) = line.trim_start().strip_prefix("M  V30 ") else {
            continue;
        };
        match payload {
            "BEGIN ATOM" => in_atom = true,
            "END ATOM" => in_atom = false,
            "BEGIN BOND" => in_bond = true,
            "END BOND" => in_bond = false,
            _ if in_atom => atoms.push(parse_v3000_atom(
                payload,
                *line_number,
                atoms.len(),
                record_index,
            )?),
            _ if in_bond => bonds.push(parse_v3000_bond(
                payload,
                *line_number,
                bonds.len(),
                record_index,
            )?),
            _ => {}
        }
    }
    if atoms.is_empty() {
        return Err(SdfParseError::InvalidV3000Line {
            line: lines.first().map(|(line, _)| *line).unwrap_or(1),
            record_index,
        });
    }
    let properties = parse_sdf_properties(lines);
    Ok((atoms, bonds, properties))
}

fn parse_v3000_atom(
    payload: &str,
    line_number: usize,
    index: usize,
    record_index: usize,
) -> Result<MoleculeAtom, SdfParseError> {
    let fields = payload.split_whitespace().collect::<Vec<_>>();
    if fields.len() < 5 {
        return Err(SdfParseError::InvalidV3000Line {
            line: line_number,
            record_index,
        });
    }
    let x = fields[2]
        .parse()
        .map_err(|_| SdfParseError::InvalidV3000Line {
            line: line_number,
            record_index,
        })?;
    let y = fields[3]
        .parse()
        .map_err(|_| SdfParseError::InvalidV3000Line {
            line: line_number,
            record_index,
        })?;
    let z = fields[4]
        .parse()
        .map_err(|_| SdfParseError::InvalidV3000Line {
            line: line_number,
            record_index,
        })?;
    Ok(MoleculeAtom {
        index,
        element: fields[1].to_string(),
        token: fields[1].to_string(),
        aromatic: false,
        bracketed: false,
        isotope: None,
        explicit_hydrogens: 0,
        charge: 0,
        chirality: None,
        atom_class: None,
        coordinate: Some(MoleculeCoordinate { x, y, z }),
        atom_type: None,
        partial_charge: None,
        substructure_id: None,
        substructure_name: None,
    })
}

fn parse_v3000_bond(
    payload: &str,
    line_number: usize,
    index: usize,
    record_index: usize,
) -> Result<MoleculeBond, SdfParseError> {
    let fields = payload.split_whitespace().collect::<Vec<_>>();
    if fields.len() < 4 {
        return Err(SdfParseError::InvalidV3000Line {
            line: line_number,
            record_index,
        });
    }
    Ok(MoleculeBond {
        index,
        source_atom: parse_one_based_index(fields[2], line_number, record_index)?,
        target_atom: parse_one_based_index(fields[3], line_number, record_index)?,
        order: sdf_bond_order(fields[1], line_number, record_index)?,
        ring_closure: false,
        stereochemistry: None,
    })
}

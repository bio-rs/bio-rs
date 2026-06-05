use crate::molecule::{MoleculeAtom, MoleculeBond, MoleculeCoordinate};

use super::{
    parse_one_based_index, parse_sdf_properties, sdf_bond_order, SdfGraphParts, SdfParseError,
};

pub(super) fn parse_v2000(
    lines: &[(usize, String)],
    record_index: usize,
) -> Result<SdfGraphParts, SdfParseError> {
    let counts = lines[3].1.split_whitespace().collect::<Vec<_>>();
    let atom_count = counts
        .first()
        .and_then(|value| value.parse::<usize>().ok())
        .ok_or(SdfParseError::InvalidCountsLine {
            line: lines[3].0,
            record_index,
        })?;
    let bond_count = counts
        .get(1)
        .and_then(|value| value.parse::<usize>().ok())
        .ok_or(SdfParseError::InvalidCountsLine {
            line: lines[3].0,
            record_index,
        })?;

    let mut atoms = Vec::new();
    for offset in 0..atom_count {
        let (line_number, line) = lines
            .get(4 + offset)
            .ok_or(SdfParseError::InvalidAtomLine {
                line: lines.last().map(|(line, _)| *line).unwrap_or(lines[3].0),
                record_index,
            })?;
        atoms.push(parse_v2000_atom(
            line,
            *line_number,
            atoms.len(),
            record_index,
        )?);
    }

    let mut bonds = Vec::new();
    for offset in 0..bond_count {
        let index = 4 + atom_count + offset;
        let (line_number, line) = lines.get(index).ok_or(SdfParseError::InvalidBondLine {
            line: lines.last().map(|(line, _)| *line).unwrap_or(lines[3].0),
            record_index,
        })?;
        bonds.push(parse_v2000_bond(
            line,
            *line_number,
            bonds.len(),
            record_index,
        )?);
    }
    let properties = parse_sdf_properties(&lines[4 + atom_count + bond_count..]);
    Ok((atoms, bonds, properties))
}

fn parse_v2000_atom(
    line: &str,
    line_number: usize,
    index: usize,
    record_index: usize,
) -> Result<MoleculeAtom, SdfParseError> {
    let fields = line.split_whitespace().collect::<Vec<_>>();
    if fields.len() < 4 {
        return Err(SdfParseError::InvalidAtomLine {
            line: line_number,
            record_index,
        });
    }
    let x = fields[0]
        .parse()
        .map_err(|_| SdfParseError::InvalidAtomLine {
            line: line_number,
            record_index,
        })?;
    let y = fields[1]
        .parse()
        .map_err(|_| SdfParseError::InvalidAtomLine {
            line: line_number,
            record_index,
        })?;
    let z = fields[2]
        .parse()
        .map_err(|_| SdfParseError::InvalidAtomLine {
            line: line_number,
            record_index,
        })?;
    Ok(MoleculeAtom {
        index,
        element: fields[3].to_string(),
        token: fields[3].to_string(),
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

fn parse_v2000_bond(
    line: &str,
    line_number: usize,
    index: usize,
    record_index: usize,
) -> Result<MoleculeBond, SdfParseError> {
    let fields = line.split_whitespace().collect::<Vec<_>>();
    if fields.len() < 3 {
        return Err(SdfParseError::InvalidBondLine {
            line: line_number,
            record_index,
        });
    }
    let source_atom = parse_one_based_index(fields[0], line_number, record_index)?;
    let target_atom = parse_one_based_index(fields[1], line_number, record_index)?;
    let order = sdf_bond_order(fields[2], line_number, record_index)?;
    Ok(MoleculeBond {
        index,
        source_atom,
        target_atom,
        order,
        ring_closure: false,
        stereochemistry: None,
    })
}

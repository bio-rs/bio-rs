use crate::molecule::{MoleculeAtom, MoleculeBond, MoleculeCoordinate};

use super::{
    parse_one_based_atom_index, parse_sdf_properties, sdf_bond_order, SdfGraphParts, SdfParseError,
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
            atoms.len(),
            record_index,
        )?);
    }
    let tail = &lines[4 + atom_count + bond_count..];
    apply_v2000_charge_records(&mut atoms, tail, record_index)?;
    let properties = parse_sdf_properties(tail);
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
    let x: f64 = fields[0]
        .parse()
        .map_err(|_| SdfParseError::InvalidAtomLine {
            line: line_number,
            record_index,
        })?;
    let y: f64 = fields[1]
        .parse()
        .map_err(|_| SdfParseError::InvalidAtomLine {
            line: line_number,
            record_index,
        })?;
    let z: f64 = fields[2]
        .parse()
        .map_err(|_| SdfParseError::InvalidAtomLine {
            line: line_number,
            record_index,
        })?;
    if !x.is_finite() || !y.is_finite() || !z.is_finite() {
        return Err(SdfParseError::InvalidAtomLine {
            line: line_number,
            record_index,
        });
    }
    let charge = fields
        .get(5)
        .map(|value| v2000_charge_code(value, line_number, record_index))
        .transpose()?
        .unwrap_or(0);
    Ok(MoleculeAtom {
        index,
        element: fields[3].to_string(),
        token: fields[3].to_string(),
        aromatic: false,
        bracketed: false,
        isotope: None,
        explicit_hydrogens: 0,
        charge,
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
    atom_count: usize,
    record_index: usize,
) -> Result<MoleculeBond, SdfParseError> {
    let fields = line.split_whitespace().collect::<Vec<_>>();
    if fields.len() < 3 {
        return Err(SdfParseError::InvalidBondLine {
            line: line_number,
            record_index,
        });
    }
    let source_atom = parse_one_based_atom_index(fields[0], atom_count, line_number, record_index)?;
    let target_atom = parse_one_based_atom_index(fields[1], atom_count, line_number, record_index)?;
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

fn v2000_charge_code(
    value: &str,
    line_number: usize,
    record_index: usize,
) -> Result<i8, SdfParseError> {
    match value.parse::<u8>() {
        Ok(0) | Ok(4) => Ok(0),
        Ok(1) => Ok(3),
        Ok(2) => Ok(2),
        Ok(3) => Ok(1),
        Ok(5) => Ok(-1),
        Ok(6) => Ok(-2),
        Ok(7) => Ok(-3),
        _ => Err(SdfParseError::InvalidAtomLine {
            line: line_number,
            record_index,
        }),
    }
}

fn apply_v2000_charge_records(
    atoms: &mut [MoleculeAtom],
    lines: &[(usize, String)],
    record_index: usize,
) -> Result<(), SdfParseError> {
    for (line_number, line) in lines {
        let fields = line.split_whitespace().collect::<Vec<_>>();
        if fields.first() != Some(&"M") || fields.get(1) != Some(&"CHG") {
            continue;
        }
        let pair_count = fields
            .get(2)
            .and_then(|value| value.parse::<usize>().ok())
            .ok_or(SdfParseError::InvalidAtomLine {
                line: *line_number,
                record_index,
            })?;
        if fields.len() < 3 + pair_count * 2 {
            return Err(SdfParseError::InvalidAtomLine {
                line: *line_number,
                record_index,
            });
        }
        for pair in 0..pair_count {
            let atom_index = fields[3 + pair * 2]
                .parse::<usize>()
                .ok()
                .and_then(|index| index.checked_sub(1))
                .ok_or(SdfParseError::InvalidAtomLine {
                    line: *line_number,
                    record_index,
                })?;
            let charge =
                fields[4 + pair * 2]
                    .parse::<i8>()
                    .map_err(|_| SdfParseError::InvalidAtomLine {
                        line: *line_number,
                        record_index,
                    })?;
            let atom = atoms
                .get_mut(atom_index)
                .ok_or(SdfParseError::InvalidAtomLine {
                    line: *line_number,
                    record_index,
                })?;
            atom.charge = charge;
        }
    }
    Ok(())
}

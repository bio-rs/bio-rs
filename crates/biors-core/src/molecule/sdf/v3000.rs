use std::collections::HashMap;

use crate::molecule::{MoleculeAtom, MoleculeBond, MoleculeCoordinate};

use super::{parse_sdf_properties, sdf_bond_order, SdfGraphParts, SdfParseError};

pub(super) fn parse_v3000(
    lines: &[(usize, String)],
    record_index: usize,
) -> Result<SdfGraphParts, SdfParseError> {
    let mut atoms = Vec::new();
    let mut atom_id_to_index = HashMap::new();
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
            _ if in_atom => {
                let (source_id, atom) =
                    parse_v3000_atom(payload, *line_number, atoms.len(), record_index)?;
                if atom_id_to_index.insert(source_id, atom.index).is_some() {
                    return Err(SdfParseError::InvalidV3000Line {
                        line: *line_number,
                        record_index,
                    });
                }
                atoms.push(atom);
            }
            _ if in_bond => bonds.push(parse_v3000_bond(
                payload,
                *line_number,
                bonds.len(),
                &atom_id_to_index,
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
) -> Result<(usize, MoleculeAtom), SdfParseError> {
    let fields = payload.split_whitespace().collect::<Vec<_>>();
    if fields.len() < 5 {
        return Err(SdfParseError::InvalidV3000Line {
            line: line_number,
            record_index,
        });
    }
    let source_id = fields[0]
        .parse::<usize>()
        .map_err(|_| SdfParseError::InvalidV3000Line {
            line: line_number,
            record_index,
        })?;
    if source_id == 0 {
        return Err(SdfParseError::InvalidV3000Line {
            line: line_number,
            record_index,
        });
    }
    let x: f64 = fields[2]
        .parse()
        .map_err(|_| SdfParseError::InvalidV3000Line {
            line: line_number,
            record_index,
        })?;
    let y: f64 = fields[3]
        .parse()
        .map_err(|_| SdfParseError::InvalidV3000Line {
            line: line_number,
            record_index,
        })?;
    let z: f64 = fields[4]
        .parse()
        .map_err(|_| SdfParseError::InvalidV3000Line {
            line: line_number,
            record_index,
        })?;
    if !x.is_finite() || !y.is_finite() || !z.is_finite() {
        return Err(SdfParseError::InvalidV3000Line {
            line: line_number,
            record_index,
        });
    }
    let charge = parse_v3000_charge(&fields, line_number, record_index)?;
    Ok((
        source_id,
        MoleculeAtom {
            index,
            element: fields[1].to_string(),
            token: fields[1].to_string(),
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
        },
    ))
}

fn parse_v3000_charge(
    fields: &[&str],
    line_number: usize,
    record_index: usize,
) -> Result<i8, SdfParseError> {
    let Some(charge_token) = fields.iter().find_map(|field| field.strip_prefix("CHG=")) else {
        return Ok(0);
    };
    charge_token
        .parse::<i8>()
        .map_err(|_| SdfParseError::InvalidV3000Line {
            line: line_number,
            record_index,
        })
}

fn parse_v3000_bond(
    payload: &str,
    line_number: usize,
    index: usize,
    atom_id_to_index: &HashMap<usize, usize>,
    record_index: usize,
) -> Result<MoleculeBond, SdfParseError> {
    let fields = payload.split_whitespace().collect::<Vec<_>>();
    if fields.len() < 4 {
        return Err(SdfParseError::InvalidV3000Line {
            line: line_number,
            record_index,
        });
    }
    let source_atom =
        resolve_v3000_atom_index(fields[2], atom_id_to_index, line_number, record_index)?;
    let target_atom =
        resolve_v3000_atom_index(fields[3], atom_id_to_index, line_number, record_index)?;
    Ok(MoleculeBond {
        index,
        source_atom,
        target_atom,
        order: sdf_bond_order(fields[1], line_number, record_index)?,
        ring_closure: false,
        stereochemistry: None,
    })
}

fn resolve_v3000_atom_index(
    value: &str,
    atom_id_to_index: &HashMap<usize, usize>,
    line: usize,
    record_index: usize,
) -> Result<usize, SdfParseError> {
    let source_id = value
        .parse::<usize>()
        .map_err(|_| SdfParseError::InvalidBondLine { line, record_index })?;
    atom_id_to_index
        .get(&source_id)
        .copied()
        .ok_or(SdfParseError::InvalidBondLine { line, record_index })
}

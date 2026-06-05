use std::collections::HashMap;

use crate::formats::{BioFormat, FormatMetadata};
use crate::molecule::graph::disconnected_components;
use crate::molecule::{
    AtomGraph, BondGraph, BondOrder, MolecularGraph, MoleculeAtom, MoleculeBond,
    MoleculeCoordinate, MoleculeMetadata, MoleculeProperty, MoleculeRecord,
};

use super::Mol2ParseError;

pub(super) fn parse_mol2_record(
    lines: &[(usize, String)],
    record_index: usize,
) -> Result<MoleculeRecord, Mol2ParseError> {
    let start = lines.first().map(|(line, _)| *line).unwrap_or(1);
    let end = lines.last().map(|(line, _)| *line).unwrap_or(start);
    if !lines
        .first()
        .is_some_and(|(_, line)| line.trim().eq_ignore_ascii_case("@<TRIPOS>MOLECULE"))
    {
        return Err(Mol2ParseError::MissingMoleculeSection {
            line: start,
            record_index,
        });
    }
    let name = lines
        .get(1)
        .map(|(_, line)| line.trim().to_string())
        .filter(|line| !line.is_empty())
        .ok_or(Mol2ParseError::MissingMoleculeName {
            line: start,
            record_index,
        })?;
    let counts = lines
        .get(2)
        .map(|(_, line)| line.split_whitespace().collect::<Vec<_>>())
        .ok_or(Mol2ParseError::MissingCountsLine {
            line: start + 2,
            record_index,
        })?;
    let expected_atoms = counts
        .first()
        .and_then(|value| value.parse::<usize>().ok())
        .ok_or(Mol2ParseError::MissingCountsLine {
            line: lines.get(2).map(|(line, _)| *line).unwrap_or(start),
            record_index,
        })?;
    let expected_bonds = counts
        .get(1)
        .and_then(|value| value.parse::<usize>().ok())
        .ok_or(Mol2ParseError::MissingCountsLine {
            line: lines.get(2).map(|(line, _)| *line).unwrap_or(start),
            record_index,
        })?;

    let mut atoms = Vec::new();
    let mut atom_id_to_index = HashMap::new();
    let mut bonds = Vec::new();
    let mut properties = Vec::new();
    let mut section = "";
    for (line_number, line) in lines.iter().skip(3) {
        let trimmed = line.trim();
        if trimmed.starts_with("@<TRIPOS>") {
            section = trimmed;
            properties.push(MoleculeProperty::new(
                "mol2_section",
                trimmed.trim_start_matches("@<TRIPOS>").to_string(),
            ));
            continue;
        }
        if trimmed.is_empty() {
            continue;
        }
        match section {
            "@<TRIPOS>ATOM" => {
                let (source_id, atom) =
                    parse_atom_line(trimmed, *line_number, atoms.len(), record_index)?;
                if atom_id_to_index.insert(source_id, atom.index).is_some() {
                    return Err(Mol2ParseError::InvalidAtomLine {
                        line: *line_number,
                        record_index,
                    });
                }
                atoms.push(atom);
            }
            "@<TRIPOS>BOND" => bonds.push(parse_bond_line(
                trimmed,
                *line_number,
                bonds.len(),
                &atom_id_to_index,
                record_index,
            )?),
            "@<TRIPOS>MOLECULE" => {}
            _ => {}
        }
    }
    if atoms.len() != expected_atoms || bonds.len() != expected_bonds {
        return Err(Mol2ParseError::MissingCountsLine {
            line: lines.get(2).map(|(line, _)| *line).unwrap_or(start),
            record_index,
        });
    }
    let metadata = MoleculeMetadata {
        source: FormatMetadata::new(record_index, start, end),
        atom_count: atoms.len(),
        bond_count: bonds.len(),
        branch_count: 0,
        ring_closure_count: 0,
        disconnected_component_count: disconnected_components(atoms.len(), &bonds),
        aromatic_atom_count: atoms.iter().filter(|atom| atom.aromatic).count(),
    };
    Ok(MoleculeRecord {
        format: BioFormat::Mol2,
        id: Some(name.clone()),
        source: name,
        metadata,
        graph: MolecularGraph {
            atoms: AtomGraph { atoms },
            bonds: BondGraph { bonds },
        },
        properties,
    })
}

fn parse_atom_line(
    line: &str,
    line_number: usize,
    index: usize,
    record_index: usize,
) -> Result<(usize, MoleculeAtom), Mol2ParseError> {
    let fields = line.split_whitespace().collect::<Vec<_>>();
    if fields.len() < 6 {
        return Err(Mol2ParseError::InvalidAtomLine {
            line: line_number,
            record_index,
        });
    }
    let source_id = fields[0]
        .parse::<usize>()
        .map_err(|_| Mol2ParseError::InvalidAtomLine {
            line: line_number,
            record_index,
        })?;
    if source_id == 0 {
        return Err(Mol2ParseError::InvalidAtomLine {
            line: line_number,
            record_index,
        });
    }
    let x: f64 = fields[2]
        .parse()
        .map_err(|_| Mol2ParseError::InvalidAtomLine {
            line: line_number,
            record_index,
        })?;
    let y: f64 = fields[3]
        .parse()
        .map_err(|_| Mol2ParseError::InvalidAtomLine {
            line: line_number,
            record_index,
        })?;
    let z: f64 = fields[4]
        .parse()
        .map_err(|_| Mol2ParseError::InvalidAtomLine {
            line: line_number,
            record_index,
        })?;
    if !x.is_finite() || !y.is_finite() || !z.is_finite() {
        return Err(Mol2ParseError::InvalidAtomLine {
            line: line_number,
            record_index,
        });
    }
    let atom_type = fields[5].to_string();
    let element = element_from_mol2_type(&atom_type);
    let substructure_id = fields.get(6).and_then(|value| value.parse().ok());
    let substructure_name = fields.get(7).map(|value| (*value).to_string());
    let partial_charge = match fields.get(8) {
        Some(value) => {
            let charge = value
                .parse::<f64>()
                .map_err(|_| Mol2ParseError::InvalidAtomLine {
                    line: line_number,
                    record_index,
                })?;
            if !charge.is_finite() {
                return Err(Mol2ParseError::InvalidAtomLine {
                    line: line_number,
                    record_index,
                });
            }
            Some(charge)
        }
        None => None,
    };
    Ok((
        source_id,
        MoleculeAtom {
            index,
            element,
            token: fields[1].to_string(),
            aromatic: atom_type.to_ascii_lowercase().contains(".ar"),
            bracketed: false,
            isotope: None,
            explicit_hydrogens: 0,
            charge: 0,
            chirality: None,
            atom_class: None,
            coordinate: Some(MoleculeCoordinate { x, y, z }),
            atom_type: Some(atom_type),
            partial_charge,
            substructure_id,
            substructure_name,
        },
    ))
}

fn parse_bond_line(
    line: &str,
    line_number: usize,
    index: usize,
    atom_id_to_index: &HashMap<usize, usize>,
    record_index: usize,
) -> Result<MoleculeBond, Mol2ParseError> {
    let fields = line.split_whitespace().collect::<Vec<_>>();
    if fields.len() < 4 {
        return Err(Mol2ParseError::InvalidBondLine {
            line: line_number,
            record_index,
        });
    }
    let source_atom =
        resolve_mol2_atom_index(fields[1], atom_id_to_index, line_number, record_index)?;
    let target_atom =
        resolve_mol2_atom_index(fields[2], atom_id_to_index, line_number, record_index)?;
    Ok(MoleculeBond {
        index,
        source_atom,
        target_atom,
        order: mol2_bond_order(fields[3], line_number, record_index)?,
        ring_closure: false,
        stereochemistry: None,
    })
}

fn resolve_mol2_atom_index(
    value: &str,
    atom_id_to_index: &HashMap<usize, usize>,
    line: usize,
    record_index: usize,
) -> Result<usize, Mol2ParseError> {
    let source_id = value
        .parse::<usize>()
        .map_err(|_| Mol2ParseError::InvalidBondLine { line, record_index })?;
    atom_id_to_index
        .get(&source_id)
        .copied()
        .ok_or(Mol2ParseError::InvalidBondLine { line, record_index })
}

fn mol2_bond_order(
    value: &str,
    line: usize,
    record_index: usize,
) -> Result<BondOrder, Mol2ParseError> {
    match value.to_ascii_lowercase().as_str() {
        "1" => Ok(BondOrder::Single),
        "2" => Ok(BondOrder::Double),
        "3" => Ok(BondOrder::Triple),
        "ar" => Ok(BondOrder::Aromatic),
        "am" | "du" | "un" | "nc" => Ok(BondOrder::Single),
        other => Err(Mol2ParseError::UnsupportedBondType {
            line,
            bond_type: other.to_string(),
            record_index,
        }),
    }
}

fn element_from_mol2_type(atom_type: &str) -> String {
    let head = atom_type.split('.').next().unwrap_or(atom_type);
    let mut chars = head.chars();
    let Some(first) = chars.next() else {
        return "*".to_string();
    };
    let mut element = first.to_ascii_uppercase().to_string();
    element.extend(chars.take(1).map(|symbol| symbol.to_ascii_lowercase()));
    element
}

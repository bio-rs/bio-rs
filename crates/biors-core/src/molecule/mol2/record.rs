mod atom;
mod bond;

use std::collections::HashMap;

use crate::formats::{BioFormat, FormatMetadata};
use crate::molecule::graph::disconnected_components;
use crate::molecule::{
    AtomGraph, BondGraph, MolecularGraph, MoleculeMetadata, MoleculeProperty, MoleculeRecord,
};

use super::Mol2ParseError;
use atom::parse_atom_line;
use bond::parse_bond_line;

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

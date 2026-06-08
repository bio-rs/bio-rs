use std::collections::HashMap;

use crate::molecule::{BondOrder, MoleculeBond};

use crate::molecule::mol2::Mol2ParseError;

pub(super) fn parse_bond_line(
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

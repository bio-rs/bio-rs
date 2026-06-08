use crate::molecule::{MoleculeAtom, MoleculeCoordinate};

use crate::molecule::mol2::Mol2ParseError;

pub(super) fn parse_atom_line(
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
    let x = parse_finite_atom_f64(fields[2], line_number, record_index)?;
    let y = parse_finite_atom_f64(fields[3], line_number, record_index)?;
    let z = parse_finite_atom_f64(fields[4], line_number, record_index)?;
    let atom_type = fields[5].to_string();
    let element = element_from_mol2_type(&atom_type);
    let substructure_id = fields.get(6).and_then(|value| value.parse().ok());
    let substructure_name = fields.get(7).map(|value| (*value).to_string());
    let partial_charge = fields
        .get(8)
        .map(|value| parse_finite_atom_f64(value, line_number, record_index))
        .transpose()?;
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

fn parse_finite_atom_f64(
    value: &str,
    line: usize,
    record_index: usize,
) -> Result<f64, Mol2ParseError> {
    let parsed = value
        .parse::<f64>()
        .map_err(|_| Mol2ParseError::InvalidAtomLine { line, record_index })?;
    if parsed.is_finite() {
        Ok(parsed)
    } else {
        Err(Mol2ParseError::InvalidAtomLine { line, record_index })
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

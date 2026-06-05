//! SDF / MOLfile parser for molecule records.

use std::io::BufRead;

use crate::formats::{BioFormat, FormatMetadata};
use crate::molecule::graph::disconnected_components;
use crate::molecule::{
    AtomGraph, BondGraph, BondOrder, MolecularGraph, MoleculeAtom, MoleculeBond, MoleculeMetadata,
    MoleculeProperty, MoleculeReadError, MoleculeRecord, ParsedMoleculeInput,
};
use crate::verification::StableInputHasher;

mod error;
mod v2000;
mod v3000;

pub use error::SdfParseError;
use v2000::parse_v2000;
use v3000::parse_v3000;

type SdfGraphParts = (Vec<MoleculeAtom>, Vec<MoleculeBond>, Vec<MoleculeProperty>);

/// Parse SDF text into molecule records.
pub fn parse_sdf_records(input: &str) -> Result<Vec<MoleculeRecord>, SdfParseError> {
    parse_sdf_records_reader(input.as_bytes())
        .map(|parsed| parsed.records)
        .map_err(|error| match error {
            MoleculeReadError::SdfParse(error) => error,
            MoleculeReadError::SmilesParse(_) | MoleculeReadError::Mol2Parse(_) => {
                unreachable!("SDF parser cannot emit non-SDF parse errors")
            }
            MoleculeReadError::Io(_) => unreachable!("in-memory SDF parsing cannot fail with I/O"),
        })
}

/// Parse SDF records from a buffered reader without preloading the full input.
pub fn parse_sdf_records_reader<R: BufRead>(
    reader: R,
) -> Result<ParsedMoleculeInput, MoleculeReadError> {
    let parsed = parse_sdf_stream(reader)?;
    Ok(ParsedMoleculeInput {
        input_hash: parsed.input_hash,
        records: parsed.records,
    })
}

struct ParsedSdfStream {
    input_hash: String,
    records: Vec<MoleculeRecord>,
}

fn parse_sdf_stream<R: BufRead>(mut reader: R) -> Result<ParsedSdfStream, MoleculeReadError> {
    let mut hasher = StableInputHasher::new();
    let mut line_number = 0usize;
    let mut records = Vec::new();
    let mut current = Vec::<(usize, String)>::new();
    loop {
        let mut raw = String::new();
        let bytes = reader.read_line(&mut raw)?;
        if bytes == 0 {
            if !current.is_empty() {
                let record_index = records.len();
                records.push(parse_sdf_record(&current, record_index)?);
            }
            break;
        }
        line_number += 1;
        hasher.update(raw.as_bytes());
        let line = raw.trim_end_matches(['\n', '\r']).to_string();
        if line.trim() == "$$$$" {
            let record_index = records.len();
            records.push(parse_sdf_record(&current, record_index)?);
            current.clear();
        } else {
            current.push((line_number, line));
        }
    }
    if records.is_empty() {
        return Err(SdfParseError::EmptyInput.into());
    }
    Ok(ParsedSdfStream {
        input_hash: hasher.finalize(),
        records,
    })
}

fn parse_sdf_record(
    lines: &[(usize, String)],
    record_index: usize,
) -> Result<MoleculeRecord, SdfParseError> {
    let line_start = lines.first().map(|(line, _)| *line).unwrap_or(1);
    let line_end = lines.last().map(|(line, _)| *line).unwrap_or(line_start);
    let title = lines
        .first()
        .map(|(_, line)| line.trim().to_string())
        .filter(|line| !line.is_empty());
    if lines.len() < 4 {
        return Err(SdfParseError::MissingCountsLine {
            line: line_start,
            record_index,
        });
    }
    let counts = &lines[3].1;
    let (atoms, bonds, properties) = if counts.contains("V3000") {
        parse_v3000(lines, record_index)?
    } else {
        parse_v2000(lines, record_index)?
    };
    let metadata = MoleculeMetadata {
        source: FormatMetadata::new(record_index, line_start, line_end),
        atom_count: atoms.len(),
        bond_count: bonds.len(),
        branch_count: 0,
        ring_closure_count: 0,
        disconnected_component_count: disconnected_components(atoms.len(), &bonds),
        aromatic_atom_count: 0,
    };
    Ok(MoleculeRecord {
        format: BioFormat::Sdf,
        id: title.clone(),
        source: title.unwrap_or_else(|| format!("sdf-record-{}", record_index + 1)),
        metadata,
        graph: MolecularGraph {
            atoms: AtomGraph { atoms },
            bonds: BondGraph { bonds },
        },
        properties,
    })
}

pub(super) fn parse_sdf_properties(lines: &[(usize, String)]) -> Vec<MoleculeProperty> {
    let mut properties = Vec::new();
    let mut current_name = None::<String>;
    let mut current_value = Vec::new();
    for (_, line) in lines {
        if line.starts_with("> ") || line.starts_with("><") || line.starts_with(">") {
            flush_property(&mut properties, &mut current_name, &mut current_value);
            current_name = property_name(line);
        } else if current_name.is_some() {
            current_value.push(line.clone());
        }
    }
    flush_property(&mut properties, &mut current_name, &mut current_value);
    properties
}

fn property_name(line: &str) -> Option<String> {
    let start = line.find('<')?;
    let end = line[start + 1..].find('>')? + start + 1;
    let name = line[start + 1..end].trim();
    (!name.is_empty()).then(|| name.to_string())
}

fn flush_property(
    properties: &mut Vec<MoleculeProperty>,
    current_name: &mut Option<String>,
    current_value: &mut Vec<String>,
) {
    if let Some(name) = current_name.take() {
        properties.push(MoleculeProperty::new(name, current_value.join("\n")));
        current_value.clear();
    }
}

pub(super) fn parse_one_based_index(
    value: &str,
    line: usize,
    record_index: usize,
) -> Result<usize, SdfParseError> {
    value
        .parse::<usize>()
        .ok()
        .and_then(|index| index.checked_sub(1))
        .ok_or(SdfParseError::InvalidBondLine { line, record_index })
}

pub(super) fn sdf_bond_order(
    value: &str,
    line: usize,
    record_index: usize,
) -> Result<BondOrder, SdfParseError> {
    match value {
        "1" => Ok(BondOrder::Single),
        "2" => Ok(BondOrder::Double),
        "3" => Ok(BondOrder::Triple),
        "4" => Ok(BondOrder::Aromatic),
        other => Err(SdfParseError::UnsupportedBondType {
            line,
            bond_type: other.to_string(),
            record_index,
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}

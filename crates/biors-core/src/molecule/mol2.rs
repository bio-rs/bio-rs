//! Tripos MOL2 parser for molecule records.

use std::io::BufRead;

use crate::molecule::{MoleculeReadError, MoleculeRecord, ParsedMoleculeInput};
use crate::verification::StableInputHasher;

mod error;
mod record;

pub use error::Mol2ParseError;
use record::parse_mol2_record;

/// Parse MOL2 text into molecule records.
pub fn parse_mol2_records(input: &str) -> Result<Vec<MoleculeRecord>, Mol2ParseError> {
    parse_mol2_records_reader(input.as_bytes())
        .map(|parsed| parsed.records)
        .map_err(|error| match error {
            MoleculeReadError::Mol2Parse(error) => error,
            MoleculeReadError::SmilesParse(_) | MoleculeReadError::SdfParse(_) => {
                unreachable!("MOL2 parser cannot emit non-MOL2 parse errors")
            }
            MoleculeReadError::Io(_) => unreachable!("in-memory MOL2 parsing cannot fail with I/O"),
        })
}

/// Parse MOL2 records from a buffered reader without preloading the full input.
pub fn parse_mol2_records_reader<R: BufRead>(
    reader: R,
) -> Result<ParsedMoleculeInput, MoleculeReadError> {
    let parsed = parse_mol2_stream(reader)?;
    Ok(ParsedMoleculeInput {
        input_hash: parsed.input_hash,
        records: parsed.records,
    })
}

struct ParsedMol2Stream {
    input_hash: String,
    records: Vec<MoleculeRecord>,
}

fn parse_mol2_stream<R: BufRead>(mut reader: R) -> Result<ParsedMol2Stream, MoleculeReadError> {
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
                records.push(parse_mol2_record(&current, record_index)?);
            }
            break;
        }
        line_number += 1;
        hasher.update(raw.as_bytes());
        let line = raw.trim_end_matches(['\n', '\r']).to_string();
        if line.trim().eq_ignore_ascii_case("@<TRIPOS>MOLECULE") && !current.is_empty() {
            let record_index = records.len();
            records.push(parse_mol2_record(&current, record_index)?);
            current.clear();
        }
        current.push((line_number, line));
    }
    if records.is_empty() {
        return Err(Mol2ParseError::EmptyInput.into());
    }
    Ok(ParsedMol2Stream {
        input_hash: hasher.finalize(),
        records,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::formats::BioFormat;

    #[test]
    fn parses_mol2_atoms_bonds_and_charges() {
        let input = "\
@<TRIPOS>MOLECULE
ethanol
3 2 0 0 0
SMALL
USER_CHARGES
@<TRIPOS>ATOM
1 C1 0.000 0.000 0.000 C.3 1 ETO -0.1
2 C2 1.500 0.000 0.000 C.3 1 ETO 0.1
3 O3 2.100 1.200 0.000 O.3 1 ETO -0.2
@<TRIPOS>BOND
1 1 2 1
2 2 3 1
";
        let records = parse_mol2_records(input).expect("parse mol2");

        assert_eq!(records[0].format, BioFormat::Mol2);
        assert_eq!(records[0].metadata.atom_count, 3);
        assert_eq!(records[0].metadata.bond_count, 2);
        assert_eq!(
            records[0].graph.atoms.atoms[2].atom_type.as_deref(),
            Some("O.3")
        );
        assert_eq!(
            records[0].graph.atoms.atoms[0].substructure_name.as_deref(),
            Some("ETO")
        );
        assert_eq!(records[0].graph.atoms.atoms[0].partial_charge, Some(-0.1));
    }
}

use std::io::BufRead;

use crate::molecule::{MoleculeReadError, MoleculeRecord, ParsedMoleculeInput};
use crate::verification::StableInputHasher;

use super::parser::SmilesParser;
use crate::molecule::smiles::SmilesParseError;

/// Parse SMILES text into molecule records.
pub fn parse_smiles_records(input: &str) -> Result<Vec<MoleculeRecord>, SmilesParseError> {
    parse_smiles_records_reader(input.as_bytes())
        .map(|parsed| parsed.records)
        .map_err(|error| match error {
            MoleculeReadError::SmilesParse(error) => error,
            MoleculeReadError::SdfParse(_) | MoleculeReadError::Mol2Parse(_) => {
                unreachable!("SMILES parser cannot emit non-SMILES parse errors")
            }
            MoleculeReadError::Io(_) => {
                unreachable!("in-memory SMILES parsing cannot fail with I/O")
            }
        })
}

/// Parse SMILES records from a buffered reader without preloading the full input.
pub fn parse_smiles_records_reader<R: BufRead>(
    reader: R,
) -> Result<ParsedMoleculeInput, MoleculeReadError> {
    let parsed = parse_smiles_stream(reader)?;
    Ok(ParsedMoleculeInput {
        input_hash: parsed.input_hash,
        records: parsed.records,
    })
}

pub(crate) struct ParsedSmilesStream {
    pub(crate) input_hash: String,
    pub(crate) records: Vec<MoleculeRecord>,
}

pub(crate) fn parse_smiles_stream<R: BufRead>(
    mut reader: R,
) -> Result<ParsedSmilesStream, MoleculeReadError> {
    let mut hasher = StableInputHasher::new();
    let mut cursor = LineCursor::default();
    let mut records = Vec::new();

    while let Some((line_number, line)) = cursor.read_line(&mut reader, &mut hasher)? {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let record_index = records.len();
        let mut fields = trimmed.split_whitespace();
        let smiles = fields.next().ok_or(SmilesParseError::MissingSmiles {
            line: line_number,
            record_index,
        })?;
        let id = fields.next().map(str::to_string);
        records.push(SmilesParser::new(smiles, id, line_number, record_index).parse()?);
    }

    if records.is_empty() {
        return Err(SmilesParseError::EmptyInput.into());
    }

    Ok(ParsedSmilesStream {
        input_hash: hasher.finalize(),
        records,
    })
}

#[derive(Default)]
struct LineCursor {
    line_number: usize,
}

impl LineCursor {
    fn read_line<R: BufRead>(
        &mut self,
        reader: &mut R,
        hasher: &mut StableInputHasher,
    ) -> Result<Option<(usize, String)>, MoleculeReadError> {
        let mut raw = String::new();
        let bytes = reader.read_line(&mut raw)?;
        if bytes == 0 {
            return Ok(None);
        }
        self.line_number += 1;
        hasher.update(raw.as_bytes());
        Ok(Some((
            self.line_number,
            raw.trim_end_matches(['\n', '\r']).to_string(),
        )))
    }
}

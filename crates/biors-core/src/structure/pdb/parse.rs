use std::collections::BTreeMap;
use std::io::BufRead;

use crate::structure::{Atom, Coordinate, MissingResidue, ParsedStructureInput, StructureRecord};
use crate::verification::StableInputHasher;

use super::builder::{ParsedAtom, StructureBuilder};
use super::error::{PdbParseError, StructureReadError};

/// Parse PDB text into a structure record.
pub fn parse_pdb_record(input: &str) -> Result<StructureRecord, PdbParseError> {
    parse_pdb_record_reader(input.as_bytes())
        .map(|parsed| parsed.record)
        .map_err(|error| match error {
            StructureReadError::PdbParse(error) => error,
            StructureReadError::Io(_) => unreachable!("in-memory PDB parsing cannot fail with I/O"),
        })
}

/// Parse a PDB record from a buffered reader without preloading the full input.
pub fn parse_pdb_record_reader<R: BufRead>(
    reader: R,
) -> Result<ParsedStructureInput, StructureReadError> {
    let parsed = parse_pdb_stream(reader)?;
    Ok(ParsedStructureInput {
        input_hash: parsed.input_hash,
        record: parsed.record,
    })
}

pub(super) struct ParsedPdbStream {
    pub input_hash: String,
    pub record: StructureRecord,
}

pub(super) fn parse_pdb_stream<R: BufRead>(
    mut reader: R,
) -> Result<ParsedPdbStream, StructureReadError> {
    let mut hasher = StableInputHasher::new();
    let mut cursor = LineCursor::default();
    let mut builder = StructureBuilder::default();
    let mut model_count = 0usize;
    let mut active_model = true;

    while let Some((line_number, line)) = cursor.read_line(&mut reader, &mut hasher)? {
        builder.line_count = line_number;
        match record_name(&line) {
            "HEADER" => builder.pdb_id = pdb_id_from_header(&line),
            "TITLE" => push_title_line(&mut builder.title_lines, &line),
            "SEQRES" => parse_seqres_line(&mut builder.seqres, &line),
            "REMARK" if line.starts_with("REMARK 465") => {
                if let Some(missing) = parse_missing_residue(&line) {
                    builder.missing_residues.push(missing);
                }
            }
            "MODEL" => {
                model_count += 1;
                active_model = model_count == 1;
            }
            "ENDMDL" => active_model = false,
            "ATOM" | "HETATM" if active_model => {
                let hetero = record_name(&line) == "HETATM";
                let row = parse_atom_line(&line, line_number, hetero)?;
                builder.add_atom(row);
            }
            _ => {}
        }
    }

    if cursor.line_number == 0 {
        return Err(PdbParseError::EmptyInput.into());
    }
    builder.model_count = model_count.max(1);
    Ok(ParsedPdbStream {
        input_hash: hasher.finalize(),
        record: builder.finish(),
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
    ) -> Result<Option<(usize, String)>, StructureReadError> {
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

fn parse_atom_line(
    line: &str,
    line_number: usize,
    hetero: bool,
) -> Result<ParsedAtom, PdbParseError> {
    let serial = required_i32(line, 6, 11, "serial", line_number)?;
    let name = required_text(line, 12, 16, "name", line_number)?;
    let residue_name = required_text(line, 17, 20, "resName", line_number)?.to_ascii_uppercase();
    let chain_id = normalize_chain_id(field(line, 21, 22));
    let sequence_number = required_i32(line, 22, 26, "resSeq", line_number)?;
    let insertion_code = optional_char(field(line, 26, 27));
    let x = required_f64(line, 30, 38, "x", line_number)?;
    let y = required_f64(line, 38, 46, "y", line_number)?;
    let z = required_f64(line, 46, 54, "z", line_number)?;
    let occupancy = optional_f64(line, 54, 60, "occupancy", line_number)?;
    let temperature_factor = optional_f64(line, 60, 66, "tempFactor", line_number)?;
    let element = optional_text(field(line, 76, 78)).map(str::to_ascii_uppercase);
    Ok(ParsedAtom {
        chain_id,
        residue_name,
        sequence_number,
        insertion_code,
        hetero,
        atom: Atom {
            serial,
            name,
            alternate_location: optional_char(field(line, 16, 17)),
            element,
            coordinate: Coordinate { x, y, z },
            occupancy,
            temperature_factor,
        },
    })
}

fn parse_seqres_line(seqres: &mut BTreeMap<String, Vec<String>>, line: &str) {
    let chain_id = normalize_chain_id(field(line, 11, 12));
    let residues = seqres.entry(chain_id).or_default();
    residues.extend(
        field(line, 19, 70)
            .split_whitespace()
            .map(str::to_ascii_uppercase),
    );
}

fn parse_missing_residue(line: &str) -> Option<MissingResidue> {
    let tokens: Vec<_> = field(line, 10, line.len()).split_whitespace().collect();
    if tokens.len() < 3 || tokens.iter().any(|token| token.chars().any(|c| c == '=')) {
        return None;
    }
    let skip_words = ["MISSING", "THE", "EXPERIMENT", "MODELS", "RES", "M"];
    if skip_words
        .iter()
        .any(|word| tokens[0].eq_ignore_ascii_case(word))
    {
        return None;
    }
    let offset = usize::from(tokens.len() >= 4 && tokens[0].parse::<i32>().is_ok());
    if tokens.len() <= offset + 2 || tokens[offset].len() != 3 {
        return None;
    }
    let (sequence_number, insertion_code) = parse_sequence_token(tokens[offset + 2])?;
    Some(MissingResidue {
        name: tokens[offset].to_ascii_uppercase(),
        chain_id: normalize_chain_id(tokens[offset + 1]),
        sequence_number,
        insertion_code,
    })
}

fn parse_sequence_token(value: &str) -> Option<(i32, Option<char>)> {
    let split_at = value
        .char_indices()
        .take_while(|(_, c)| c.is_ascii_digit() || *c == '-')
        .last()
        .map(|(idx, c)| idx + c.len_utf8())
        .unwrap_or(0);
    if split_at == 0 {
        return None;
    }
    let number = value[..split_at].parse().ok()?;
    let insertion_code = value[split_at..].chars().find(|c| !c.is_whitespace());
    Some((number, insertion_code))
}

fn required_text(
    line: &str,
    start: usize,
    end: usize,
    field_name: &'static str,
    line_number: usize,
) -> Result<String, PdbParseError> {
    optional_text(field(line, start, end))
        .map(str::to_string)
        .ok_or(PdbParseError::MissingAtomField {
            field: field_name,
            line: line_number,
        })
}

fn required_i32(
    line: &str,
    start: usize,
    end: usize,
    field_name: &'static str,
    line_number: usize,
) -> Result<i32, PdbParseError> {
    let value = required_text(line, start, end, field_name, line_number)?;
    value
        .parse()
        .map_err(|_| invalid_field(field_name, &value, line_number))
}

fn required_f64(
    line: &str,
    start: usize,
    end: usize,
    field_name: &'static str,
    line_number: usize,
) -> Result<f64, PdbParseError> {
    let value = required_text(line, start, end, field_name, line_number)?;
    parse_finite_f64(field_name, &value, line_number)
}

fn optional_f64(
    line: &str,
    start: usize,
    end: usize,
    field_name: &'static str,
    line_number: usize,
) -> Result<Option<f64>, PdbParseError> {
    optional_text(field(line, start, end))
        .map(|value| parse_finite_f64(field_name, value, line_number))
        .transpose()
}

fn parse_finite_f64(
    field_name: &'static str,
    value: &str,
    line_number: usize,
) -> Result<f64, PdbParseError> {
    let parsed: f64 = value
        .parse()
        .map_err(|_| invalid_field(field_name, value, line_number))?;
    if parsed.is_finite() {
        Ok(parsed)
    } else {
        Err(invalid_field(field_name, value, line_number))
    }
}

fn invalid_field(field: &'static str, value: &str, line: usize) -> PdbParseError {
    PdbParseError::InvalidAtomField {
        field,
        value: value.to_string(),
        line,
    }
}

fn field(line: &str, start: usize, end: usize) -> &str {
    line.get(start..end.min(line.len())).unwrap_or("")
}

fn optional_text(value: &str) -> Option<&str> {
    let trimmed = value.trim();
    (!trimmed.is_empty()).then_some(trimmed)
}

fn optional_char(value: &str) -> Option<char> {
    optional_text(value).and_then(|value| value.chars().next())
}

fn record_name(line: &str) -> &str {
    field(line, 0, 6).trim()
}

fn normalize_chain_id(value: &str) -> String {
    optional_text(value).unwrap_or("_").to_string()
}

fn pdb_id_from_header(line: &str) -> Option<String> {
    optional_text(field(line, 62, 66)).map(str::to_ascii_uppercase)
}

fn push_title_line(title_lines: &mut Vec<String>, line: &str) {
    if let Some(title) = optional_text(field(line, 10, line.len())) {
        title_lines.push(title.to_string());
    }
}

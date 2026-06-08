use crate::structure::PdbParseError;

pub(super) fn required_text(
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

pub(super) fn required_i32(
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

pub(super) fn required_f64(
    line: &str,
    start: usize,
    end: usize,
    field_name: &'static str,
    line_number: usize,
) -> Result<f64, PdbParseError> {
    let value = required_text(line, start, end, field_name, line_number)?;
    parse_finite_f64(field_name, &value, line_number)
}

pub(super) fn parse_finite_f64(
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

pub(super) fn field(line: &str, start: usize, end: usize) -> &str {
    line.get(start..end.min(line.len())).unwrap_or("")
}

pub(super) fn optional_text(value: &str) -> Option<&str> {
    let trimmed = value.trim();
    (!trimmed.is_empty()).then_some(trimmed)
}

pub(super) fn optional_char(value: &str) -> Option<char> {
    optional_text(value).and_then(|value| value.chars().next())
}

pub(super) fn normalize_chain_id(value: &str) -> String {
    optional_text(value).unwrap_or("_").to_string()
}

fn invalid_field(field: &'static str, value: &str, line: usize) -> PdbParseError {
    PdbParseError::InvalidAtomField {
        field,
        value: value.to_string(),
        line,
    }
}

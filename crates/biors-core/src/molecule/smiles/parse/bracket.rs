use super::syntax::{is_aromatic_token, is_supported_element, normalize_element};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct BracketAtom {
    pub element: String,
    pub aromatic: bool,
    pub isotope: Option<u16>,
    pub explicit_hydrogens: u8,
    pub charge: i8,
    pub chirality: Option<String>,
    pub atom_class: Option<u32>,
}

pub(super) fn parse_bracket_payload(payload: &str) -> Result<BracketAtom, &'static str> {
    if payload.is_empty() {
        return Err("empty bracket atom");
    }
    let chars: Vec<_> = payload.char_indices().collect();
    let mut position = 0usize;
    let mut isotope_digits = String::new();
    while position < chars.len() && chars[position].1.is_ascii_digit() {
        isotope_digits.push(chars[position].1);
        position += 1;
    }
    let isotope = if isotope_digits.is_empty() {
        None
    } else {
        Some(
            isotope_digits
                .parse()
                .map_err(|_| "invalid isotope mass number")?,
        )
    };
    let (symbol, next_position) =
        parse_bracket_symbol(payload, &chars, position).ok_or("missing bracket atom symbol")?;
    let element = normalize_element(symbol);
    if !is_supported_element(&element) {
        return Err("unsupported bracket atom element");
    }
    position = next_position;

    let mut chirality = None;
    let mut explicit_hydrogens = 0u8;
    let mut charge = 0i8;
    let mut atom_class = None;
    while position < chars.len() {
        match chars[position].1 {
            '@' => {
                let start = chars[position].0;
                position += 1;
                if position < chars.len() && chars[position].1 == '@' {
                    position += 1;
                }
                let end = chars
                    .get(position)
                    .map(|(byte, _)| *byte)
                    .unwrap_or(payload.len());
                chirality = Some(payload[start..end].to_string());
            }
            'H' => {
                position += 1;
                let mut digits = String::new();
                while position < chars.len() && chars[position].1.is_ascii_digit() {
                    digits.push(chars[position].1);
                    position += 1;
                }
                explicit_hydrogens = if digits.is_empty() {
                    1
                } else {
                    digits.parse().map_err(|_| "invalid hydrogen count")?
                };
            }
            '+' | '-' => {
                let sign = if chars[position].1 == '+' { 1i8 } else { -1i8 };
                position += 1;
                let mut repeated = 1i8;
                while position < chars.len()
                    && ((sign > 0 && chars[position].1 == '+')
                        || (sign < 0 && chars[position].1 == '-'))
                {
                    repeated += 1;
                    position += 1;
                }
                let mut digits = String::new();
                while position < chars.len() && chars[position].1.is_ascii_digit() {
                    digits.push(chars[position].1);
                    position += 1;
                }
                let magnitude = if digits.is_empty() {
                    repeated
                } else {
                    digits.parse().map_err(|_| "invalid charge magnitude")?
                };
                charge = charge.saturating_add(sign.saturating_mul(magnitude));
            }
            ':' => {
                position += 1;
                let mut digits = String::new();
                while position < chars.len() && chars[position].1.is_ascii_digit() {
                    digits.push(chars[position].1);
                    position += 1;
                }
                atom_class = Some(digits.parse().map_err(|_| "invalid atom class")?);
            }
            _ => return Err("unsupported bracket atom modifier"),
        }
    }
    Ok(BracketAtom {
        element,
        aromatic: is_aromatic_token(symbol),
        isotope,
        explicit_hydrogens,
        charge,
        chirality,
        atom_class,
    })
}

fn parse_bracket_symbol<'a>(
    payload: &'a str,
    chars: &[(usize, char)],
    position: usize,
) -> Option<(&'a str, usize)> {
    let (_, first) = chars.get(position)?;
    if *first == '*' {
        return Some(("*", position + 1));
    }
    let start = chars[position].0;
    let mut end_position = position + 1;
    if end_position < chars.len() && chars[end_position].1.is_ascii_lowercase() {
        end_position += 1;
    }
    let end = chars
        .get(end_position)
        .map(|(byte, _)| *byte)
        .unwrap_or_else(|| {
            chars
                .last()
                .map(|(byte, ch)| byte + ch.len_utf8())
                .unwrap_or(start)
        });
    Some((&payload[start..end], end_position))
}

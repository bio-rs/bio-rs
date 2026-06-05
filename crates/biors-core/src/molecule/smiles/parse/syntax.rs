use crate::molecule::BondOrder;

pub(super) fn bond_order_from_symbol(symbol: char) -> Option<BondOrder> {
    match symbol {
        '-' | '/' | '\\' => Some(BondOrder::Single),
        '=' => Some(BondOrder::Double),
        '#' => Some(BondOrder::Triple),
        '$' => Some(BondOrder::Quadruple),
        ':' => Some(BondOrder::Aromatic),
        _ => None,
    }
}

pub(super) fn bond_stereochemistry_from_symbol(symbol: char) -> Option<String> {
    match symbol {
        '/' | '\\' => Some(symbol.to_string()),
        _ => None,
    }
}

pub(super) fn parse_percent_ring(chars: &[(usize, char)], position: usize) -> Option<(u16, usize)> {
    let first = chars.get(position + 1)?.1;
    let second = chars.get(position + 2)?.1;
    if !first.is_ascii_digit() || !second.is_ascii_digit() {
        return None;
    }
    let ring = first.to_digit(10)? as u16 * 10 + second.to_digit(10)? as u16;
    Some((ring, position + 3))
}

pub(super) fn normalize_element(token: &str) -> String {
    if token == "*" {
        return "*".to_string();
    }
    match token {
        "se" => "Se".to_string(),
        "as" => "As".to_string(),
        _ => {
            let mut chars = token.chars();
            let Some(first) = chars.next() else {
                return token.to_string();
            };
            let mut element = first.to_ascii_uppercase().to_string();
            element.extend(chars.map(|character| character.to_ascii_lowercase()));
            element
        }
    }
}

pub(super) fn is_aromatic_token(token: &str) -> bool {
    token
        .chars()
        .next()
        .is_some_and(|character| character.is_ascii_lowercase())
}

pub(super) fn is_supported_element(element: &str) -> bool {
    matches!(
        element,
        "*" | "B"
            | "C"
            | "N"
            | "O"
            | "P"
            | "S"
            | "F"
            | "Cl"
            | "Br"
            | "I"
            | "Se"
            | "As"
            | "Si"
            | "Na"
            | "K"
            | "Li"
            | "Mg"
            | "Ca"
            | "Zn"
            | "Fe"
            | "Cu"
            | "Mn"
            | "Co"
            | "Ni"
            | "H"
    )
}

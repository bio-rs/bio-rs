use crate::molecule::MoleculeAtom;

use super::bracket::parse_bracket_payload;
use super::parser::SmilesParser;
use super::syntax::{is_aromatic_token, normalize_element};
use crate::molecule::smiles::SmilesParseError;

impl SmilesParser<'_> {
    pub(super) fn parse_organic_atom(
        &self,
        chars: &[(usize, char)],
        position: usize,
    ) -> Option<(MoleculeAtom, usize)> {
        let (byte_index, symbol) = chars[position];
        let remaining = &self.smiles[byte_index..];
        for token in ["Cl", "Br", "se", "as"] {
            if remaining.starts_with(token) {
                let element = normalize_element(token);
                return Some((
                    self.atom(
                        token,
                        &element,
                        is_aromatic_token(token),
                        false,
                        None,
                        0,
                        0,
                        None,
                        None,
                    ),
                    position + token.chars().count(),
                ));
            }
        }
        let allowed = [
            "B", "C", "N", "O", "P", "S", "F", "I", "b", "c", "n", "o", "p", "s",
        ];
        let token = symbol.to_string();
        if allowed.contains(&token.as_str()) {
            let element = normalize_element(&token);
            return Some((
                self.atom(
                    &token,
                    &element,
                    is_aromatic_token(&token),
                    false,
                    None,
                    0,
                    0,
                    None,
                    None,
                ),
                position + 1,
            ));
        }
        None
    }

    pub(super) fn parse_bracket_atom(
        &self,
        chars: &[(usize, char)],
        position: usize,
    ) -> Result<(MoleculeAtom, usize), SmilesParseError> {
        let start_column = chars[position].0 + 1;
        let mut end_position = position + 1;
        while end_position < chars.len() && chars[end_position].1 != ']' {
            end_position += 1;
        }
        if end_position >= chars.len() {
            return Err(self.invalid_bracket_atom(start_column, "[", "missing closing ']'"));
        }
        let start = chars[position].0 + 1;
        let end = chars[end_position].0;
        let payload = &self.smiles[start..end];
        let token = &self.smiles[chars[position].0..=chars[end_position].0];
        let parsed = parse_bracket_payload(payload)
            .map_err(|reason| self.invalid_bracket_atom(start_column, token, reason))?;
        Ok((
            self.atom(
                token,
                &parsed.element,
                parsed.aromatic,
                true,
                parsed.isotope,
                parsed.explicit_hydrogens,
                parsed.charge,
                parsed.chirality,
                parsed.atom_class,
            ),
            end_position + 1,
        ))
    }

    #[allow(clippy::too_many_arguments)]
    pub(super) fn atom(
        &self,
        token: &str,
        element: &str,
        aromatic: bool,
        bracketed: bool,
        isotope: Option<u16>,
        explicit_hydrogens: u8,
        charge: i8,
        chirality: Option<String>,
        atom_class: Option<u32>,
    ) -> MoleculeAtom {
        MoleculeAtom {
            index: self.atoms.len(),
            element: element.to_string(),
            token: token.to_string(),
            aromatic,
            bracketed,
            isotope,
            explicit_hydrogens,
            charge,
            chirality,
            atom_class,
            coordinate: None,
            atom_type: None,
            partial_charge: None,
            substructure_id: None,
            substructure_name: None,
        }
    }
}

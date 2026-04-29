use super::{PROTEIN_20_UNKNOWN_TOKEN_ID, TOKEN_LOOKUP_MISSING};
use crate::sequence::{is_ambiguous_residue, ResidueIssue};

pub(super) fn push_tokenized_residue(
    residue: char,
    position: usize,
    tokens: &mut Vec<u8>,
    warnings: &mut Vec<ResidueIssue>,
    errors: &mut Vec<ResidueIssue>,
) {
    if let Some(token) = protein_20_token_id(residue) {
        tokens.push(token);
    } else if is_ambiguous_residue(residue) {
        tokens.push(PROTEIN_20_UNKNOWN_TOKEN_ID);
        warnings.push(ResidueIssue { residue, position });
    } else {
        tokens.push(PROTEIN_20_UNKNOWN_TOKEN_ID);
        errors.push(ResidueIssue { residue, position });
    }
}

pub(super) fn push_tokenized_residue_byte(
    residue: u8,
    position: usize,
    tokens: &mut Vec<u8>,
    warnings: &mut Vec<ResidueIssue>,
    errors: &mut Vec<ResidueIssue>,
) {
    if let Some(token) = protein_20_token_id_byte(residue) {
        tokens.push(token);
    } else if is_ambiguous_residue_byte(residue) {
        tokens.push(PROTEIN_20_UNKNOWN_TOKEN_ID);
        warnings.push(ResidueIssue {
            residue: residue.to_ascii_uppercase() as char,
            position,
        });
    } else {
        tokens.push(PROTEIN_20_UNKNOWN_TOKEN_ID);
        errors.push(ResidueIssue {
            residue: residue.to_ascii_uppercase() as char,
            position,
        });
    }
}

pub(super) fn protein_20_token_id(residue: char) -> Option<u8> {
    if residue.is_ascii() {
        return protein_20_token_id_byte(residue as u8);
    }

    match residue {
        'A' => Some(0),
        'C' => Some(1),
        'D' => Some(2),
        'E' => Some(3),
        'F' => Some(4),
        'G' => Some(5),
        'H' => Some(6),
        'I' => Some(7),
        'K' => Some(8),
        'L' => Some(9),
        'M' => Some(10),
        'N' => Some(11),
        'P' => Some(12),
        'Q' => Some(13),
        'R' => Some(14),
        'S' => Some(15),
        'T' => Some(16),
        'V' => Some(17),
        'W' => Some(18),
        'Y' => Some(19),
        _ => None,
    }
}

pub(super) fn protein_20_token_id_byte(residue: u8) -> Option<u8> {
    let token = PROTEIN_20_TOKEN_LOOKUP[residue as usize];
    if token == TOKEN_LOOKUP_MISSING {
        None
    } else {
        Some(token)
    }
}

pub(super) fn is_ambiguous_residue_byte(residue: u8) -> bool {
    AMBIGUOUS_RESIDUE_LOOKUP[residue as usize]
}

const PROTEIN_20_TOKEN_LOOKUP: [u8; 256] = {
    let mut lookup = [TOKEN_LOOKUP_MISSING; 256];
    lookup[b'A' as usize] = 0;
    lookup[b'C' as usize] = 1;
    lookup[b'D' as usize] = 2;
    lookup[b'E' as usize] = 3;
    lookup[b'F' as usize] = 4;
    lookup[b'G' as usize] = 5;
    lookup[b'H' as usize] = 6;
    lookup[b'I' as usize] = 7;
    lookup[b'K' as usize] = 8;
    lookup[b'L' as usize] = 9;
    lookup[b'M' as usize] = 10;
    lookup[b'N' as usize] = 11;
    lookup[b'P' as usize] = 12;
    lookup[b'Q' as usize] = 13;
    lookup[b'R' as usize] = 14;
    lookup[b'S' as usize] = 15;
    lookup[b'T' as usize] = 16;
    lookup[b'V' as usize] = 17;
    lookup[b'W' as usize] = 18;
    lookup[b'Y' as usize] = 19;
    lookup[b'a' as usize] = 0;
    lookup[b'c' as usize] = 1;
    lookup[b'd' as usize] = 2;
    lookup[b'e' as usize] = 3;
    lookup[b'f' as usize] = 4;
    lookup[b'g' as usize] = 5;
    lookup[b'h' as usize] = 6;
    lookup[b'i' as usize] = 7;
    lookup[b'k' as usize] = 8;
    lookup[b'l' as usize] = 9;
    lookup[b'm' as usize] = 10;
    lookup[b'n' as usize] = 11;
    lookup[b'p' as usize] = 12;
    lookup[b'q' as usize] = 13;
    lookup[b'r' as usize] = 14;
    lookup[b's' as usize] = 15;
    lookup[b't' as usize] = 16;
    lookup[b'v' as usize] = 17;
    lookup[b'w' as usize] = 18;
    lookup[b'y' as usize] = 19;
    lookup
};

const AMBIGUOUS_RESIDUE_LOOKUP: [bool; 256] = {
    let mut lookup = [false; 256];
    lookup[b'X' as usize] = true;
    lookup[b'B' as usize] = true;
    lookup[b'Z' as usize] = true;
    lookup[b'J' as usize] = true;
    lookup[b'U' as usize] = true;
    lookup[b'O' as usize] = true;
    lookup[b'x' as usize] = true;
    lookup[b'b' as usize] = true;
    lookup[b'z' as usize] = true;
    lookup[b'j' as usize] = true;
    lookup[b'u' as usize] = true;
    lookup[b'o' as usize] = true;
    lookup
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn protein_20_token_id_matches_vocab_order() {
        for (expected, residue) in crate::sequence::PROTEIN_20_RESIDUES.iter().enumerate() {
            assert_eq!(protein_20_token_id(*residue), Some(expected as u8));
            assert_eq!(
                protein_20_token_id_byte(*residue as u8),
                Some(expected as u8)
            );
            assert_eq!(
                protein_20_token_id_byte((*residue as u8).to_ascii_lowercase()),
                Some(expected as u8)
            );
        }

        assert_eq!(protein_20_token_id('X'), None);
        assert_eq!(protein_20_token_id_byte(b'X'), None);
        assert_eq!(protein_20_token_id('*'), None);
        assert_eq!(protein_20_token_id_byte(b'*'), None);
    }

    #[test]
    fn ambiguous_residue_lookup_matches_policy_residues() {
        for residue in crate::sequence::AMBIGUOUS_RESIDUES {
            assert!(is_ambiguous_residue(residue));
        }

        assert!(!is_ambiguous_residue('A'));
        assert!(!is_ambiguous_residue('*'));
    }
}

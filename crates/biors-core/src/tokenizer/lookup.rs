use super::{
    ProteinTokenizerProfile, NUCLEOTIDE_UNKNOWN_TOKEN_ID, PROTEIN_20_UNKNOWN_TOKEN_ID,
    TOKEN_LOOKUP_MISSING,
};
use crate::sequence::{AlphabetPolicy, ResidueIssue, SequenceKind, SymbolClass};

pub(super) fn push_tokenized_residue(
    profile: ProteinTokenizerProfile,
    residue: char,
    position: usize,
    tokens: &mut Vec<u8>,
    warnings: &mut Vec<ResidueIssue>,
    errors: &mut Vec<ResidueIssue>,
) {
    if let Some(token) = profile_token_id(profile, residue) {
        tokens.push(token);
    } else if is_ambiguous_for_profile(profile, residue) {
        tokens.push(profile_unknown_token_id(profile));
        warnings.push(ResidueIssue { residue, position });
    } else {
        tokens.push(profile_unknown_token_id(profile));
        errors.push(ResidueIssue { residue, position });
    }
}

pub(super) fn push_tokenized_residue_byte(
    profile: ProteinTokenizerProfile,
    residue: u8,
    position: usize,
    tokens: &mut Vec<u8>,
    warnings: &mut Vec<ResidueIssue>,
    errors: &mut Vec<ResidueIssue>,
) {
    if let Some(token) = profile_token_id_byte(profile, residue) {
        tokens.push(token);
    } else if is_ambiguous_byte_for_profile(profile, residue) {
        tokens.push(profile_unknown_token_id(profile));
        warnings.push(ResidueIssue {
            residue: residue.to_ascii_uppercase() as char,
            position,
        });
    } else {
        tokens.push(profile_unknown_token_id(profile));
        errors.push(ResidueIssue {
            residue: residue.to_ascii_uppercase() as char,
            position,
        });
    }
}

pub(super) fn profile_unknown_token_id(profile: ProteinTokenizerProfile) -> u8 {
    match profile.sequence_kind() {
        SequenceKind::Protein => PROTEIN_20_UNKNOWN_TOKEN_ID,
        SequenceKind::Dna | SequenceKind::Rna => NUCLEOTIDE_UNKNOWN_TOKEN_ID,
    }
}

pub(super) fn profile_token_id(profile: ProteinTokenizerProfile, residue: char) -> Option<u8> {
    if residue.is_ascii() {
        return profile_token_id_byte(profile, residue as u8);
    }

    None
}

pub(super) fn profile_token_id_byte(profile: ProteinTokenizerProfile, residue: u8) -> Option<u8> {
    match profile.sequence_kind() {
        SequenceKind::Protein => protein_20_token_id_byte(residue),
        SequenceKind::Dna => dna_iupac_token_id_byte(residue),
        SequenceKind::Rna => rna_iupac_token_id_byte(residue),
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

pub(super) fn is_ambiguous_for_profile(profile: ProteinTokenizerProfile, residue: char) -> bool {
    AlphabetPolicy::for_kind(profile.sequence_kind()).classify(residue) == SymbolClass::Ambiguous
}

pub(super) fn is_ambiguous_byte_for_profile(profile: ProteinTokenizerProfile, residue: u8) -> bool {
    AlphabetPolicy::for_kind(profile.sequence_kind()).classify_byte(residue)
        == SymbolClass::Ambiguous
}

fn dna_iupac_token_id_byte(residue: u8) -> Option<u8> {
    let token = DNA_IUPAC_TOKEN_LOOKUP[residue as usize];
    if token == TOKEN_LOOKUP_MISSING {
        None
    } else {
        Some(token)
    }
}

fn rna_iupac_token_id_byte(residue: u8) -> Option<u8> {
    let token = RNA_IUPAC_TOKEN_LOOKUP[residue as usize];
    if token == TOKEN_LOOKUP_MISSING {
        None
    } else {
        Some(token)
    }
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

const DNA_IUPAC_TOKEN_LOOKUP: [u8; 256] = {
    let mut lookup = [TOKEN_LOOKUP_MISSING; 256];
    lookup[b'A' as usize] = 0;
    lookup[b'C' as usize] = 1;
    lookup[b'G' as usize] = 2;
    lookup[b'T' as usize] = 3;
    lookup[b'a' as usize] = 0;
    lookup[b'c' as usize] = 1;
    lookup[b'g' as usize] = 2;
    lookup[b't' as usize] = 3;
    lookup
};

const RNA_IUPAC_TOKEN_LOOKUP: [u8; 256] = {
    let mut lookup = [TOKEN_LOOKUP_MISSING; 256];
    lookup[b'A' as usize] = 0;
    lookup[b'C' as usize] = 1;
    lookup[b'G' as usize] = 2;
    lookup[b'U' as usize] = 3;
    lookup[b'a' as usize] = 0;
    lookup[b'c' as usize] = 1;
    lookup[b'g' as usize] = 2;
    lookup[b'u' as usize] = 3;
    lookup
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn protein_20_token_id_matches_vocab_order() {
        for (expected, residue) in crate::sequence::PROTEIN_20_RESIDUES.iter().enumerate() {
            assert_eq!(
                protein_20_token_id_byte(*residue as u8),
                Some(expected as u8)
            );
            assert_eq!(
                protein_20_token_id_byte((*residue as u8).to_ascii_lowercase()),
                Some(expected as u8)
            );
        }

        assert_eq!(protein_20_token_id_byte(b'X'), None);
        assert_eq!(protein_20_token_id_byte(b'*'), None);
    }

    #[test]
    fn ambiguous_residue_lookup_matches_policy_residues() {
        for residue in crate::sequence::AMBIGUOUS_RESIDUES {
            assert!(is_ambiguous_for_profile(
                ProteinTokenizerProfile::Protein20,
                residue
            ));
        }

        assert!(is_ambiguous_for_profile(
            ProteinTokenizerProfile::DnaIupac,
            'N'
        ));
        assert!(is_ambiguous_for_profile(
            ProteinTokenizerProfile::RnaIupac,
            'N'
        ));
        assert!(!is_ambiguous_for_profile(
            ProteinTokenizerProfile::Protein20,
            'A'
        ));
        assert!(!is_ambiguous_for_profile(
            ProteinTokenizerProfile::DnaIupac,
            '*'
        ));
    }
}

/// Name of the strict 20-residue protein alphabet.
pub const PROTEIN_20: &str = "protein-20";

/// Residues accepted without warnings under the `protein-20` policy.
pub const PROTEIN_20_RESIDUES: [char; 20] = [
    'A', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'K', 'L', 'M', 'N', 'P', 'Q', 'R', 'S', 'T', 'V', 'W',
    'Y',
];

/// Ambiguous residues accepted with warnings under the current policy.
pub const AMBIGUOUS_RESIDUES: [char; 6] = ['X', 'B', 'Z', 'J', 'U', 'O'];

pub(crate) fn is_protein_20_residue(residue: char) -> bool {
    if residue.is_ascii() {
        return PROTEIN_20_RESIDUE_LOOKUP[residue as usize];
    }

    matches!(
        residue,
        'A' | 'C'
            | 'D'
            | 'E'
            | 'F'
            | 'G'
            | 'H'
            | 'I'
            | 'K'
            | 'L'
            | 'M'
            | 'N'
            | 'P'
            | 'Q'
            | 'R'
            | 'S'
            | 'T'
            | 'V'
            | 'W'
            | 'Y'
    )
}

pub(crate) fn is_ambiguous_residue(residue: char) -> bool {
    if residue.is_ascii() {
        return AMBIGUOUS_RESIDUE_LOOKUP[residue as usize];
    }

    matches!(residue, 'X' | 'B' | 'Z' | 'J' | 'U' | 'O')
}

pub(crate) fn is_protein_20_residue_byte(residue: u8) -> bool {
    PROTEIN_20_RESIDUE_LOOKUP[residue as usize]
}

pub(crate) fn is_ambiguous_residue_byte(residue: u8) -> bool {
    AMBIGUOUS_RESIDUE_LOOKUP[residue as usize]
}

const PROTEIN_20_RESIDUE_LOOKUP: [bool; 256] = {
    let mut lookup = [false; 256];
    lookup[b'A' as usize] = true;
    lookup[b'C' as usize] = true;
    lookup[b'D' as usize] = true;
    lookup[b'E' as usize] = true;
    lookup[b'F' as usize] = true;
    lookup[b'G' as usize] = true;
    lookup[b'H' as usize] = true;
    lookup[b'I' as usize] = true;
    lookup[b'K' as usize] = true;
    lookup[b'L' as usize] = true;
    lookup[b'M' as usize] = true;
    lookup[b'N' as usize] = true;
    lookup[b'P' as usize] = true;
    lookup[b'Q' as usize] = true;
    lookup[b'R' as usize] = true;
    lookup[b'S' as usize] = true;
    lookup[b'T' as usize] = true;
    lookup[b'V' as usize] = true;
    lookup[b'W' as usize] = true;
    lookup[b'Y' as usize] = true;
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
    lookup
};

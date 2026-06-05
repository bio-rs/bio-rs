pub fn normalize_sequence(sequence: &str) -> String {
    let mut normalized = String::with_capacity(sequence.len());
    append_normalized_sequence(sequence, &mut normalized);
    normalized
}

pub(crate) fn normalize_sequence_bytes(sequence: &[u8]) -> Vec<u8> {
    let mut normalized = Vec::with_capacity(sequence.len());
    append_normalized_sequence_bytes_to_vec(sequence, &mut normalized);
    normalized
}

pub(crate) fn append_normalized_sequence(sequence: &str, output: &mut String) {
    output.reserve(sequence.len());
    if sequence.is_ascii() {
        append_normalized_sequence_bytes(sequence.as_bytes(), output);
        return;
    }

    output.extend(normalized_residues(sequence));
}

pub(crate) fn append_normalized_sequence_bytes(sequence: &[u8], output: &mut String) {
    output.reserve(sequence.len());
    for &byte in sequence {
        if !byte.is_ascii_whitespace() {
            output.push(byte.to_ascii_uppercase() as char);
        }
    }
}

pub(crate) fn append_normalized_sequence_to_vec(sequence: &str, output: &mut Vec<u8>) {
    output.reserve(sequence.len());
    if sequence.is_ascii() {
        append_normalized_sequence_bytes_to_vec(sequence.as_bytes(), output);
        return;
    }
    for symbol in normalized_residues(sequence) {
        let mut buf = [0; 4];
        let encoded = symbol.encode_utf8(&mut buf);
        output.extend_from_slice(encoded.as_bytes());
    }
}

pub(crate) fn append_normalized_sequence_bytes_to_vec(sequence: &[u8], output: &mut Vec<u8>) {
    output.reserve(sequence.len());
    for &byte in sequence {
        if !byte.is_ascii_whitespace() {
            output.push(byte.to_ascii_uppercase());
        }
    }
}

pub(crate) fn normalized_residues(sequence: &str) -> impl Iterator<Item = char> + '_ {
    sequence
        .chars()
        .filter(|residue| !residue.is_whitespace())
        .map(|residue| residue.to_ascii_uppercase())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum NormalizedResidue {
    Byte { value: u8, position: usize },
    Char { value: char, position: usize },
}

pub(crate) fn for_each_normalized_residue(
    normalized: &[u8],
    mut visit: impl FnMut(NormalizedResidue),
) {
    if normalized.is_ascii() {
        visit_normalized_bytes(normalized, &mut visit);
    } else if let Ok(sequence) = std::str::from_utf8(normalized) {
        for (index, residue) in sequence.chars().enumerate() {
            visit(NormalizedResidue::Char {
                value: residue,
                position: index + 1,
            });
        }
    } else {
        visit_normalized_bytes(normalized, &mut visit);
    }
}

fn visit_normalized_bytes(normalized: &[u8], visit: &mut impl FnMut(NormalizedResidue)) {
    for (index, byte) in normalized.iter().copied().enumerate() {
        visit(NormalizedResidue::Byte {
            value: byte,
            position: index + 1,
        });
    }
}

#[cfg(test)]
mod tests;

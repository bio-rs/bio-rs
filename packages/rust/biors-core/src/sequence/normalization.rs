pub fn normalize_sequence(sequence: &str) -> String {
    let mut normalized = String::with_capacity(sequence.len());
    append_normalized_sequence(sequence, &mut normalized);
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

#[cfg(test)]
mod tests;

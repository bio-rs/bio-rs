use super::*;

#[test]
fn append_normalized_sequence_extends_existing_buffer() {
    let mut output = String::from("AC");

    append_normalized_sequence(" d e\tf g ", &mut output);

    assert_eq!(output, "ACDEFG");
}

#[test]
fn append_normalized_sequence_bytes_extends_existing_buffer() {
    let mut output = String::from("AC");

    append_normalized_sequence_bytes(b" d e\tf g ", &mut output);

    assert_eq!(output, "ACDEFG");
}

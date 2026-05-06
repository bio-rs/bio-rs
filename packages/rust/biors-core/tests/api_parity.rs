use biors_core::{
    parse_fasta_records, parse_fasta_records_reader, tokenize_fasta_records,
    tokenize_fasta_records_reader,
};
use std::io::Cursor;

#[test]
fn string_and_reader_fasta_paths_stay_behaviorally_identical() {
    let inputs = [
        ">seq1\nACDE\n",
        "  >seq1 description\r\n ac de \r\n\r\n>seq2\nxbzjuo*\n",
        ">seq1\nACDE\n>seq2\n",
        "ACDE\n",
        ">\nACDE\n",
    ];

    for input in inputs {
        let string_parse = parse_fasta_records(input);
        let reader_parse =
            parse_fasta_records_reader(Cursor::new(input)).map(|parsed| parsed.records);
        assert_eq!(
            reader_parse.map_err(|error| error.code()),
            string_parse.map_err(|error| error.code())
        );

        let string_tokens = tokenize_fasta_records(input);
        let reader_tokens =
            tokenize_fasta_records_reader(Cursor::new(input)).map(|parsed| parsed.records);
        assert_eq!(
            reader_tokens.map_err(|error| error.code()),
            string_tokens.map_err(|error| error.code())
        );
    }
}

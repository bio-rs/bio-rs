use super::*;
use std::io::Cursor;

#[derive(Default)]
struct ByteCountingSink {
    byte_lines: usize,
    text_lines: usize,
}

impl FastaRecordSink for ByteCountingSink {
    fn push_sequence_line(&mut self, _line: &str) {
        self.text_lines += 1;
    }

    fn push_sequence_line_bytes(&mut self, _line: &[u8]) {
        self.byte_lines += 1;
    }

    fn finish_record(
        &mut self,
        _id: String,
        _header_line: usize,
        _record_index: usize,
    ) -> Result<(), BioRsError> {
        Ok(())
    }
}

#[test]
fn reader_scanner_uses_byte_sink_for_ascii_sequence_lines() {
    let mut sink = ByteCountingSink::default();

    scan_fasta_reader(Cursor::new(b">seq1\nACDE\nFGHI\n"), &mut sink, |_line| {}).expect("valid FASTA");

    assert_eq!(sink.byte_lines, 2);
    assert_eq!(sink.text_lines, 0);
}

#[test]
fn ascii_utf8_helper_accepts_ascii_slices() {
    assert_eq!(ascii_utf8_unchecked(b"seq1"), "seq1");
}

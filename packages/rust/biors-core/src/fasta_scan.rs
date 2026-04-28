use crate::verification::StableInputHasher;
use crate::{BioRsError, FastaReadError};
use std::io::BufRead;

pub(crate) trait FastaRecordSink {
    fn push_sequence_line(&mut self, line: &str);

    fn finish_record(
        &mut self,
        id: String,
        header_line: usize,
        record_index: usize,
    ) -> Result<(), BioRsError>;
}

pub(crate) fn scan_fasta_str<S: FastaRecordSink>(
    input: &str,
    sink: &mut S,
) -> Result<(), BioRsError> {
    if input.trim().is_empty() {
        return Err(BioRsError::EmptyInput);
    }

    let mut state = FastaScanState::default();
    for (line_index, raw_line) in input.lines().enumerate() {
        let line_number = line_index + 1;
        state.scan_line(raw_line, line_number, sink)?;
    }
    state.finish(sink)
}

pub(crate) fn scan_fasta_reader<R: BufRead, S: FastaRecordSink>(
    mut reader: R,
    sink: &mut S,
) -> Result<String, FastaReadError> {
    let mut state = FastaScanState::default();
    let mut hasher = StableInputHasher::new();
    let mut raw_line = String::new();

    loop {
        raw_line.clear();
        let bytes = reader.read_line(&mut raw_line)?;
        if bytes == 0 {
            break;
        }
        state.line_number += 1;
        hasher.update(raw_line.as_bytes());
        state.scan_line(&raw_line, state.line_number, sink)?;
    }

    if state.line_number == 0 || state.no_records_started() {
        return Err(BioRsError::EmptyInput.into());
    }

    state.finish(sink)?;
    Ok(hasher.finalize())
}

#[derive(Default)]
struct FastaScanState {
    current_id: Option<String>,
    current_header_line: usize,
    current_record_index: usize,
    line_number: usize,
}

impl FastaScanState {
    fn scan_line<S: FastaRecordSink>(
        &mut self,
        raw_line: &str,
        line_number: usize,
        sink: &mut S,
    ) -> Result<(), BioRsError> {
        let line = trim_fasta_line(raw_line);

        if line.is_empty() {
            return Ok(());
        }

        if let Some(header) = line.strip_prefix('>') {
            let next_id = fasta_id(header).ok_or(BioRsError::MissingIdentifier {
                line: line_number,
                record_index: self.current_record_index,
            })?;
            if let Some(id) = self.current_id.replace(next_id) {
                sink.finish_record(id, self.current_header_line, self.current_record_index)?;
                self.current_record_index += 1;
            }
            self.current_header_line = line_number;
            return Ok(());
        }

        if self.current_id.is_none() {
            return Err(BioRsError::MissingHeader { line: line_number });
        }

        sink.push_sequence_line(line);
        Ok(())
    }

    fn no_records_started(&self) -> bool {
        self.current_record_index == 0 && self.current_id.is_none()
    }

    fn finish<S: FastaRecordSink>(self, sink: &mut S) -> Result<(), BioRsError> {
        let id = self
            .current_id
            .ok_or(BioRsError::MissingHeader { line: 1 })?;
        sink.finish_record(id, self.current_header_line, self.current_record_index)
    }
}

fn trim_fasta_line(line: &str) -> &str {
    if line.is_ascii() {
        trim_ascii(line)
    } else {
        line.trim()
    }
}

fn trim_ascii(line: &str) -> &str {
    let bytes = line.as_bytes();
    let mut start = 0;
    let mut end = bytes.len();

    while start < end && bytes[start].is_ascii_whitespace() {
        start += 1;
    }

    while end > start && bytes[end - 1].is_ascii_whitespace() {
        end -= 1;
    }

    &line[start..end]
}

fn fasta_id(header: &str) -> Option<String> {
    if header.is_ascii() {
        return fasta_id_ascii(header);
    }

    header.split_whitespace().next().map(str::to_string)
}

fn fasta_id_ascii(header: &str) -> Option<String> {
    let bytes = header.as_bytes();
    let mut start = 0;

    while start < bytes.len() && bytes[start].is_ascii_whitespace() {
        start += 1;
    }

    if start == bytes.len() {
        return None;
    }

    let mut end = start;
    while end < bytes.len() && !bytes[end].is_ascii_whitespace() {
        end += 1;
    }

    Some(header[start..end].to_string())
}

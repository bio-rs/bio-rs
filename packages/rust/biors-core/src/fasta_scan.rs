use crate::verification::StableInputHasher;
use crate::{BioRsError, FastaReadError};
use std::io::{BufRead, ErrorKind};

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
    let mut raw_line = Vec::new();

    loop {
        raw_line.clear();
        let bytes = reader.read_until(b'\n', &mut raw_line)?;
        if bytes == 0 {
            break;
        }
        state.line_number += 1;
        hasher.update(&raw_line);
        state.scan_line_bytes(&raw_line, sink)?;
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
    fn scan_line_bytes<S: FastaRecordSink>(
        &mut self,
        raw_line: &[u8],
        sink: &mut S,
    ) -> Result<(), FastaReadError> {
        let line_number = self.line_number;
        let raw_line = trim_fasta_line_bytes(raw_line);

        if raw_line.is_empty() {
            return Ok(());
        }

        if raw_line[0] == b'>' {
            let header = &raw_line[1..];
            let next_id = fasta_id_bytes(header).ok_or(BioRsError::MissingIdentifier {
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
            return Err(BioRsError::MissingHeader { line: line_number }.into());
        }

        if raw_line.is_ascii() {
            // SAFETY: `is_ascii` guarantees valid UTF-8.
            sink.push_sequence_line(unsafe { std::str::from_utf8_unchecked(raw_line) });
        } else {
            let line = std::str::from_utf8(raw_line)
                .map_err(|error| invalid_utf8_error(line_number, error))?;
            sink.push_sequence_line(line.trim());
        }
        Ok(())
    }

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

fn invalid_utf8_error(line: usize, error: std::str::Utf8Error) -> FastaReadError {
    FastaReadError::Io(std::io::Error::new(
        ErrorKind::InvalidData,
        format!("FASTA input contains invalid UTF-8 at line {line}: {error}"),
    ))
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

fn trim_fasta_line_bytes(line: &[u8]) -> &[u8] {
    if line.is_ascii() {
        trim_ascii_bytes(line)
    } else {
        let mut end = line.len();
        while end > 0 && matches!(line[end - 1], b'\n' | b'\r') {
            end -= 1;
        }
        &line[..end]
    }
}

fn trim_ascii_bytes(line: &[u8]) -> &[u8] {
    let mut start = 0;
    let mut end = line.len();

    while start < end && line[start].is_ascii_whitespace() {
        start += 1;
    }

    while end > start && line[end - 1].is_ascii_whitespace() {
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

fn fasta_id_bytes(header: &[u8]) -> Option<String> {
    if header.is_ascii() {
        return fasta_id_ascii_bytes(header);
    }

    let header = std::str::from_utf8(header).ok()?;
    header.split_whitespace().next().map(str::to_string)
}

fn fasta_id_ascii_bytes(header: &[u8]) -> Option<String> {
    let mut start = 0;

    while start < header.len() && header[start].is_ascii_whitespace() {
        start += 1;
    }

    if start == header.len() {
        return None;
    }

    let mut end = start;
    while end < header.len() && !header[end].is_ascii_whitespace() {
        end += 1;
    }

    // SAFETY: ASCII identifier slices are valid UTF-8.
    Some(unsafe { std::str::from_utf8_unchecked(&header[start..end]) }.to_string())
}

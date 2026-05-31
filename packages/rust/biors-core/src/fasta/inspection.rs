use crate::error::{BioRsError, FastaReadError};
use crate::fasta_scan::{scan_fasta_reader, FastaRecordSink};
use crate::hash::Sha256ByteHasher;
use crate::sequence::normalized_residues;
use crate::verification::StableInputHasher;
use serde::{Deserialize, Serialize};
use std::io::BufRead;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FastaRecordMetadata {
    pub id: String,
    pub length: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InspectedFastaInput {
    pub input_hash: String,
    pub sha256: String,
    pub records: Vec<FastaRecordMetadata>,
}

/// Inspect FASTA records from a buffered reader without retaining full records.
pub fn inspect_fasta_records_reader<R: BufRead>(
    reader: R,
) -> Result<InspectedFastaInput, FastaReadError> {
    let mut sink = RecordMetadataSink::default();
    let mut input_hasher = StableInputHasher::new();
    let mut sha256_hasher = Sha256ByteHasher::new();
    scan_fasta_reader(reader, &mut sink, |line| {
        input_hasher.update(line);
        sha256_hasher.update(line);
    })?;

    Ok(InspectedFastaInput {
        input_hash: input_hasher.finalize(),
        sha256: sha256_hasher.finalize(),
        records: sink.records,
    })
}

#[derive(Default)]
struct RecordMetadataSink {
    records: Vec<FastaRecordMetadata>,
    current_length: usize,
}

impl FastaRecordSink for RecordMetadataSink {
    fn push_sequence_line(&mut self, line: &str) {
        if line.is_ascii() {
            self.push_sequence_line_bytes(line.as_bytes());
            return;
        }

        self.current_length += normalized_residues(line).count();
    }

    fn push_sequence_line_bytes(&mut self, line: &[u8]) {
        self.current_length += line
            .iter()
            .filter(|byte| !byte.is_ascii_whitespace())
            .count();
    }

    fn finish_record(
        &mut self,
        id: String,
        line: usize,
        record_index: usize,
    ) -> Result<(), BioRsError> {
        if self.current_length == 0 {
            return Err(BioRsError::MissingSequence {
                id,
                line,
                record_index,
            });
        }

        self.records.push(FastaRecordMetadata {
            id,
            length: self.current_length,
        });
        self.current_length = 0;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inspect_reader_reports_metadata_and_raw_sha256_without_records() {
        let input = b">seq1\nACDE\n>seq2\nFGHI\n";
        let result =
            inspect_fasta_records_reader(std::io::Cursor::new(input)).expect("inspect FASTA");

        assert!(result.input_hash.starts_with("fnv1a64:"));
        assert_eq!(result.sha256, crate::hash::sha256_bytes_digest(input));
        assert_eq!(
            result.records,
            vec![
                FastaRecordMetadata {
                    id: "seq1".to_string(),
                    length: 4,
                },
                FastaRecordMetadata {
                    id: "seq2".to_string(),
                    length: 4,
                },
            ]
        );
    }
}

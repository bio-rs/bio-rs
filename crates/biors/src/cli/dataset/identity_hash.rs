use super::inspect_output::{DatasetDescriptor, DatasetFile, DatasetSample};
use crate::errors::CliError;
use biors_core::hash::sha256_canonical_json_digest;
use serde::Serialize;
use std::collections::BTreeMap;

#[derive(Debug, Serialize)]
struct DatasetHashInput<'a> {
    descriptor: &'a DatasetDescriptor,
    metadata: &'a BTreeMap<String, String>,
    resolved_files: &'a [DatasetFile],
    samples: &'a [DatasetSample],
}

#[derive(Debug, Serialize)]
struct DatasetContentHashInput<'a> {
    descriptor: &'a DatasetDescriptor,
    metadata: &'a BTreeMap<String, String>,
    files: Vec<DatasetContentFile<'a>>,
    samples: Vec<DatasetContentSample<'a>>,
}

#[derive(Debug, Serialize)]
struct DatasetContentFile<'a> {
    bytes: u64,
    sha256: &'a str,
    records: usize,
}

#[derive(Debug, Serialize)]
struct DatasetContentSample<'a> {
    sample_id: &'a str,
    record_index: usize,
    file_sha256: &'a str,
    sequence_length: usize,
}

pub(super) fn dataset_content_hash(
    descriptor: &DatasetDescriptor,
    metadata: &BTreeMap<String, String>,
    resolved_files: &[DatasetFile],
    samples: &[DatasetSample],
) -> Result<String, CliError> {
    let mut files: Vec<_> = resolved_files
        .iter()
        .map(|file| DatasetContentFile {
            bytes: file.bytes,
            sha256: file.sha256.as_str(),
            records: file.records,
        })
        .collect();
    files.sort_by(|left, right| {
        left.sha256
            .cmp(right.sha256)
            .then_with(|| left.bytes.cmp(&right.bytes))
            .then_with(|| left.records.cmp(&right.records))
    });

    let mut content_samples: Vec<_> = samples
        .iter()
        .map(|sample| DatasetContentSample {
            sample_id: sample.sample_id.as_str(),
            record_index: sample.record_index,
            file_sha256: sample.file_sha256.as_str(),
            sequence_length: sample.sequence_length,
        })
        .collect();
    content_samples.sort_by(|left, right| {
        left.file_sha256
            .cmp(right.file_sha256)
            .then_with(|| left.record_index.cmp(&right.record_index))
            .then_with(|| left.sample_id.cmp(right.sample_id))
            .then_with(|| left.sequence_length.cmp(&right.sequence_length))
    });

    let input = DatasetContentHashInput {
        descriptor,
        metadata,
        files,
        samples: content_samples,
    };
    let bytes = serde_json::to_vec(&input).map_err(CliError::Serialization)?;
    Ok(sha256_canonical_json_digest(&bytes))
}

pub(super) fn dataset_mapping_hash(
    descriptor: &DatasetDescriptor,
    metadata: &BTreeMap<String, String>,
    resolved_files: &[DatasetFile],
    samples: &[DatasetSample],
) -> Result<String, CliError> {
    let input = DatasetHashInput {
        descriptor,
        metadata,
        resolved_files,
        samples,
    };
    let bytes = serde_json::to_vec(&input).map_err(CliError::Serialization)?;
    Ok(sha256_canonical_json_digest(&bytes))
}

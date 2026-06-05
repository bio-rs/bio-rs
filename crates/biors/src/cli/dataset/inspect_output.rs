use super::fasta_inspection::inspect_dataset_file;
use super::identity_hash::{dataset_content_hash, dataset_mapping_hash};
use crate::errors::CliError;
use crate::input::ResolvedInputDataset;
use serde::Serialize;
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize)]
pub(super) struct DatasetDescriptor {
    pub(super) source: String,
    pub(super) version: String,
    pub(super) split: String,
}

#[derive(Debug, Serialize)]
pub(super) struct DatasetInspectOutput {
    provided_inputs: usize,
    descriptor: DatasetDescriptor,
    metadata: BTreeMap<String, String>,
    files: usize,
    total_bytes: u64,
    sample_count: usize,
    dataset_hash: String,
    dataset_mapping_hash: String,
    resolved_files: Vec<DatasetFile>,
    samples: Vec<DatasetSample>,
}

#[derive(Debug, Serialize)]
pub(super) struct DatasetFile {
    pub(super) path: String,
    pub(super) bytes: u64,
    pub(super) sha256: String,
    pub(super) records: usize,
}

#[derive(Debug, Serialize)]
pub(super) struct DatasetSample {
    pub(super) dataset: DatasetDescriptor,
    pub(super) sample_id: String,
    pub(super) record_index: usize,
    pub(super) file_path: String,
    pub(super) file_sha256: String,
    pub(super) sequence_length: usize,
}

impl DatasetInspectOutput {
    pub(super) fn from_dataset(
        dataset: ResolvedInputDataset,
        descriptor: DatasetDescriptor,
        metadata: BTreeMap<String, String>,
    ) -> Result<Self, CliError> {
        let mut resolved_files = Vec::with_capacity(dataset.files.len());
        let mut samples = Vec::new();

        for file in dataset.files {
            let inspected = inspect_dataset_file(file, &descriptor)?;
            samples.extend(inspected.samples);
            resolved_files.push(inspected.file);
        }

        let total_bytes = resolved_files.iter().map(|file| file.bytes).sum();
        let sample_count = samples.len();
        let dataset_hash = dataset_content_hash(&descriptor, &metadata, &resolved_files, &samples)?;
        let dataset_mapping_hash =
            dataset_mapping_hash(&descriptor, &metadata, &resolved_files, &samples)?;
        Ok(Self {
            provided_inputs: dataset.provided_inputs,
            files: resolved_files.len(),
            descriptor,
            metadata,
            total_bytes,
            sample_count,
            dataset_hash,
            dataset_mapping_hash,
            resolved_files,
            samples,
        })
    }
}

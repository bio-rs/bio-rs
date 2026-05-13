use super::DatasetCommand;
use crate::errors::CliError;
use crate::input::{resolve_fasta_input_dataset, ResolvedInputDataset, ResolvedInputFile};
use crate::output::print_success;
use biors_core::{
    fasta::parse_fasta_records_reader, hash::sha256_digest, sequence::ProteinSequence,
};
use serde::Serialize;
use std::collections::BTreeMap;
use std::io::Cursor;

#[derive(Debug, Clone, Serialize)]
struct DatasetDescriptor {
    source: String,
    version: String,
    split: String,
}

#[derive(Debug, Serialize)]
struct DatasetInspectOutput {
    provided_inputs: usize,
    descriptor: DatasetDescriptor,
    metadata: BTreeMap<String, String>,
    files: usize,
    total_bytes: u64,
    sample_count: usize,
    dataset_hash: String,
    resolved_files: Vec<DatasetFile>,
    samples: Vec<DatasetSample>,
}

#[derive(Debug, Serialize)]
struct DatasetFile {
    path: String,
    bytes: u64,
    sha256: String,
    records: usize,
}

#[derive(Debug, Serialize)]
struct DatasetSample {
    dataset: DatasetDescriptor,
    sample_id: String,
    record_index: usize,
    file_path: String,
    file_sha256: String,
    sequence_length: usize,
}

#[derive(Debug, Serialize)]
struct DatasetHashInput<'a> {
    descriptor: &'a DatasetDescriptor,
    metadata: &'a BTreeMap<String, String>,
    resolved_files: &'a [DatasetFile],
    samples: &'a [DatasetSample],
}

pub(crate) fn run_dataset_command(command: DatasetCommand) -> Result<(), CliError> {
    match command {
        DatasetCommand::Inspect {
            source,
            version,
            split,
            metadata,
            inputs,
        } => {
            let dataset = resolve_fasta_input_dataset(&inputs)?;
            if dataset.files.is_empty() {
                return Err(CliError::Validation {
                    code: "dataset.no_inputs",
                    message: "dataset inspect did not resolve any FASTA input files".to_string(),
                    location: None,
                });
            }
            let descriptor = DatasetDescriptor {
                source,
                version,
                split,
            };
            let metadata = parse_metadata(metadata)?;
            print_success(
                None,
                DatasetInspectOutput::from_dataset(dataset, descriptor, metadata)?,
            )
        }
    }
}

impl DatasetInspectOutput {
    fn from_dataset(
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
        let dataset_hash = dataset_hash(&descriptor, &metadata, &resolved_files, &samples)?;
        Ok(Self {
            provided_inputs: dataset.provided_inputs,
            files: resolved_files.len(),
            descriptor,
            metadata,
            total_bytes,
            sample_count,
            dataset_hash,
            resolved_files,
            samples,
        })
    }
}

struct InspectedDatasetFile {
    file: DatasetFile,
    samples: Vec<DatasetSample>,
}

fn inspect_dataset_file(
    file: ResolvedInputFile,
    descriptor: &DatasetDescriptor,
) -> Result<InspectedDatasetFile, CliError> {
    let bytes = std::fs::read(&file.path).map_err(|source| CliError::Read {
        path: file.path.clone(),
        source,
    })?;
    let sha256 = sha256_digest(&bytes);
    let records = parse_fasta_records_reader(Cursor::new(&bytes))
        .map_err(|error| CliError::from_fasta_read(file.path.clone(), error))?
        .records;
    let path = file.path.display().to_string();
    let samples = samples_from_records(descriptor, &path, &sha256, &records);

    Ok(InspectedDatasetFile {
        file: DatasetFile {
            path,
            bytes: file.bytes,
            sha256,
            records: records.len(),
        },
        samples,
    })
}

fn samples_from_records(
    descriptor: &DatasetDescriptor,
    file_path: &str,
    file_sha256: &str,
    records: &[ProteinSequence],
) -> Vec<DatasetSample> {
    records
        .iter()
        .enumerate()
        .map(|(record_index, record)| DatasetSample {
            dataset: descriptor.clone(),
            sample_id: record.id.clone(),
            record_index,
            file_path: file_path.to_string(),
            file_sha256: file_sha256.to_string(),
            sequence_length: record.sequence.len(),
        })
        .collect()
}

fn parse_metadata(values: Vec<String>) -> Result<BTreeMap<String, String>, CliError> {
    let mut metadata = BTreeMap::new();
    for value in values {
        let Some((key, val)) = value.split_once('=') else {
            return Err(CliError::Validation {
                code: "dataset.invalid_metadata",
                message: "dataset metadata must use key=value".to_string(),
                location: Some(value),
            });
        };
        let key = key.trim();
        let val = val.trim();
        if key.is_empty() || val.is_empty() {
            return Err(CliError::Validation {
                code: "dataset.invalid_metadata",
                message: "dataset metadata keys and values must be non-empty".to_string(),
                location: Some(value),
            });
        }
        metadata.insert(key.to_string(), val.to_string());
    }
    Ok(metadata)
}

fn dataset_hash(
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
    Ok(sha256_digest(&bytes))
}

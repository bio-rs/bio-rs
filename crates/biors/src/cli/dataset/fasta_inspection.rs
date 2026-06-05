use super::inspect_output::{DatasetDescriptor, DatasetFile, DatasetSample};
use crate::errors::CliError;
use crate::input::ResolvedInputFile;
use biors_core::fasta::{inspect_fasta_records_reader, FastaRecordMetadata};
use std::fs::File;
use std::io::BufReader;

pub(super) struct InspectedDatasetFile {
    pub(super) file: DatasetFile,
    pub(super) samples: Vec<DatasetSample>,
}

pub(super) fn inspect_dataset_file(
    file: ResolvedInputFile,
    descriptor: &DatasetDescriptor,
) -> Result<InspectedDatasetFile, CliError> {
    let reader = File::open(&file.path).map_err(|source| CliError::Read {
        path: file.path.clone(),
        source,
    })?;
    let inspected = inspect_fasta_records_reader(BufReader::new(reader))
        .map_err(|error| CliError::from_fasta_read(file.path.clone(), error))?;
    let path = file.path.display().to_string();
    let samples = samples_from_records(descriptor, &path, &inspected.sha256, &inspected.records);
    let records = inspected.records.len();

    Ok(InspectedDatasetFile {
        file: DatasetFile {
            path,
            bytes: file.bytes,
            sha256: inspected.sha256,
            records,
        },
        samples,
    })
}

fn samples_from_records(
    descriptor: &DatasetDescriptor,
    file_path: &str,
    file_sha256: &str,
    records: &[FastaRecordMetadata],
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
            sequence_length: record.length,
        })
        .collect()
}

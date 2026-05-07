use super::DatasetCommand;
use crate::errors::CliError;
use crate::input::{resolve_fasta_input_dataset, ResolvedInputDataset, ResolvedInputFile};
use crate::output::print_success;
use serde::Serialize;

#[derive(Debug, Serialize)]
struct DatasetInspectOutput {
    provided_inputs: usize,
    files: usize,
    total_bytes: u64,
    resolved_files: Vec<DatasetFile>,
}

#[derive(Debug, Serialize)]
struct DatasetFile {
    path: String,
    bytes: u64,
}

pub(crate) fn run_dataset_command(command: DatasetCommand) -> Result<(), CliError> {
    match command {
        DatasetCommand::Inspect { inputs } => {
            let dataset = resolve_fasta_input_dataset(&inputs)?;
            if dataset.files.is_empty() {
                return Err(CliError::Validation {
                    code: "dataset.no_inputs",
                    message: "dataset inspect did not resolve any FASTA input files".to_string(),
                    location: None,
                });
            }
            print_success(None, DatasetInspectOutput::from_dataset(dataset))
        }
    }
}

impl DatasetInspectOutput {
    fn from_dataset(dataset: ResolvedInputDataset) -> Self {
        let total_bytes = dataset.files.iter().map(|file| file.bytes).sum();
        Self {
            provided_inputs: dataset.provided_inputs,
            files: dataset.files.len(),
            total_bytes,
            resolved_files: dataset
                .files
                .into_iter()
                .map(DatasetFile::from_file)
                .collect(),
        }
    }
}

impl DatasetFile {
    fn from_file(file: ResolvedInputFile) -> Self {
        Self {
            path: file.path.display().to_string(),
            bytes: file.bytes,
        }
    }
}

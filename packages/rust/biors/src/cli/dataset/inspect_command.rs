use super::super::DatasetCommand;
use super::inspect_output::{DatasetDescriptor, DatasetInspectOutput};
use super::metadata::parse_metadata;
use crate::errors::CliError;
use crate::input::resolve_fasta_input_dataset;
use crate::output::print_success;

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

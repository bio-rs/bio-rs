use crate::cli::{StructureCommand, StructureFormatArg};
use crate::errors::CliError;
use crate::input::open_buffered_input;
use crate::output::print_success;
use biors_core::structure::{
    extract_structure_sequences, parse_pdb_record_reader, validate_pdb_reader_with_hash,
};
use std::path::PathBuf;

pub(super) fn run_structure_command(command: StructureCommand) -> Result<(), CliError> {
    match command {
        StructureCommand::Validate { format, path } => run_structure_validation(format, path),
        StructureCommand::Sequence { format, path } => run_structure_sequence(format, path),
    }
}

fn run_structure_validation(format: StructureFormatArg, path: PathBuf) -> Result<(), CliError> {
    match format {
        StructureFormatArg::Pdb => {
            let reader = open_buffered_input(&path)?;
            let output = validate_pdb_reader_with_hash(reader)
                .map_err(|error| CliError::from_structure_read(path, error))?;
            print_success(Some(output.input_hash), output.report)
        }
    }
}

fn run_structure_sequence(format: StructureFormatArg, path: PathBuf) -> Result<(), CliError> {
    match format {
        StructureFormatArg::Pdb => {
            let reader = open_buffered_input(&path)?;
            let output = parse_pdb_record_reader(reader)
                .map_err(|error| CliError::from_structure_read(path, error))?;
            let sequences = extract_structure_sequences(&output.record);
            print_success(Some(output.input_hash), sequences)
        }
    }
}

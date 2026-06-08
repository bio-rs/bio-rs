use crate::cli::{FormatArg, FormatsCommand};
use crate::errors::CliError;
use crate::input::open_buffered_input;
use crate::output::print_success;
use biors_core::formats::{format_capabilities, validate_fastq_reader_with_hash};
use std::path::PathBuf;

pub(super) fn run_formats_command(command: FormatsCommand) -> Result<(), CliError> {
    match command {
        FormatsCommand::List => print_success(None, format_capabilities()),
        FormatsCommand::Validate { format, path } => run_format_validation(format, path),
    }
}

fn run_format_validation(format: FormatArg, path: PathBuf) -> Result<(), CliError> {
    match format {
        FormatArg::Fastq => {
            let reader = open_buffered_input(&path)?;
            let output = validate_fastq_reader_with_hash(reader)
                .map_err(|error| CliError::from_format_read(path, error))?;
            print_success(Some(output.input_hash), output.report)
        }
    }
}

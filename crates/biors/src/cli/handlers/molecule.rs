use crate::cli::{MoleculeCommand, MoleculeFormatArg};
use crate::errors::CliError;
use crate::input::open_buffered_input;
use crate::output::print_success;
use biors_core::molecule::{
    parse_mol2_records_reader, parse_sdf_records_reader, parse_smiles_records_reader,
    validate_molecule_records, validate_smiles_reader_with_hash,
};
use std::path::PathBuf;

pub(super) fn run_molecule_command(command: MoleculeCommand) -> Result<(), CliError> {
    match command {
        MoleculeCommand::Validate { format, path } => run_molecule_validation(format, path),
        MoleculeCommand::Inspect { format, path } => run_molecule_inspect(format, path),
    }
}

fn run_molecule_validation(format: MoleculeFormatArg, path: PathBuf) -> Result<(), CliError> {
    match format {
        MoleculeFormatArg::Smiles => {
            let reader = open_buffered_input(&path)?;
            let output = validate_smiles_reader_with_hash(reader)
                .map_err(|error| CliError::from_molecule_read(path, error))?;
            print_success(Some(output.input_hash), output.report)
        }
        MoleculeFormatArg::Sdf => {
            let reader = open_buffered_input(&path)?;
            let output = parse_sdf_records_reader(reader)
                .map_err(|error| CliError::from_molecule_read(path, error))?;
            print_success(
                Some(output.input_hash),
                validate_molecule_records(&output.records),
            )
        }
        MoleculeFormatArg::Mol2 => {
            let reader = open_buffered_input(&path)?;
            let output = parse_mol2_records_reader(reader)
                .map_err(|error| CliError::from_molecule_read(path, error))?;
            print_success(
                Some(output.input_hash),
                validate_molecule_records(&output.records),
            )
        }
    }
}

fn run_molecule_inspect(format: MoleculeFormatArg, path: PathBuf) -> Result<(), CliError> {
    match format {
        MoleculeFormatArg::Smiles => {
            let reader = open_buffered_input(&path)?;
            let output = parse_smiles_records_reader(reader)
                .map_err(|error| CliError::from_molecule_read(path, error))?;
            print_success(Some(output.input_hash), output.records)
        }
        MoleculeFormatArg::Sdf => {
            let reader = open_buffered_input(&path)?;
            let output = parse_sdf_records_reader(reader)
                .map_err(|error| CliError::from_molecule_read(path, error))?;
            print_success(Some(output.input_hash), output.records)
        }
        MoleculeFormatArg::Mol2 => {
            let reader = open_buffered_input(&path)?;
            let output = parse_mol2_records_reader(reader)
                .map_err(|error| CliError::from_molecule_read(path, error))?;
            print_success(Some(output.input_hash), output.records)
        }
    }
}

use super::{Command, FastaCommand, KindArg, PackageCommand, PaddingArg, SeqCommand};
use crate::errors::{classify_validation_code, classify_verification_code, CliError};
use crate::input::{open_fasta_input, read_fixture_observations, read_package_manifest};
use crate::output::print_success;
use biors_core::{
    build_model_inputs_checked, inspect_package_manifest, plan_runtime_bridge,
    summarize_fasta_records_reader, tokenize_fasta_records_reader,
    validate_fasta_reader_with_kind_and_hash, validate_package_manifest_artifacts,
    verify_package_outputs_with_observation_base, ModelInputPolicy,
};
use std::path::PathBuf;

pub fn run(command: Command) -> Result<(), CliError> {
    match command {
        Command::Fasta { command } => run_fasta_command(command),
        Command::Inspect { path } => run_inspect(path),
        Command::ModelInput {
            max_length,
            pad_token_id,
            padding,
            path,
        } => run_model_input(max_length, pad_token_id, padding, path),
        Command::Package { command } => run_package_command(command),
        Command::Seq { command } => run_seq_command(command),
        Command::Tokenize { path } => run_tokenize(path),
    }
}

fn run_fasta_command(command: FastaCommand) -> Result<(), CliError> {
    match command {
        FastaCommand::Validate { kind, path } => run_sequence_validation(path, kind),
    }
}

fn run_seq_command(command: SeqCommand) -> Result<(), CliError> {
    match command {
        SeqCommand::Validate { kind, path } => run_sequence_validation(path, kind),
    }
}

fn run_inspect(path: PathBuf) -> Result<(), CliError> {
    let reader = open_fasta_input(&path)?;
    let output = summarize_fasta_records_reader(reader)
        .map_err(|error| CliError::from_fasta_read(path, error))?;
    print_success(Some(output.input_hash), output.summary)
}

fn run_model_input(
    max_length: usize,
    pad_token_id: u8,
    padding: PaddingArg,
    path: PathBuf,
) -> Result<(), CliError> {
    let reader = open_fasta_input(&path)?;
    let output = tokenize_fasta_records_reader(reader)
        .map_err(|error| CliError::from_fasta_read(path, error))?;
    let model_input = build_model_inputs_checked(
        &output.records,
        ModelInputPolicy {
            max_length,
            pad_token_id,
            padding: padding.into(),
        },
    )?;
    print_success(Some(output.input_hash), model_input)
}

fn run_package_command(command: PackageCommand) -> Result<(), CliError> {
    match command {
        PackageCommand::Bridge { path } => {
            let (manifest, manifest_base_dir) = read_package_manifest(path)?;
            let report = plan_runtime_bridge(&manifest);
            let validation = validate_package_manifest_artifacts(&manifest, &manifest_base_dir);
            if !validation.valid || !report.ready {
                return Err(CliError::Validation {
                    code: "package.bridge_not_ready",
                    message: format!(
                        "{:?}",
                        validation
                            .issues
                            .iter()
                            .chain(report.blocking_issues.iter())
                            .collect::<Vec<_>>()
                    ),
                    location: Some("manifest".to_string()),
                });
            }
            print_success(None, report)
        }
        PackageCommand::Inspect { path } => {
            let (manifest, _) = read_package_manifest(path)?;
            let summary = inspect_package_manifest(&manifest);
            print_success(None, summary)
        }
        PackageCommand::Validate { path } => {
            let (manifest, manifest_base_dir) = read_package_manifest(path)?;
            let report = validate_package_manifest_artifacts(&manifest, &manifest_base_dir);
            if !report.valid {
                return Err(CliError::Validation {
                    code: classify_validation_code(&report),
                    message: format!("{:?}", report.issues),
                    location: Some("manifest".to_string()),
                });
            }
            print_success(None, report)
        }
        PackageCommand::Verify {
            manifest,
            observations,
        } => {
            let (manifest, manifest_base_dir) = read_package_manifest(manifest)?;
            let (observations, observations_base_dir) = read_fixture_observations(observations)?;
            let report = verify_package_outputs_with_observation_base(
                &manifest,
                &observations,
                &manifest_base_dir,
                &observations_base_dir,
            );
            if report.failed > 0 {
                return Err(CliError::Validation {
                    code: classify_verification_code(&report),
                    message: format!(
                        "{:?}",
                        report
                            .results
                            .iter()
                            .filter_map(|result| result.issue.as_ref())
                            .collect::<Vec<_>>()
                    ),
                    location: Some("fixtures".to_string()),
                });
            }
            print_success(None, report)
        }
    }
}

fn run_tokenize(path: PathBuf) -> Result<(), CliError> {
    let reader = open_fasta_input(&path)?;
    let output = tokenize_fasta_records_reader(reader)
        .map_err(|error| CliError::from_fasta_read(path, error))?;
    print_success(Some(output.input_hash), output.records)
}

fn run_sequence_validation(path: PathBuf, kind: KindArg) -> Result<(), CliError> {
    let reader = open_fasta_input(&path)?;
    let output = validate_fasta_reader_with_kind_and_hash(reader, kind.into())
        .map_err(|error| CliError::from_fasta_read(path, error))?;
    print_success(Some(output.input_hash), output.report)
}

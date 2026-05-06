use super::{
    Cli, Command, FastaCommand, KindArg, PackageCommand, PaddingArg, SeqCommand, TokenizerCommand,
    TokenizerProfileArg,
};
use crate::cli::{build_doctor_report, run_batch_command};
use crate::errors::{classify_validation_code, classify_verification_code, CliError};
use crate::input::{
    open_fasta_input, read_fixture_observations, read_package_manifest, read_tokenizer_config,
};
use crate::output::print_success;
use biors_core::{
    build_model_inputs_checked, inspect_package_manifest, plan_runtime_bridge,
    prepare_protein_model_input_workflow, summarize_fasta_records_reader,
    tokenize_fasta_records_reader, tokenize_fasta_records_reader_with_config,
    validate_fasta_reader_with_kind_and_hash, validate_package_manifest_artifacts,
    verify_package_outputs_with_observation_base, ModelInputPolicy, ProteinTokenizerConfig,
};
use clap::CommandFactory;
use std::path::PathBuf;

pub fn run(command: Command) -> Result<(), CliError> {
    match command {
        Command::Batch { command } => run_batch_command(command),
        Command::Completions { shell } => run_completions(shell),
        Command::Doctor => run_doctor(),
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
        Command::Tokenize {
            profile,
            config,
            path,
        } => run_tokenize(profile, config, path),
        Command::Tokenizer { command } => run_tokenizer_command(command),
        Command::Workflow {
            max_length,
            pad_token_id,
            padding,
            path,
        } => run_workflow(max_length, pad_token_id, padding, path),
    }
}

fn run_completions(shell: clap_complete::Shell) -> Result<(), CliError> {
    let mut command = Cli::command();
    let name = command.get_name().to_string();
    clap_complete::generate(shell, &mut command, name, &mut std::io::stdout());
    Ok(())
}

fn run_doctor() -> Result<(), CliError> {
    print_success(None, build_doctor_report())
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

fn run_tokenize(
    profile: TokenizerProfileArg,
    config: Option<PathBuf>,
    path: PathBuf,
) -> Result<(), CliError> {
    let config = resolve_tokenizer_config(profile, config)?;
    let reader = open_fasta_input(&path)?;
    let output = tokenize_fasta_records_reader_with_config(reader, &config)
        .map_err(|error| CliError::from_fasta_read(path, error))?;
    print_success(Some(output.input_hash), output.records)
}

fn run_tokenizer_command(command: TokenizerCommand) -> Result<(), CliError> {
    match command {
        TokenizerCommand::Inspect { profile, config } => {
            let config = resolve_tokenizer_config(profile, config)?;
            print_success(None, biors_core::inspect_protein_tokenizer_config(&config))
        }
    }
}

fn resolve_tokenizer_config(
    profile: TokenizerProfileArg,
    config: Option<PathBuf>,
) -> Result<ProteinTokenizerConfig, CliError> {
    match config {
        Some(path) => read_tokenizer_config(path),
        None => Ok(biors_core::protein_tokenizer_config_for_profile(
            profile.into(),
        )),
    }
}

fn run_workflow(
    max_length: usize,
    pad_token_id: u8,
    padding: PaddingArg,
    path: PathBuf,
) -> Result<(), CliError> {
    let reader = open_fasta_input(&path)?;
    let input = biors_core::parse_fasta_records_reader(reader)
        .map_err(|error| CliError::from_fasta_read(path, error))?;
    let input_hash = input.input_hash.clone();
    let output = prepare_protein_model_input_workflow(
        input.input_hash,
        &input.records,
        ModelInputPolicy {
            max_length,
            pad_token_id,
            padding: padding.into(),
        },
    )?;
    print_success(Some(input_hash), output)
}

fn run_sequence_validation(path: PathBuf, kind: KindArg) -> Result<(), CliError> {
    let reader = open_fasta_input(&path)?;
    let output = validate_fasta_reader_with_kind_and_hash(reader, kind.into())
        .map_err(|error| CliError::from_fasta_read(path, error))?;
    print_success(Some(output.input_hash), output.report)
}

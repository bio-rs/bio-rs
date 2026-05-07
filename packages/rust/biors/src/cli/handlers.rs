use super::{
    Cli, Command, FastaCommand, KindArg, PackageCommand, PaddingArg, SeqCommand, TokenizerCommand,
    TokenizerProfileArg,
};
use crate::cli::{
    build_doctor_report, run_batch_command, run_debug, run_diff, run_pipeline, run_workflow,
};
use crate::errors::{classify_validation_code, classify_verification_code, CliError};
use crate::input::{
    open_fasta_input, read_fixture_observations, read_package_manifest, read_tokenizer_config,
};
use crate::output::print_success;
use biors_core::{
    model_input::{build_model_inputs_checked, ModelInputPolicy},
    package::{inspect_package_manifest, plan_runtime_bridge, validate_package_manifest_artifacts},
    sequence::validate_fasta_reader_with_kind_and_hash,
    tokenizer::{
        inspect_protein_tokenizer_config, protein_tokenizer_config_for_profile,
        summarize_fasta_records_reader, tokenize_fasta_records_reader,
        tokenize_fasta_records_reader_with_config, ProteinTokenizerConfig,
    },
    verification::verify_package_outputs_with_observation_base,
};
use clap::CommandFactory;
use std::path::PathBuf;

pub fn run(command: Command) -> Result<(), CliError> {
    match command {
        Command::Batch { command } => run_batch_command(command),
        Command::Completions { shell } => run_completions(shell),
        Command::Debug { max_length, path } => run_debug(max_length, path),
        Command::Diff { expected, observed } => run_diff(expected, observed),
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
        Command::Pipeline {
            config,
            dry_run,
            explain_plan,
            max_length,
            pad_token_id,
            padding,
            path,
        } => run_pipeline(
            config,
            dry_run,
            explain_plan,
            max_length,
            pad_token_id,
            padding,
            path,
        ),
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
        FastaCommand::Validate { kind, path } => run_fasta_validate(kind, path),
    }
}

fn run_seq_command(command: SeqCommand) -> Result<(), CliError> {
    match command {
        SeqCommand::Validate { kind, path } => run_seq_validate(kind, path),
    }
}

fn run_fasta_validate(kind: KindArg, path: PathBuf) -> Result<(), CliError> {
    run_sequence_validation(path, kind)
}

fn run_seq_validate(kind: KindArg, path: PathBuf) -> Result<(), CliError> {
    run_sequence_validation(path, kind)
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
        PackageCommand::Bridge { path } => run_package_bridge(path),
        PackageCommand::Inspect { path } => run_package_inspect(path),
        PackageCommand::Validate { path } => run_package_validate(path),
        PackageCommand::Verify {
            manifest,
            observations,
        } => run_package_verify(manifest, observations),
    }
}

fn run_package_bridge(path: PathBuf) -> Result<(), CliError> {
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

fn run_package_inspect(path: PathBuf) -> Result<(), CliError> {
    let (manifest, _) = read_package_manifest(path)?;
    let summary = inspect_package_manifest(&manifest);
    print_success(None, summary)
}

fn run_package_validate(path: PathBuf) -> Result<(), CliError> {
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

fn run_package_verify(manifest: PathBuf, observations: PathBuf) -> Result<(), CliError> {
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
            print_success(None, inspect_protein_tokenizer_config(&config))
        }
    }
}

fn resolve_tokenizer_config(
    profile: TokenizerProfileArg,
    config: Option<PathBuf>,
) -> Result<ProteinTokenizerConfig, CliError> {
    match config {
        Some(path) => read_tokenizer_config(path),
        None => Ok(protein_tokenizer_config_for_profile(profile.into())),
    }
}

fn run_sequence_validation(path: PathBuf, kind: KindArg) -> Result<(), CliError> {
    let reader = open_fasta_input(&path)?;
    let output = validate_fasta_reader_with_kind_and_hash(reader, kind.into())
        .map_err(|error| CliError::from_fasta_read(path, error))?;
    print_success(Some(output.input_hash), output.report)
}

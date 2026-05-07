use super::{
    Cli, Command, FastaCommand, KindArg, PaddingArg, SeqCommand, TokenizerCommand,
    TokenizerProfileArg,
};
use crate::cli::{
    build_doctor_report, run_batch_command, run_dataset_command, run_debug, run_diff,
    run_package_command, run_pipeline, run_workflow, PipelineRunOptions,
};
use crate::errors::CliError;
use crate::input::{open_fasta_input, read_tokenizer_config};
use crate::output::print_success;
use biors_core::{
    model_input::{build_model_inputs_checked, ModelInputPolicy},
    sequence::validate_fasta_reader_with_kind_and_hash,
    tokenizer::{
        inspect_protein_tokenizer_config, protein_tokenizer_config_for_profile,
        summarize_fasta_records_reader, tokenize_fasta_records_reader,
        tokenize_fasta_records_reader_with_config, ProteinTokenizerConfig,
    },
};
use clap::CommandFactory;
use std::path::PathBuf;

pub fn run(command: Command) -> Result<(), CliError> {
    match command {
        Command::Batch { command } => run_batch_command(command),
        Command::Completions { shell } => run_completions(shell),
        Command::Dataset { command } => run_dataset_command(command),
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
            package,
            write_lock,
            max_length,
            pad_token_id,
            padding,
            path,
        } => run_pipeline(PipelineRunOptions {
            config,
            dry_run,
            explain_plan,
            package,
            write_lock,
            max_length,
            pad_token_id,
            padding,
            path,
        }),
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

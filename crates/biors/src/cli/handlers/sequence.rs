use super::tokenizer::resolve_tokenizer_config;
use crate::cli::{FastaCommand, KindArg, PaddingArg, SeqCommand, TokenizerProfileArg};
use crate::errors::CliError;
use crate::input::open_buffered_input;
use crate::output::print_success;
use biors_core::{
    model_input::{build_model_inputs_checked, ModelInputPolicy},
    sequence::validate_fasta_reader_with_kind_and_hash,
    tokenizer::{
        protein_tokenizer_config_for_profile, summarize_fasta_records_reader,
        tokenize_fasta_records_reader_with_config,
    },
};
use std::path::PathBuf;

pub(super) fn run_fasta_command(command: FastaCommand) -> Result<(), CliError> {
    match command {
        FastaCommand::Validate { kind, path } => run_sequence_validation(path, kind),
    }
}

pub(super) fn run_seq_command(command: SeqCommand) -> Result<(), CliError> {
    match command {
        SeqCommand::Validate { kind, path } => run_sequence_validation(path, kind),
    }
}

pub(super) fn run_inspect(path: PathBuf) -> Result<(), CliError> {
    let reader = open_buffered_input(&path)?;
    let output = summarize_fasta_records_reader(reader)
        .map_err(|error| CliError::from_fasta_read(path, error))?;
    print_success(Some(output.input_hash), output.summary)
}

pub(super) fn run_model_input(
    profile: TokenizerProfileArg,
    max_length: usize,
    pad_token_id: u8,
    padding: PaddingArg,
    path: PathBuf,
) -> Result<(), CliError> {
    let config = protein_tokenizer_config_for_profile(profile.into());
    let reader = open_buffered_input(&path)?;
    let output = tokenize_fasta_records_reader_with_config(reader, &config)
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

pub(super) fn run_tokenize(
    profile: TokenizerProfileArg,
    config: Option<PathBuf>,
    path: PathBuf,
) -> Result<(), CliError> {
    let config = resolve_tokenizer_config(profile, config)?;
    let reader = open_buffered_input(&path)?;
    let output = tokenize_fasta_records_reader_with_config(reader, &config)
        .map_err(|error| CliError::from_fasta_read(path, error))?;
    print_success(Some(output.input_hash), output.records)
}

fn run_sequence_validation(path: PathBuf, kind: KindArg) -> Result<(), CliError> {
    let reader = open_buffered_input(&path)?;
    let output = validate_fasta_reader_with_kind_and_hash(reader, kind.into())
        .map_err(|error| CliError::from_fasta_read(path, error))?;
    print_success(Some(output.input_hash), output.report)
}

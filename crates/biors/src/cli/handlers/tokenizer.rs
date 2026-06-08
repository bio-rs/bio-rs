use crate::cli::{TokenizerCommand, TokenizerProfileArg};
use crate::errors::CliError;
use crate::input::read_tokenizer_config;
use crate::output::print_success;
use biors_core::tokenizer::{
    inspect_protein_tokenizer_config, protein_tokenizer_config_for_profile, ProteinTokenizerConfig,
};
use std::path::PathBuf;

pub(super) fn run_tokenizer_command(command: TokenizerCommand) -> Result<(), CliError> {
    match command {
        TokenizerCommand::ConvertHf { path, output } => {
            crate::cli::tokenizer_convert::run_tokenizer_convert_hf(path, output)
        }
        TokenizerCommand::Inspect { profile, config } => {
            let config = resolve_tokenizer_config(profile, config)?;
            print_success(None, inspect_protein_tokenizer_config(&config))
        }
    }
}

pub(super) fn resolve_tokenizer_config(
    profile: TokenizerProfileArg,
    config: Option<PathBuf>,
) -> Result<ProteinTokenizerConfig, CliError> {
    match config {
        Some(path) => read_tokenizer_config(path),
        None => Ok(protein_tokenizer_config_for_profile(profile.into())),
    }
}

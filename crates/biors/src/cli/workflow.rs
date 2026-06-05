use super::{PaddingArg, TokenizerProfileArg};
use crate::errors::CliError;
use crate::input::open_fasta_input;
use crate::output::print_success;
use biors_core::{
    fasta::parse_fasta_records_reader,
    model_input::{validate_model_input_policy, ModelInputPolicy},
    tokenizer::protein_tokenizer_config_for_profile,
    workflow::{
        prepare_model_input_workflow_with_config, SequenceWorkflowInvocation,
        SequenceWorkflowOutput,
    },
};
use std::path::{Path, PathBuf};

pub(crate) fn run_workflow(
    profile: TokenizerProfileArg,
    max_length: usize,
    pad_token_id: u8,
    padding: PaddingArg,
    path: PathBuf,
) -> Result<(), CliError> {
    let output = workflow_output(
        "biors workflow",
        profile,
        max_length,
        pad_token_id,
        padding,
        path,
    )?;
    let input_hash = output.provenance.input_hash.clone();
    print_success(Some(input_hash), output)
}

pub(crate) fn workflow_output(
    command: &'static str,
    profile: TokenizerProfileArg,
    max_length: usize,
    pad_token_id: u8,
    padding: PaddingArg,
    path: PathBuf,
) -> Result<SequenceWorkflowOutput, CliError> {
    workflow_output_with_invocation_path(
        command,
        profile,
        max_length,
        pad_token_id,
        padding,
        path.clone(),
        path,
    )
}

pub(crate) fn workflow_output_with_invocation_path(
    command: &'static str,
    profile: TokenizerProfileArg,
    max_length: usize,
    pad_token_id: u8,
    padding: PaddingArg,
    path: PathBuf,
    invocation_path: PathBuf,
) -> Result<SequenceWorkflowOutput, CliError> {
    let policy = ModelInputPolicy {
        max_length,
        pad_token_id,
        padding: padding.into(),
    };
    validate_model_input_policy(&policy)?;
    let reader = open_fasta_input(&path)?;
    let input = parse_fasta_records_reader(reader)
        .map_err(|error| CliError::from_fasta_read(path.clone(), error))?;
    let invocation = workflow_invocation(
        command,
        profile,
        max_length,
        pad_token_id,
        padding,
        &invocation_path,
    );
    prepare_model_input_workflow_with_config(
        input.input_hash,
        &input.records,
        policy,
        protein_tokenizer_config_for_profile(profile.into()),
        invocation,
    )
    .map_err(CliError::from)
}

fn workflow_invocation(
    command: &'static str,
    profile: TokenizerProfileArg,
    max_length: usize,
    pad_token_id: u8,
    padding: PaddingArg,
    path: &Path,
) -> SequenceWorkflowInvocation {
    SequenceWorkflowInvocation {
        command: command.to_string(),
        arguments: vec![
            "--max-length".to_string(),
            max_length.to_string(),
            "--profile".to_string(),
            tokenizer_profile_arg_value(profile).to_string(),
            "--pad-token-id".to_string(),
            pad_token_id.to_string(),
            "--padding".to_string(),
            padding_arg_value(padding).to_string(),
            path.to_string_lossy().into_owned(),
        ],
    }
}

fn tokenizer_profile_arg_value(profile: TokenizerProfileArg) -> &'static str {
    match profile {
        TokenizerProfileArg::Protein20 => "protein-20",
        TokenizerProfileArg::Protein20Special => "protein-20-special",
        TokenizerProfileArg::DnaIupac => "dna-iupac",
        TokenizerProfileArg::DnaIupacSpecial => "dna-iupac-special",
        TokenizerProfileArg::RnaIupac => "rna-iupac",
        TokenizerProfileArg::RnaIupacSpecial => "rna-iupac-special",
    }
}

fn padding_arg_value(padding: PaddingArg) -> &'static str {
    match padding {
        PaddingArg::FixedLength => "fixed-length",
        PaddingArg::NoPadding => "no-padding",
    }
}

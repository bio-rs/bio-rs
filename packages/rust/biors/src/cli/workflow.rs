use super::PaddingArg;
use crate::errors::CliError;
use crate::input::open_fasta_input;
use crate::output::print_success;
use biors_core::{
    prepare_protein_model_input_workflow_with_invocation, validate_model_input_policy,
    ModelInputPolicy, SequenceWorkflowInvocation, SequenceWorkflowOutput,
};
use std::path::{Path, PathBuf};

pub(crate) fn run_workflow(
    max_length: usize,
    pad_token_id: u8,
    padding: PaddingArg,
    path: PathBuf,
) -> Result<(), CliError> {
    let output = workflow_output("biors workflow", max_length, pad_token_id, padding, path)?;
    let input_hash = output.provenance.input_hash.clone();
    print_success(Some(input_hash), output)
}

pub(crate) fn workflow_output(
    command: &'static str,
    max_length: usize,
    pad_token_id: u8,
    padding: PaddingArg,
    path: PathBuf,
) -> Result<SequenceWorkflowOutput, CliError> {
    validate_model_input_policy(&ModelInputPolicy {
        max_length,
        pad_token_id,
        padding: padding.into(),
    })?;
    let reader = open_fasta_input(&path)?;
    let input = biors_core::parse_fasta_records_reader(reader)
        .map_err(|error| CliError::from_fasta_read(path.clone(), error))?;
    let invocation = workflow_invocation(command, max_length, pad_token_id, padding, &path);
    prepare_protein_model_input_workflow_with_invocation(
        input.input_hash,
        &input.records,
        ModelInputPolicy {
            max_length,
            pad_token_id,
            padding: padding.into(),
        },
        invocation,
    )
    .map_err(CliError::from)
}

fn workflow_invocation(
    command: &'static str,
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
            "--pad-token-id".to_string(),
            pad_token_id.to_string(),
            "--padding".to_string(),
            padding_arg_value(padding).to_string(),
            path.to_string_lossy().into_owned(),
        ],
    }
}

fn padding_arg_value(padding: PaddingArg) -> &'static str {
    match padding {
        PaddingArg::FixedLength => "fixed_length",
        PaddingArg::NoPadding => "no_padding",
    }
}

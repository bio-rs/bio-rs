use super::{workflow::workflow_output, PaddingArg};
use crate::errors::CliError;
use crate::output::print_success;
use biors_core::{
    model_input::ModelInputRecord, sequence::ValidatedSequence, tokenizer::TokenizedProtein,
    workflow::SequenceWorkflowOutput,
};
use serde::Serialize;
use std::collections::BTreeMap;
use std::path::PathBuf;

pub(crate) fn run_debug(max_length: usize, path: PathBuf) -> Result<(), CliError> {
    let output = workflow_output("biors debug", max_length, 0, PaddingArg::FixedLength, path)?;
    let debug = SequenceDebugOutput::from_workflow(&output);
    print_success(Some(output.provenance.input_hash), debug)
}

#[derive(Debug, Serialize)]
struct SequenceDebugOutput {
    view: &'static str,
    records: Vec<SequenceDebugRecord>,
}

#[derive(Debug, Serialize)]
struct SequenceDebugRecord {
    id: String,
    normalized_sequence: String,
    token_map: Vec<TokenDebugStep>,
    model_input: Option<ModelInputRecord>,
    error_visualization: ErrorVisualization,
}

#[derive(Debug, Serialize)]
struct TokenDebugStep {
    position: usize,
    residue: char,
    token_id: u8,
    status: &'static str,
}

#[derive(Debug, Serialize)]
struct ErrorVisualization {
    sequence: String,
    markers: String,
    legend: &'static str,
}

impl SequenceDebugOutput {
    fn from_workflow(workflow: &SequenceWorkflowOutput) -> Self {
        let model_records: BTreeMap<_, _> = workflow
            .model_input
            .as_ref()
            .map(|input| {
                input
                    .records
                    .iter()
                    .map(|record| (record.id.as_str(), record.clone()))
                    .collect()
            })
            .unwrap_or_default();

        let records = workflow
            .validation
            .sequences
            .iter()
            .zip(workflow.tokenization.records.iter())
            .map(|(validated, tokenized)| SequenceDebugRecord {
                id: validated.id.clone(),
                normalized_sequence: validated.sequence.clone(),
                token_map: token_debug_steps(validated, tokenized),
                model_input: model_records.get(validated.id.as_str()).cloned(),
                error_visualization: error_visualization(validated),
            })
            .collect();

        Self {
            view: "sequence_debug.v0",
            records,
        }
    }
}

fn token_debug_steps(
    validated: &ValidatedSequence,
    tokenized: &TokenizedProtein,
) -> Vec<TokenDebugStep> {
    validated
        .sequence
        .chars()
        .enumerate()
        .map(|(index, residue)| {
            let position = index + 1;
            TokenDebugStep {
                position,
                residue,
                token_id: tokenized.tokens.get(index).copied().unwrap_or_default(),
                status: token_status(position, validated),
            }
        })
        .collect()
}

fn token_status(position: usize, validated: &ValidatedSequence) -> &'static str {
    if validated
        .errors
        .iter()
        .any(|issue| issue.position == position)
    {
        "error"
    } else if validated
        .warnings
        .iter()
        .any(|issue| issue.position == position)
    {
        "warning"
    } else {
        "standard"
    }
}

fn error_visualization(validated: &ValidatedSequence) -> ErrorVisualization {
    let markers: String = validated
        .sequence
        .chars()
        .enumerate()
        .map(|(index, _)| match token_status(index + 1, validated) {
            "error" => 'E',
            "warning" => 'W',
            _ => '.',
        })
        .collect();
    ErrorVisualization {
        sequence: validated.sequence.clone(),
        markers,
        legend: ". standard, W warning, E error",
    }
}

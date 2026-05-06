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
    build_model_inputs_checked, diff_output_bytes, inspect_package_manifest, plan_runtime_bridge,
    prepare_protein_model_input_workflow_with_invocation, summarize_fasta_records_reader,
    tokenize_fasta_records_reader, tokenize_fasta_records_reader_with_config,
    validate_fasta_reader_with_kind_and_hash, validate_model_input_policy,
    validate_package_manifest_artifacts, verify_package_outputs_with_observation_base,
    ModelInputPolicy, ModelInputRecord, ProteinTokenizerConfig, SequenceWorkflowInvocation,
    SequenceWorkflowOutput, TokenizedProtein, ValidatedSequence,
};
use clap::CommandFactory;
use serde::Serialize;
use std::collections::BTreeMap;
use std::fs;
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
            max_length,
            pad_token_id,
            padding,
            path,
        } => run_pipeline(max_length, pad_token_id, padding, path),
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

fn run_diff(expected: PathBuf, observed: PathBuf) -> Result<(), CliError> {
    let expected_bytes = fs::read(&expected).map_err(|source| CliError::Read {
        path: expected.clone(),
        source,
    })?;
    let observed_bytes = fs::read(&observed).map_err(|source| CliError::Read {
        path: observed.clone(),
        source,
    })?;
    let report = diff_output_bytes(
        &expected.display().to_string(),
        &observed.display().to_string(),
        &expected_bytes,
        &observed_bytes,
    );
    print_success(None, report)
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
    let output = workflow_output("biors workflow", max_length, pad_token_id, padding, path)?;
    let input_hash = output.provenance.input_hash.clone();
    print_success(Some(input_hash), output)
}

fn run_pipeline(
    max_length: usize,
    pad_token_id: u8,
    padding: PaddingArg,
    path: PathBuf,
) -> Result<(), CliError> {
    let output = workflow_output("biors pipeline", max_length, pad_token_id, padding, path)?;
    let pipeline = PipelineOutput::from_workflow(output);
    print_success(
        Some(pipeline.workflow.provenance.input_hash.clone()),
        pipeline,
    )
}

fn run_debug(max_length: usize, path: PathBuf) -> Result<(), CliError> {
    let output = workflow_output("biors debug", max_length, 0, PaddingArg::FixedLength, path)?;
    let debug = SequenceDebugOutput::from_workflow(&output);
    print_success(Some(output.provenance.input_hash), debug)
}

fn workflow_output(
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
    path: &std::path::Path,
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

fn run_sequence_validation(path: PathBuf, kind: KindArg) -> Result<(), CliError> {
    let reader = open_fasta_input(&path)?;
    let output = validate_fasta_reader_with_kind_and_hash(reader, kind.into())
        .map_err(|error| CliError::from_fasta_read(path, error))?;
    print_success(Some(output.input_hash), output.report)
}

#[derive(Debug, Serialize)]
struct PipelineOutput {
    pipeline: &'static str,
    ready: bool,
    steps: Vec<PipelineStep>,
    workflow: SequenceWorkflowOutput,
}

#[derive(Debug, Serialize)]
struct PipelineStep {
    name: &'static str,
    status: &'static str,
    records: usize,
    warning_count: usize,
    error_count: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    output_sha256: Option<String>,
}

impl PipelineOutput {
    fn from_workflow(workflow: SequenceWorkflowOutput) -> Self {
        let validation = &workflow.validation;
        let tokenization = &workflow.tokenization.summary;
        let export_status = if workflow.model_ready {
            "passed"
        } else {
            "blocked"
        };
        Self {
            pipeline: "validate_tokenize_export.v0",
            ready: workflow.model_ready,
            steps: vec![
                PipelineStep {
                    name: "validate",
                    status: if validation.error_count == 0 {
                        "passed"
                    } else {
                        "failed"
                    },
                    records: validation.records,
                    warning_count: validation.warning_count,
                    error_count: validation.error_count,
                    output_sha256: None,
                },
                PipelineStep {
                    name: "tokenize",
                    status: if tokenization.error_count == 0 {
                        "passed"
                    } else {
                        "failed"
                    },
                    records: tokenization.records,
                    warning_count: tokenization.warning_count,
                    error_count: tokenization.error_count,
                    output_sha256: None,
                },
                PipelineStep {
                    name: "export",
                    status: export_status,
                    records: workflow
                        .model_input
                        .as_ref()
                        .map(|input| input.records.len())
                        .unwrap_or(0),
                    warning_count: 0,
                    error_count: if workflow.model_ready {
                        0
                    } else {
                        workflow.readiness_issues.len()
                    },
                    output_sha256: workflow
                        .model_ready
                        .then(|| workflow.provenance.hashes.output_data_sha256.clone()),
                },
            ],
            workflow,
        }
    }
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

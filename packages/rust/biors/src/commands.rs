use crate::errors::{classify_validation_code, classify_verification_code, CliError};
use crate::input::{open_fasta_input, read_fixture_observations, read_package_manifest};
use crate::output::print_success;
use biors_core::{
    build_model_inputs_checked, inspect_package_manifest, plan_runtime_bridge,
    summarize_fasta_records_reader, tokenize_fasta_records_reader, validate_fasta_reader_with_hash,
    validate_package_manifest_artifacts, verify_package_outputs_with_observation_base,
    ModelInputPolicy, PaddingPolicy,
};
use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(name = "biors")]
#[command(about = "Rust/WASM tools for biological AI models.")]
pub(crate) struct Cli {
    #[arg(long, global = true, help = "Emit machine-readable JSON errors")]
    pub(crate) json: bool,
    #[command(subcommand)]
    pub(crate) command: Command,
}

#[derive(Debug, Subcommand)]
pub(crate) enum Command {
    Fasta {
        #[command(subcommand)]
        command: FastaCommand,
    },
    Inspect {
        path: PathBuf,
    },
    ModelInput {
        #[arg(long)]
        max_length: usize,
        #[arg(long, default_value_t = 0)]
        pad_token_id: u8,
        #[arg(long, default_value_t = PaddingArg::FixedLength, value_enum)]
        padding: PaddingArg,
        path: PathBuf,
    },
    Package {
        #[command(subcommand)]
        command: PackageCommand,
    },
    Tokenize {
        path: PathBuf,
    },
}

#[derive(Debug, Subcommand)]
pub(crate) enum FastaCommand {
    Validate { path: PathBuf },
}

#[derive(Debug, Subcommand)]
pub(crate) enum PackageCommand {
    Bridge {
        path: PathBuf,
    },
    Inspect {
        path: PathBuf,
    },
    Validate {
        path: PathBuf,
    },
    Verify {
        manifest: PathBuf,
        observations: PathBuf,
    },
}

#[derive(Debug, Clone, Copy, Default, clap::ValueEnum)]
pub(crate) enum PaddingArg {
    #[default]
    FixedLength,
    NoPadding,
}

impl From<PaddingArg> for PaddingPolicy {
    fn from(value: PaddingArg) -> Self {
        match value {
            PaddingArg::FixedLength => Self::FixedLength,
            PaddingArg::NoPadding => Self::NoPadding,
        }
    }
}

pub(crate) fn run(command: Command) -> Result<(), CliError> {
    match command {
        Command::Fasta { command } => match command {
            FastaCommand::Validate { path } => {
                let reader = open_fasta_input(&path)?;
                let output = validate_fasta_reader_with_hash(reader)
                    .map_err(|error| CliError::from_fasta_read(path, error))?;
                print_success(Some(output.input_hash), output.report)?;
            }
        },
        Command::Inspect { path } => {
            let reader = open_fasta_input(&path)?;
            let output = summarize_fasta_records_reader(reader)
                .map_err(|error| CliError::from_fasta_read(path, error))?;
            print_success(Some(output.input_hash), output.summary)?;
        }
        Command::ModelInput {
            max_length,
            pad_token_id,
            padding,
            path,
        } => {
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
            print_success(Some(output.input_hash), model_input)?;
        }
        Command::Package { command } => match command {
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
                print_success(None, report)?;
            }
            PackageCommand::Inspect { path } => {
                let (manifest, _) = read_package_manifest(path)?;
                let summary = inspect_package_manifest(&manifest);
                print_success(None, summary)?;
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
                print_success(None, report)?;
            }
            PackageCommand::Verify {
                manifest,
                observations,
            } => {
                let (manifest, manifest_base_dir) = read_package_manifest(manifest)?;
                let (observations, observations_base_dir) =
                    read_fixture_observations(observations)?;
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
                print_success(None, report)?;
            }
        },
        Command::Tokenize { path } => {
            let reader = open_fasta_input(&path)?;
            let output = tokenize_fasta_records_reader(reader)
                .map_err(|error| CliError::from_fasta_read(path, error))?;
            print_success(Some(output.input_hash), output.records)?;
        }
    }

    Ok(())
}

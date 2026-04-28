use biors_core::{
    build_model_inputs_checked, inspect_package_manifest, plan_runtime_bridge, stable_input_hash,
    summarize_tokenized_proteins, tokenize_fasta_records, validate_fasta_input,
    validate_package_manifest_artifacts, verify_package_outputs_with_observation_base, BioRsError,
    ErrorLocation, FixtureObservation, ModelInputBuildError, ModelInputPolicy, PackageManifest,
    PaddingPolicy,
};
use clap::{Parser, Subcommand};
use serde::Serialize;
use std::fs;
use std::io::{self, Read};
use std::path::PathBuf;

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Debug, Parser)]
#[command(name = "biors")]
#[command(about = "Rust/WASM tools for biological AI models.")]
struct Cli {
    #[arg(long, global = true, help = "Emit machine-readable JSON errors")]
    json: bool,
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
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
enum FastaCommand {
    Validate { path: PathBuf },
}

#[derive(Debug, Subcommand)]
enum PackageCommand {
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

fn main() {
    let cli = Cli::parse();
    if let Err(error) = run(cli.command) {
        let exit_code = error.exit_code();
        if cli.json {
            let payload = CliFailure::from(error);
            println!("{}", to_json(&payload).expect("serialize JSON error"));
        } else {
            eprintln!("error: {error}");
        }
        std::process::exit(exit_code);
    }
}

fn run(command: Command) -> Result<(), CliError> {
    match command {
        Command::Fasta { command } => match command {
            FastaCommand::Validate { path } => {
                let input = read_input(path)?;
                let report = validate_fasta_input(&input)?;
                print_success(Some(stable_input_hash(&input)), report)?;
            }
        },
        Command::Inspect { path } => {
            let input = read_input(path)?;
            let tokenized = tokenize_fasta_records(&input)?;
            let summary = summarize_tokenized_proteins(&tokenized);
            print_success(Some(stable_input_hash(&input)), summary)?;
        }
        Command::ModelInput {
            max_length,
            pad_token_id,
            padding,
            path,
        } => {
            let input = read_input(path)?;
            let tokenized = tokenize_fasta_records(&input)?;
            let model_input = build_model_inputs_checked(
                &tokenized,
                ModelInputPolicy {
                    max_length,
                    pad_token_id,
                    padding: padding.into(),
                },
            )?;
            print_success(Some(stable_input_hash(&input)), model_input)?;
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
                        code: classify_validation_code(&report.issues),
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
            let input = read_input(path)?;
            let tokenized = tokenize_fasta_records(&input)?;
            print_success(Some(stable_input_hash(&input)), tokenized)?;
        }
    }

    Ok(())
}

#[derive(Debug, Clone, Copy, Default, clap::ValueEnum)]
enum PaddingArg {
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

fn print_success<T: Serialize>(input_hash: Option<String>, data: T) -> Result<(), CliError> {
    let payload = CliSuccess {
        ok: true,
        biors_version: VERSION,
        input_hash,
        data,
    };
    println!("{}", to_json(&payload)?);
    Ok(())
}

fn read_input(path: PathBuf) -> Result<String, CliError> {
    if path.as_os_str() == "-" {
        let mut input = String::new();
        io::stdin()
            .read_to_string(&mut input)
            .map_err(|source| CliError::Read { path, source })?;
        return Ok(input);
    }

    fs::read_to_string(&path).map_err(|source| CliError::Read { path, source })
}

fn read_package_manifest(path: PathBuf) -> Result<(PackageManifest, PathBuf), CliError> {
    let (input, base_dir) = read_input_with_base_dir(path)?;
    Ok((
        serde_json::from_str(&input).map_err(CliError::Json)?,
        base_dir,
    ))
}

fn read_fixture_observations(
    path: PathBuf,
) -> Result<(Vec<FixtureObservation>, PathBuf), CliError> {
    let (input, base_dir) = read_input_with_base_dir(path)?;
    Ok((
        serde_json::from_str(&input).map_err(CliError::Json)?,
        base_dir,
    ))
}

fn to_json<T: Serialize>(value: &T) -> Result<String, serde_json::Error> {
    serde_json::to_string_pretty(value)
}

fn read_input_with_base_dir(path: PathBuf) -> Result<(String, PathBuf), CliError> {
    if path.as_os_str() == "-" {
        return Ok((
            read_input(path)?,
            std::env::current_dir().map_err(CliError::CurrentDir)?,
        ));
    }

    let base_dir = path
        .parent()
        .map(std::path::Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."));
    Ok((read_input(path)?, base_dir))
}

#[derive(Debug, Serialize)]
struct CliSuccess<T: Serialize> {
    ok: bool,
    biors_version: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    input_hash: Option<String>,
    data: T,
}

#[derive(Debug, Serialize)]
struct CliFailure {
    ok: bool,
    error: CliErrorBody,
}

impl From<CliError> for CliFailure {
    fn from(error: CliError) -> Self {
        Self {
            ok: false,
            error: CliErrorBody {
                code: error.code(),
                message: error.to_string(),
                location: error.location(),
            },
        }
    }
}

#[derive(Debug, Serialize)]
struct CliErrorBody {
    code: &'static str,
    message: String,
    location: Option<ErrorLocationValue>,
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
enum ErrorLocationValue {
    Core(ErrorLocation),
    Label(String),
}

#[derive(Debug)]
enum CliError {
    Core(BioRsError),
    ModelInput(ModelInputBuildError),
    Json(serde_json::Error),
    CurrentDir(std::io::Error),
    Read {
        path: PathBuf,
        source: std::io::Error,
    },
    Serialization(serde_json::Error),
    Validation {
        code: &'static str,
        message: String,
        location: Option<String>,
    },
}

impl CliError {
    const fn code(&self) -> &'static str {
        match self {
            Self::Core(error) => error.code(),
            Self::ModelInput(ModelInputBuildError::InvalidPolicy { .. }) => {
                "model_input.invalid_policy"
            }
            Self::ModelInput(ModelInputBuildError::InvalidTokenizedSequence { .. }) => {
                "model_input.invalid_sequence"
            }
            Self::Json(_) => "json.invalid",
            Self::CurrentDir(_) => "io.read_failed",
            Self::Read { .. } => "io.read_failed",
            Self::Serialization(_) => "json.serialization_failed",
            Self::Validation { code, .. } => code,
        }
    }

    fn location(&self) -> Option<ErrorLocationValue> {
        match self {
            Self::Core(error) => error.location().map(ErrorLocationValue::Core),
            Self::ModelInput(ModelInputBuildError::InvalidPolicy { .. }) => None,
            Self::ModelInput(ModelInputBuildError::InvalidTokenizedSequence { id, .. }) => {
                Some(ErrorLocationValue::Label(id.clone()))
            }
            Self::Read { path, .. } => Some(ErrorLocationValue::Label(path.display().to_string())),
            Self::Validation { location, .. } => location.clone().map(ErrorLocationValue::Label),
            Self::Json(_) | Self::CurrentDir(_) | Self::Serialization(_) => None,
        }
    }

    const fn exit_code(&self) -> i32 {
        match self {
            Self::Core(_) | Self::ModelInput(_) | Self::Json(_) | Self::Validation { .. } => 2,
            Self::Read { .. } | Self::CurrentDir(_) | Self::Serialization(_) => 1,
        }
    }
}

impl std::fmt::Display for CliError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Core(error) => write!(f, "{error}"),
            Self::ModelInput(error) => write!(f, "{error}"),
            Self::Json(error) => write!(f, "{error}"),
            Self::CurrentDir(error) => write!(f, "failed to determine current directory: {error}"),
            Self::Read { path, source } => {
                write!(f, "failed to read '{}': {source}", path.display())
            }
            Self::Serialization(error) => write!(f, "{error}"),
            Self::Validation { message, .. } => write!(f, "{message}"),
        }
    }
}

impl std::error::Error for CliError {}

impl From<BioRsError> for CliError {
    fn from(error: BioRsError) -> Self {
        Self::Core(error)
    }
}

impl From<ModelInputBuildError> for CliError {
    fn from(error: ModelInputBuildError) -> Self {
        Self::ModelInput(error)
    }
}

impl From<serde_json::Error> for CliError {
    fn from(error: serde_json::Error) -> Self {
        Self::Serialization(error)
    }
}

fn classify_validation_code(issues: &[String]) -> &'static str {
    if issues
        .iter()
        .any(|issue| issue.contains(".checksum must use sha256:"))
    {
        "package.invalid_checksum_format"
    } else if issues
        .iter()
        .any(|issue| issue.contains("checksum mismatch"))
    {
        "package.checksum_mismatch"
    } else if issues
        .iter()
        .any(|issue| issue.contains("must be relative") || issue.contains("must stay inside"))
    {
        "package.invalid_asset_path"
    } else if issues
        .iter()
        .any(|issue| issue.contains("failed to read asset"))
    {
        "package.asset_read_failed"
    } else {
        "package.validation_failed"
    }
}

fn classify_verification_code(report: &biors_core::PackageVerificationReport) -> &'static str {
    if report
        .results
        .iter()
        .any(|result| matches!(result.status, biors_core::VerificationStatus::Missing))
    {
        "package.observed_output_missing"
    } else if report.results.iter().any(|result| result.checksum_mismatch) {
        "package.checksum_mismatch"
    } else if report.results.iter().any(|result| result.content_mismatch) {
        "package.output_content_mismatch"
    } else {
        "package.verification_failed"
    }
}

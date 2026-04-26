use biors_core::{
    inspect_package_manifest, plan_runtime_bridge, stable_input_hash, summarize_tokenized_proteins,
    tokenize_fasta_records, validate_fasta_input, validate_package_manifest,
    verify_package_outputs, BioRsError, ErrorLocation, FixtureObservation, PackageManifest,
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
        Command::Package { command } => match command {
            PackageCommand::Bridge { path } => {
                let manifest = read_package_manifest(path)?;
                let report = plan_runtime_bridge(&manifest);
                if !report.ready {
                    return Err(CliError::Validation {
                        code: "package.bridge_not_ready",
                        message: format!("{:?}", report.blocking_issues),
                        location: Some("manifest".to_string()),
                    });
                }
                print_success(None, report)?;
            }
            PackageCommand::Inspect { path } => {
                let manifest = read_package_manifest(path)?;
                let summary = inspect_package_manifest(&manifest);
                print_success(None, summary)?;
            }
            PackageCommand::Validate { path } => {
                let manifest = read_package_manifest(path)?;
                let report = validate_package_manifest(&manifest);
                if !report.valid {
                    return Err(CliError::Validation {
                        code: "package.validation_failed",
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
                let manifest = read_package_manifest(manifest)?;
                let observations = read_fixture_observations(observations)?;
                let report = verify_package_outputs(&manifest, &observations);
                if report.failed > 0 {
                    return Err(CliError::Validation {
                        code: "package.verification_failed",
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

fn read_package_manifest(path: PathBuf) -> Result<PackageManifest, CliError> {
    let input = read_input(path)?;
    serde_json::from_str(&input).map_err(CliError::Json)
}

fn read_fixture_observations(path: PathBuf) -> Result<Vec<FixtureObservation>, CliError> {
    let input = read_input(path)?;
    serde_json::from_str(&input).map_err(CliError::Json)
}

fn to_json<T: Serialize>(value: &T) -> Result<String, serde_json::Error> {
    serde_json::to_string_pretty(value)
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
    Json(serde_json::Error),
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
            Self::Json(_) => "json.invalid",
            Self::Read { .. } => "io.read_failed",
            Self::Serialization(_) => "json.serialization_failed",
            Self::Validation { code, .. } => code,
        }
    }

    fn location(&self) -> Option<ErrorLocationValue> {
        match self {
            Self::Core(error) => error.location().map(ErrorLocationValue::Core),
            Self::Read { path, .. } => Some(ErrorLocationValue::Label(path.display().to_string())),
            Self::Validation { location, .. } => location.clone().map(ErrorLocationValue::Label),
            Self::Json(_) | Self::Serialization(_) => None,
        }
    }

    const fn exit_code(&self) -> i32 {
        match self {
            Self::Core(_) | Self::Json(_) | Self::Validation { .. } => 2,
            Self::Read { .. } | Self::Serialization(_) => 1,
        }
    }
}

impl std::fmt::Display for CliError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Core(error) => write!(f, "{error}"),
            Self::Json(error) => write!(f, "{error}"),
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

impl From<serde_json::Error> for CliError {
    fn from(error: serde_json::Error) -> Self {
        Self::Serialization(error)
    }
}

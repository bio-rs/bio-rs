use biors_core::{
    inspect_package_manifest, summarize_tokenized_proteins, tokenize_fasta_records,
    validate_package_manifest, BioRsError, PackageManifest,
};
use clap::{Parser, Subcommand};
use std::fs;
use std::io::{self, Read};
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(name = "biors")]
#[command(about = "Rust/WASM tools for biological AI models.")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
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
enum PackageCommand {
    Inspect { path: PathBuf },
    Validate { path: PathBuf },
}

fn main() {
    if let Err(error) = run() {
        eprintln!("error: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), CliError> {
    let cli = Cli::parse();

    match cli.command {
        Command::Inspect { path } => {
            let input = read_input(path)?;
            let tokenized = tokenize_fasta_records(&input)?;
            let summary = summarize_tokenized_proteins(&tokenized);

            let json = serde_json::to_string_pretty(&summary)?;
            println!("{json}");
        }
        Command::Package { command } => match command {
            PackageCommand::Inspect { path } => {
                let manifest = read_package_manifest(path)?;
                let summary = inspect_package_manifest(&manifest);

                let json = serde_json::to_string_pretty(&summary)?;
                println!("{json}");
            }
            PackageCommand::Validate { path } => {
                let manifest = read_package_manifest(path)?;
                let report = validate_package_manifest(&manifest);

                let json = serde_json::to_string_pretty(&report)?;
                println!("{json}");

                if !report.valid {
                    return Err(CliError::Validation(report.issues));
                }
            }
        },
        Command::Tokenize { path } => {
            let input = read_input(path)?;
            let tokenized = tokenize_fasta_records(&input)?;

            let json = serde_json::to_string_pretty(&tokenized)?;
            println!("{json}");
        }
    }

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

#[derive(Debug)]
enum CliError {
    Core(BioRsError),
    Json(serde_json::Error),
    Read {
        path: PathBuf,
        source: std::io::Error,
    },
    Validation(Vec<String>),
}

impl std::fmt::Display for CliError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Core(error) => write!(f, "{error}"),
            Self::Json(error) => write!(f, "{error}"),
            Self::Read { path, source } => {
                write!(f, "failed to read '{}': {source}", path.display())
            }
            Self::Validation(issues) => write!(f, "package manifest is invalid: {issues:?}"),
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
        Self::Json(error)
    }
}

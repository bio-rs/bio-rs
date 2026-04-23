use biors_core::{parse_fasta, tokenize_fasta, BioRsError};
use clap::{Parser, Subcommand, ValueEnum};
use std::fs;
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
    Tokenize {
        path: PathBuf,
        #[arg(long, value_enum, default_value_t = OutputFormat::Json)]
        format: OutputFormat,
    },
}

#[derive(Clone, Debug, ValueEnum)]
enum OutputFormat {
    Json,
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
            let input =
                fs::read_to_string(&path).map_err(|source| CliError::Read { path, source })?;
            let protein = parse_fasta(&input)?;
            let tokenized = biors_core::tokenize_protein(&protein);

            println!("id: {}", tokenized.id);
            println!("length: {}", tokenized.length);
            println!("alphabet: {}", tokenized.alphabet);
            println!("valid: {}", tokenized.valid);
            println!("warnings: {}", tokenized.warnings.len());
            println!("errors: {}", tokenized.errors.len());
        }
        Command::Tokenize { path, format } => {
            let input =
                fs::read_to_string(&path).map_err(|source| CliError::Read { path, source })?;
            let tokenized = tokenize_fasta(&input)?;

            match format {
                OutputFormat::Json => {
                    let json = serde_json::to_string_pretty(&tokenized)?;
                    println!("{json}");
                }
            }
        }
    }

    Ok(())
}

#[derive(Debug)]
enum CliError {
    Core(BioRsError),
    Json(serde_json::Error),
    Read {
        path: PathBuf,
        source: std::io::Error,
    },
}

impl std::fmt::Display for CliError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Core(error) => write!(f, "{error}"),
            Self::Json(error) => write!(f, "{error}"),
            Self::Read { path, source } => {
                write!(f, "failed to read '{}': {source}", path.display())
            }
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

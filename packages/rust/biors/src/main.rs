use biors_core::{summarize_tokenized_proteins, tokenize_fasta_records, BioRsError};
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
    Inspect { path: PathBuf },
    Tokenize { path: PathBuf },
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

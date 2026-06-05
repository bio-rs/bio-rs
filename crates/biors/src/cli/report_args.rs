use clap::Subcommand;
use std::path::PathBuf;

#[derive(Debug, Subcommand)]
pub enum ReportCommand {
    Generate {
        input: PathBuf,
        #[arg(long)]
        output: Option<PathBuf>,
        #[arg(long)]
        shareable_json: Option<PathBuf>,
    },
}

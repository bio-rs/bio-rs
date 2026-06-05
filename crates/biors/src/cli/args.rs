use super::molecule_args::MoleculeCommand;
use super::package_args::PackageCommand;
use super::profile_args::TokenizerProfileArg;
use super::report_args::ReportCommand;
use super::serve_args::ServeArgs;
use super::service_args::ServiceCommand;
use super::structure_args::StructureCommand;
use super::template_args::TemplateCommand;
use biors_core::{
    formats::BioFormat,
    model_input::PaddingPolicy,
    sequence::{SequenceKind, SequenceKindSelection},
};
use clap::{Parser, Subcommand};
use clap_complete::Shell;
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(name = "biors")]
#[command(about = "Rust/WASM tools for biological AI models.")]
#[command(version)]
pub struct Cli {
    #[arg(long, global = true, help = "Emit machine-readable JSON errors")]
    pub json: bool,
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    Batch {
        #[command(subcommand)]
        command: BatchCommand,
    },
    Cache {
        #[command(subcommand)]
        command: CacheCommand,
    },
    Completions {
        #[arg(value_enum)]
        shell: Shell,
    },
    Dataset {
        #[command(subcommand)]
        command: DatasetCommand,
    },
    Debug {
        #[arg(long)]
        max_length: usize,
        path: PathBuf,
    },
    Diff {
        expected: PathBuf,
        observed: PathBuf,
    },
    Doctor,
    Fasta {
        #[command(subcommand)]
        command: FastaCommand,
    },
    Formats {
        #[command(subcommand)]
        command: FormatsCommand,
    },
    Inspect {
        path: PathBuf,
    },
    ModelInput {
        #[arg(long, value_enum, default_value_t = TokenizerProfileArg::Protein20)]
        profile: TokenizerProfileArg,
        #[arg(long)]
        max_length: usize,
        #[arg(long, default_value_t = 0)]
        pad_token_id: u8,
        #[arg(long, default_value_t = PaddingArg::FixedLength, value_enum)]
        padding: PaddingArg,
        path: PathBuf,
    },
    Molecule {
        #[command(subcommand)]
        command: MoleculeCommand,
    },
    Package {
        #[command(subcommand)]
        command: PackageCommand,
    },
    Pipeline {
        #[arg(long)]
        config: Option<PathBuf>,
        #[arg(long)]
        dry_run: bool,
        #[arg(long)]
        explain_plan: bool,
        #[arg(long)]
        package: Option<PathBuf>,
        #[arg(long)]
        write_lock: Option<PathBuf>,
        #[arg(long)]
        max_length: Option<usize>,
        #[arg(long, default_value_t = 0)]
        pad_token_id: u8,
        #[arg(long, default_value_t = PaddingArg::FixedLength, value_enum)]
        padding: PaddingArg,
        path: Option<PathBuf>,
    },
    Seq {
        #[command(subcommand)]
        command: SeqCommand,
    },
    Report {
        #[command(subcommand)]
        command: ReportCommand,
    },
    Serve(ServeArgs),
    Service {
        #[command(subcommand)]
        command: ServiceCommand,
    },
    Structure {
        #[command(subcommand)]
        command: StructureCommand,
    },
    Templates {
        #[command(subcommand)]
        command: TemplateCommand,
    },
    Tokenize {
        #[arg(long, value_enum, default_value_t = TokenizerProfileArg::Protein20)]
        profile: TokenizerProfileArg,
        #[arg(long)]
        config: Option<PathBuf>,
        path: PathBuf,
    },
    Tokenizer {
        #[command(subcommand)]
        command: TokenizerCommand,
    },
    Workflow {
        #[arg(long, value_enum, default_value_t = TokenizerProfileArg::Protein20)]
        profile: TokenizerProfileArg,
        #[arg(long)]
        max_length: usize,
        #[arg(long, default_value_t = 0)]
        pad_token_id: u8,
        #[arg(long, default_value_t = PaddingArg::FixedLength, value_enum)]
        padding: PaddingArg,
        path: PathBuf,
    },
}

#[derive(Debug, Subcommand)]
pub enum TokenizerCommand {
    ConvertHf {
        path: PathBuf,
        #[arg(long)]
        output: Option<PathBuf>,
    },
    Inspect {
        #[arg(long, value_enum, default_value_t = TokenizerProfileArg::Protein20)]
        profile: TokenizerProfileArg,
        #[arg(long)]
        config: Option<PathBuf>,
    },
}

#[derive(Debug, Subcommand)]
pub enum BatchCommand {
    Validate {
        #[arg(long, default_value_t = KindArg::Auto, value_enum)]
        kind: KindArg,
        #[arg(required = true)]
        inputs: Vec<PathBuf>,
    },
}

#[derive(Debug, Subcommand)]
pub enum CacheCommand {
    Clean {
        #[arg(long)]
        root: Option<PathBuf>,
        #[arg(long)]
        dry_run: bool,
        #[arg(long)]
        yes: bool,
    },
    Inspect {
        #[arg(long)]
        root: Option<PathBuf>,
    },
}

#[derive(Debug, Subcommand)]
pub enum FastaCommand {
    Validate {
        #[arg(long, default_value_t = KindArg::Protein, value_enum)]
        kind: KindArg,
        path: PathBuf,
    },
}

#[derive(Debug, Subcommand)]
pub enum FormatsCommand {
    List,
    Validate {
        #[arg(long, value_enum)]
        format: FormatArg,
        path: PathBuf,
    },
}

#[derive(Debug, Subcommand)]
pub enum DatasetCommand {
    Inspect {
        #[arg(long, default_value = "local")]
        source: String,
        #[arg(long, default_value = "unversioned")]
        version: String,
        #[arg(long, default_value = "unspecified")]
        split: String,
        #[arg(long = "metadata")]
        metadata: Vec<String>,
        #[arg(required = true)]
        inputs: Vec<PathBuf>,
    },
}

#[derive(Debug, Subcommand)]
pub enum SeqCommand {
    Validate {
        #[arg(long, default_value_t = KindArg::Auto, value_enum)]
        kind: KindArg,
        path: PathBuf,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum FormatArg {
    Fastq,
}

impl From<FormatArg> for BioFormat {
    fn from(value: FormatArg) -> Self {
        match value {
            FormatArg::Fastq => BioFormat::Fastq,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, clap::ValueEnum)]
pub enum PaddingArg {
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

#[derive(Debug, Clone, Copy, Default, clap::ValueEnum)]
pub enum KindArg {
    Auto,
    #[default]
    Protein,
    Dna,
    Rna,
}

impl From<KindArg> for SequenceKindSelection {
    fn from(value: KindArg) -> Self {
        match value {
            KindArg::Auto => Self::Auto,
            KindArg::Protein => Self::Explicit(SequenceKind::Protein),
            KindArg::Dna => Self::Explicit(SequenceKind::Dna),
            KindArg::Rna => Self::Explicit(SequenceKind::Rna),
        }
    }
}

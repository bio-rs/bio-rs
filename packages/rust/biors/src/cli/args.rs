use biors_core::{
    model_input::PaddingPolicy,
    package::SchemaVersion,
    sequence::{SequenceKind, SequenceKindSelection},
    tokenizer::ProteinTokenizerProfile,
};
use clap::{Args, Parser, Subcommand};
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
    Completions {
        #[arg(value_enum)]
        shell: Shell,
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
pub enum FastaCommand {
    Validate {
        #[arg(long, default_value_t = KindArg::Protein, value_enum)]
        kind: KindArg,
        path: PathBuf,
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

#[derive(Debug, Subcommand)]
pub enum PackageCommand {
    Bridge {
        path: PathBuf,
    },
    Compatibility {
        left: PathBuf,
        right: PathBuf,
    },
    Convert(Box<PackageConvertArgs>),
    Diff {
        left: PathBuf,
        right: PathBuf,
    },
    Inspect {
        path: PathBuf,
    },
    Migrate {
        path: PathBuf,
        #[arg(long, value_enum, default_value = "biors.package.v1")]
        to: PackageSchemaArg,
    },
    Validate {
        path: PathBuf,
    },
    Verify {
        manifest: PathBuf,
        observations: PathBuf,
    },
}

#[derive(Debug, Args)]
pub struct PackageConvertArgs {
    pub path: PathBuf,
    #[arg(long, value_enum, default_value = "biors.package.v1")]
    pub to: PackageSchemaArg,
    #[arg(long)]
    pub output: Option<PathBuf>,
    #[arg(long)]
    pub license: Option<String>,
    #[arg(long)]
    pub citation: Option<String>,
    #[arg(long)]
    pub doi: Option<String>,
    #[arg(long = "model-card")]
    pub model_card: Option<String>,
    #[arg(long = "model-card-summary")]
    pub model_card_summary: Option<String>,
    #[arg(long = "intended-use")]
    pub intended_use: Vec<String>,
    #[arg(long = "limitation")]
    pub limitations: Vec<String>,
    #[arg(long = "license-file")]
    pub license_file: Option<String>,
    #[arg(long = "citation-file")]
    pub citation_file: Option<String>,
    #[arg(long = "models-dir")]
    pub models_dir: Option<String>,
    #[arg(long = "tokenizers-dir")]
    pub tokenizers_dir: Option<String>,
    #[arg(long = "vocabs-dir")]
    pub vocabs_dir: Option<String>,
    #[arg(long = "pipelines-dir")]
    pub pipelines_dir: Option<String>,
    #[arg(long = "fixtures-dir")]
    pub fixtures_dir: Option<String>,
    #[arg(long = "observed-dir")]
    pub observed_dir: Option<String>,
    #[arg(long = "docs-dir")]
    pub docs_dir: Option<String>,
}

#[derive(Debug, Clone, Copy, clap::ValueEnum)]
pub enum PackageSchemaArg {
    #[value(name = "biors.package.v0")]
    BiorsPackageV0,
    #[value(name = "biors.package.v1")]
    BiorsPackageV1,
}

impl From<PackageSchemaArg> for SchemaVersion {
    fn from(value: PackageSchemaArg) -> Self {
        match value {
            PackageSchemaArg::BiorsPackageV0 => Self::BiorsPackageV0,
            PackageSchemaArg::BiorsPackageV1 => Self::BiorsPackageV1,
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
pub enum TokenizerProfileArg {
    #[default]
    #[value(name = "protein-20")]
    Protein20,
    #[value(name = "protein-20-special")]
    Protein20Special,
}

impl From<TokenizerProfileArg> for ProteinTokenizerProfile {
    fn from(value: TokenizerProfileArg) -> Self {
        match value {
            TokenizerProfileArg::Protein20 => Self::Protein20,
            TokenizerProfileArg::Protein20Special => Self::Protein20Special,
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

use biors_core::{
    model_input::PaddingPolicy,
    sequence::{SequenceKind, SequenceKindSelection},
    tokenizer::ProteinTokenizerProfile,
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
        max_length: usize,
        #[arg(long, default_value_t = 0)]
        pad_token_id: u8,
        #[arg(long, default_value_t = PaddingArg::FixedLength, value_enum)]
        padding: PaddingArg,
        path: PathBuf,
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

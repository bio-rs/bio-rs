use biors_core::formats::BioFormat;
use clap::Subcommand;
use std::path::PathBuf;

#[derive(Debug, Subcommand)]
pub enum StructureCommand {
    Validate {
        #[arg(long, value_enum)]
        format: StructureFormatArg,
        path: PathBuf,
    },
    Sequence {
        #[arg(long, value_enum)]
        format: StructureFormatArg,
        path: PathBuf,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum StructureFormatArg {
    Pdb,
}

impl From<StructureFormatArg> for BioFormat {
    fn from(value: StructureFormatArg) -> Self {
        match value {
            StructureFormatArg::Pdb => BioFormat::Pdb,
        }
    }
}

use biors_core::formats::BioFormat;
use clap::Subcommand;
use std::path::PathBuf;

#[derive(Debug, Subcommand)]
pub enum MoleculeCommand {
    Validate {
        #[arg(long, value_enum)]
        format: MoleculeFormatArg,
        path: PathBuf,
    },
    Inspect {
        #[arg(long, value_enum)]
        format: MoleculeFormatArg,
        path: PathBuf,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum MoleculeFormatArg {
    Smiles,
    Sdf,
    Mol2,
}

impl From<MoleculeFormatArg> for BioFormat {
    fn from(value: MoleculeFormatArg) -> Self {
        match value {
            MoleculeFormatArg::Smiles => BioFormat::Smiles,
            MoleculeFormatArg::Sdf => BioFormat::Sdf,
            MoleculeFormatArg::Mol2 => BioFormat::Mol2,
        }
    }
}

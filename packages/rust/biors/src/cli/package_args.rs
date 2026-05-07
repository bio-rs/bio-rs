use biors_core::package::SchemaVersion;
use clap::{Args, Subcommand};
use std::path::PathBuf;

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
    ConvertProject(Box<PackageConvertProjectArgs>),
    Diff {
        left: PathBuf,
        right: PathBuf,
    },
    Init(Box<PackageInitArgs>),
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

#[derive(Debug, Args)]
pub struct PackageInitArgs {
    pub output_dir: PathBuf,
    #[arg(long)]
    pub name: String,
    #[arg(long)]
    pub model: PathBuf,
    #[arg(long = "tokenizer-config")]
    pub tokenizer_config: Option<PathBuf>,
    #[arg(long = "fixture-input")]
    pub fixture_input: PathBuf,
    #[arg(long = "fixture-output")]
    pub fixture_output: PathBuf,
    #[arg(long)]
    pub license: String,
    #[arg(long)]
    pub citation: String,
    #[arg(long)]
    pub doi: Option<String>,
    #[arg(long = "model-card-summary")]
    pub model_card_summary: String,
    #[arg(long = "intended-use")]
    pub intended_use: Vec<String>,
    #[arg(long = "limitation")]
    pub limitations: Vec<String>,
    #[arg(long)]
    pub force: bool,
}

#[derive(Debug, Args)]
pub struct PackageConvertProjectArgs {
    pub project_dir: PathBuf,
    #[arg(long)]
    pub output: PathBuf,
    #[arg(long)]
    pub name: String,
    #[arg(long)]
    pub model: Option<PathBuf>,
    #[arg(long = "tokenizer-config")]
    pub tokenizer_config: Option<PathBuf>,
    #[arg(long = "fixture-input")]
    pub fixture_input: PathBuf,
    #[arg(long = "fixture-output")]
    pub fixture_output: PathBuf,
    #[arg(long)]
    pub license: String,
    #[arg(long)]
    pub citation: String,
    #[arg(long)]
    pub doi: Option<String>,
    #[arg(long = "model-card-summary")]
    pub model_card_summary: String,
    #[arg(long = "intended-use")]
    pub intended_use: Vec<String>,
    #[arg(long = "limitation")]
    pub limitations: Vec<String>,
    #[arg(long)]
    pub force: bool,
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

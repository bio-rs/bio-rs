use clap::Subcommand;

#[derive(Debug, Subcommand)]
pub enum ServiceCommand {
    Contract,
}

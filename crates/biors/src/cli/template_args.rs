use clap::Subcommand;

#[derive(Debug, Subcommand)]
pub enum TemplateCommand {
    List,
    Show { id: String },
}

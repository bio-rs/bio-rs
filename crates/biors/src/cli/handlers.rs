mod formats;
mod molecule;
mod sequence;
mod structure;
mod tokenizer;

use self::{
    formats::run_formats_command,
    molecule::run_molecule_command,
    sequence::{run_fasta_command, run_inspect, run_model_input, run_seq_command, run_tokenize},
    structure::run_structure_command,
    tokenizer::run_tokenizer_command,
};
use super::{Cli, Command, ServiceCommand};
use crate::cli::{
    build_doctor_report, run_batch_command, run_dataset_command, run_debug, run_diff,
    run_package_command, run_pipeline, run_report_command, run_serve, run_workflow,
    PipelineRunOptions,
};
use crate::errors::CliError;
use crate::output::print_success;
use clap::CommandFactory;

pub fn run(command: Command) -> Result<(), CliError> {
    match command {
        Command::Batch { command } => run_batch_command(command),
        Command::Completions { shell } => run_completions(shell),
        Command::Dataset { command } => run_dataset_command(command),
        Command::Debug { max_length, path } => run_debug(max_length, path),
        Command::Diff { expected, observed } => run_diff(expected, observed),
        Command::Doctor => run_doctor(),
        Command::Fasta { command } => run_fasta_command(command),
        Command::Formats { command } => run_formats_command(command),
        Command::Inspect { path } => run_inspect(path),
        Command::ModelInput {
            profile,
            max_length,
            pad_token_id,
            padding,
            path,
        } => run_model_input(profile, max_length, pad_token_id, padding, path),
        Command::Molecule { command } => run_molecule_command(command),
        Command::Package { command } => run_package_command(command),
        Command::Pipeline {
            config,
            dry_run,
            explain_plan,
            package,
            write_lock,
            max_length,
            pad_token_id,
            padding,
            path,
        } => run_pipeline(PipelineRunOptions {
            config,
            dry_run,
            explain_plan,
            package,
            write_lock,
            max_length,
            pad_token_id,
            padding,
            path,
        }),
        Command::Seq { command } => run_seq_command(command),
        Command::Report { command } => run_report_command(command),
        Command::Serve(args) => run_serve(args),
        Command::Service { command } => run_service_command(command),
        Command::Structure { command } => run_structure_command(command),
        Command::Tokenize {
            profile,
            config,
            path,
        } => run_tokenize(profile, config, path),
        Command::Tokenizer { command } => run_tokenizer_command(command),
        Command::Workflow {
            profile,
            max_length,
            pad_token_id,
            padding,
            path,
        } => run_workflow(profile, max_length, pad_token_id, padding, path),
    }
}

fn run_completions(shell: clap_complete::Shell) -> Result<(), CliError> {
    let mut command = Cli::command();
    let name = command.get_name().to_string();
    clap_complete::generate(shell, &mut command, name, &mut std::io::stdout());
    Ok(())
}

fn run_doctor() -> Result<(), CliError> {
    print_success(None, build_doctor_report())
}

fn run_service_command(command: ServiceCommand) -> Result<(), CliError> {
    match command {
        ServiceCommand::Contract => print_success(
            None,
            biors_core::service::current_service_interface_document(),
        ),
    }
}

use super::{
    Cli, Command, FastaCommand, FormatArg, FormatsCommand, KindArg, MoleculeCommand,
    MoleculeFormatArg, PaddingArg, SeqCommand, ServiceCommand, StructureCommand,
    StructureFormatArg, TokenizerCommand, TokenizerProfileArg,
};
use crate::cli::{
    build_doctor_report, run_batch_command, run_cache_command, run_dataset_command, run_debug,
    run_diff, run_package_command, run_pipeline, run_report_command, run_serve,
    run_template_command, run_workflow, PipelineRunOptions,
};
use crate::errors::CliError;
use crate::input::{open_buffered_input, open_fasta_input, read_tokenizer_config};
use crate::output::print_success;
use biors_core::{
    formats::{format_capabilities, validate_fastq_reader_with_hash},
    model_input::{build_model_inputs_checked, ModelInputPolicy},
    molecule::{
        parse_mol2_records_reader, parse_sdf_records_reader, parse_smiles_records_reader,
        validate_molecule_records, validate_smiles_reader_with_hash,
    },
    sequence::validate_fasta_reader_with_kind_and_hash,
    structure::{
        extract_structure_sequences, parse_pdb_record_reader, validate_pdb_reader_with_hash,
    },
    tokenizer::{
        inspect_protein_tokenizer_config, protein_tokenizer_config_for_profile,
        summarize_fasta_records_reader, tokenize_fasta_records_reader_with_config,
        ProteinTokenizerConfig,
    },
};
use clap::CommandFactory;
use std::path::PathBuf;

pub fn run(command: Command) -> Result<(), CliError> {
    match command {
        Command::Batch { command } => run_batch_command(command),
        Command::Cache { command } => run_cache_command(command),
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
        Command::Templates { command } => run_template_command(command),
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

fn run_fasta_command(command: FastaCommand) -> Result<(), CliError> {
    match command {
        FastaCommand::Validate { kind, path } => run_sequence_validation(path, kind),
    }
}

fn run_formats_command(command: FormatsCommand) -> Result<(), CliError> {
    match command {
        FormatsCommand::List => print_success(None, format_capabilities()),
        FormatsCommand::Validate { format, path } => run_format_validation(format, path),
    }
}

fn run_format_validation(format: FormatArg, path: PathBuf) -> Result<(), CliError> {
    match format {
        FormatArg::Fastq => {
            let reader = open_buffered_input(&path)?;
            let output = validate_fastq_reader_with_hash(reader)
                .map_err(|error| CliError::from_format_read(path, error))?;
            print_success(Some(output.input_hash), output.report)
        }
    }
}

fn run_seq_command(command: SeqCommand) -> Result<(), CliError> {
    match command {
        SeqCommand::Validate { kind, path } => run_sequence_validation(path, kind),
    }
}

fn run_service_command(command: ServiceCommand) -> Result<(), CliError> {
    match command {
        ServiceCommand::Contract => print_success(
            None,
            biors_core::service::current_service_interface_document(),
        ),
    }
}

fn run_structure_command(command: StructureCommand) -> Result<(), CliError> {
    match command {
        StructureCommand::Validate { format, path } => run_structure_validation(format, path),
        StructureCommand::Sequence { format, path } => run_structure_sequence(format, path),
    }
}

fn run_structure_validation(format: StructureFormatArg, path: PathBuf) -> Result<(), CliError> {
    match format {
        StructureFormatArg::Pdb => {
            let reader = open_buffered_input(&path)?;
            let output = validate_pdb_reader_with_hash(reader)
                .map_err(|error| CliError::from_structure_read(path, error))?;
            print_success(Some(output.input_hash), output.report)
        }
    }
}

fn run_structure_sequence(format: StructureFormatArg, path: PathBuf) -> Result<(), CliError> {
    match format {
        StructureFormatArg::Pdb => {
            let reader = open_buffered_input(&path)?;
            let output = parse_pdb_record_reader(reader)
                .map_err(|error| CliError::from_structure_read(path, error))?;
            let sequences = extract_structure_sequences(&output.record);
            print_success(Some(output.input_hash), sequences)
        }
    }
}

fn run_molecule_command(command: MoleculeCommand) -> Result<(), CliError> {
    match command {
        MoleculeCommand::Validate { format, path } => run_molecule_validation(format, path),
        MoleculeCommand::Inspect { format, path } => run_molecule_inspect(format, path),
    }
}

fn run_molecule_validation(format: MoleculeFormatArg, path: PathBuf) -> Result<(), CliError> {
    match format {
        MoleculeFormatArg::Smiles => {
            let reader = open_buffered_input(&path)?;
            let output = validate_smiles_reader_with_hash(reader)
                .map_err(|error| CliError::from_molecule_read(path, error))?;
            print_success(Some(output.input_hash), output.report)
        }
        MoleculeFormatArg::Sdf => {
            let reader = open_buffered_input(&path)?;
            let output = parse_sdf_records_reader(reader)
                .map_err(|error| CliError::from_molecule_read(path, error))?;
            print_success(
                Some(output.input_hash),
                validate_molecule_records(&output.records),
            )
        }
        MoleculeFormatArg::Mol2 => {
            let reader = open_buffered_input(&path)?;
            let output = parse_mol2_records_reader(reader)
                .map_err(|error| CliError::from_molecule_read(path, error))?;
            print_success(
                Some(output.input_hash),
                validate_molecule_records(&output.records),
            )
        }
    }
}

fn run_molecule_inspect(format: MoleculeFormatArg, path: PathBuf) -> Result<(), CliError> {
    match format {
        MoleculeFormatArg::Smiles => {
            let reader = open_buffered_input(&path)?;
            let output = parse_smiles_records_reader(reader)
                .map_err(|error| CliError::from_molecule_read(path, error))?;
            print_success(Some(output.input_hash), output.records)
        }
        MoleculeFormatArg::Sdf => {
            let reader = open_buffered_input(&path)?;
            let output = parse_sdf_records_reader(reader)
                .map_err(|error| CliError::from_molecule_read(path, error))?;
            print_success(Some(output.input_hash), output.records)
        }
        MoleculeFormatArg::Mol2 => {
            let reader = open_buffered_input(&path)?;
            let output = parse_mol2_records_reader(reader)
                .map_err(|error| CliError::from_molecule_read(path, error))?;
            print_success(Some(output.input_hash), output.records)
        }
    }
}

fn run_inspect(path: PathBuf) -> Result<(), CliError> {
    let reader = open_fasta_input(&path)?;
    let output = summarize_fasta_records_reader(reader)
        .map_err(|error| CliError::from_fasta_read(path, error))?;
    print_success(Some(output.input_hash), output.summary)
}

fn run_model_input(
    profile: TokenizerProfileArg,
    max_length: usize,
    pad_token_id: u8,
    padding: PaddingArg,
    path: PathBuf,
) -> Result<(), CliError> {
    let config = protein_tokenizer_config_for_profile(profile.into());
    let reader = open_fasta_input(&path)?;
    let output = tokenize_fasta_records_reader_with_config(reader, &config)
        .map_err(|error| CliError::from_fasta_read(path, error))?;
    let model_input = build_model_inputs_checked(
        &output.records,
        ModelInputPolicy {
            max_length,
            pad_token_id,
            padding: padding.into(),
        },
    )?;
    print_success(Some(output.input_hash), model_input)
}

fn run_tokenize(
    profile: TokenizerProfileArg,
    config: Option<PathBuf>,
    path: PathBuf,
) -> Result<(), CliError> {
    let config = resolve_tokenizer_config(profile, config)?;
    let reader = open_fasta_input(&path)?;
    let output = tokenize_fasta_records_reader_with_config(reader, &config)
        .map_err(|error| CliError::from_fasta_read(path, error))?;
    print_success(Some(output.input_hash), output.records)
}

fn run_tokenizer_command(command: TokenizerCommand) -> Result<(), CliError> {
    match command {
        TokenizerCommand::ConvertHf { path, output } => run_tokenizer_convert_hf(path, output),
        TokenizerCommand::Inspect { profile, config } => {
            let config = resolve_tokenizer_config(profile, config)?;
            print_success(None, inspect_protein_tokenizer_config(&config))
        }
    }
}

fn run_tokenizer_convert_hf(path: PathBuf, output: Option<PathBuf>) -> Result<(), CliError> {
    crate::cli::tokenizer_convert::run_tokenizer_convert_hf(path, output)
}

fn resolve_tokenizer_config(
    profile: TokenizerProfileArg,
    config: Option<PathBuf>,
) -> Result<ProteinTokenizerConfig, CliError> {
    match config {
        Some(path) => read_tokenizer_config(path),
        None => Ok(protein_tokenizer_config_for_profile(profile.into())),
    }
}

fn run_sequence_validation(path: PathBuf, kind: KindArg) -> Result<(), CliError> {
    let reader = open_fasta_input(&path)?;
    let output = validate_fasta_reader_with_kind_and_hash(reader, kind.into())
        .map_err(|error| CliError::from_fasta_read(path, error))?;
    print_success(Some(output.input_hash), output.report)
}

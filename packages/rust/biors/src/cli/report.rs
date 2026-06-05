use super::ReportCommand;
use crate::errors::CliError;
use crate::output::print_success;
use biors_core::reports::build_shareable_report_from_json_bytes;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};

pub(crate) fn run_report_command(command: ReportCommand) -> Result<(), CliError> {
    match command {
        ReportCommand::Generate {
            input,
            output,
            shareable_json,
        } => run_generate(input, output, shareable_json),
    }
}

fn run_generate(
    input: PathBuf,
    output: Option<PathBuf>,
    shareable_json: Option<PathBuf>,
) -> Result<(), CliError> {
    reject_stdout_output(output.as_deref(), "report.output_stdout_ambiguous")?;
    reject_stdout_output(
        shareable_json.as_deref(),
        "report.shareable_json_stdout_ambiguous",
    )?;

    let bytes = read_input(&input)?;
    let report = build_shareable_report_from_json_bytes(&bytes)?;
    if let Some(path) = output {
        fs::write(&path, &report.human_report).map_err(CliError::Write)?;
    }
    if let Some(path) = shareable_json {
        let json = serde_json::to_vec_pretty(&report).map_err(CliError::Serialization)?;
        fs::write(&path, json).map_err(CliError::Write)?;
    }
    print_success(None, report)
}

fn read_input(path: &PathBuf) -> Result<Vec<u8>, CliError> {
    if path.as_os_str() == "-" {
        let mut input = Vec::new();
        std::io::stdin()
            .read_to_end(&mut input)
            .map_err(|source| CliError::Read {
                path: path.clone(),
                source,
            })?;
        return Ok(input);
    }

    fs::read(path).map_err(|source| CliError::Read {
        path: path.clone(),
        source,
    })
}

fn reject_stdout_output(path: Option<&Path>, code: &'static str) -> Result<(), CliError> {
    if matches!(path, Some(path) if path.as_os_str() == "-") {
        return Err(CliError::Validation {
            code,
            message: "report file outputs cannot use '-' because stdout is reserved for the JSON envelope"
                .to_string(),
            location: Some("-".to_string()),
        });
    }
    Ok(())
}

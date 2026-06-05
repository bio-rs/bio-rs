mod cli;
mod errors;
mod exit_code;
mod input;
mod output;

use clap::{error::ErrorKind, Parser};
use std::ffi::OsStr;

fn main() {
    let cli = match cli::Cli::try_parse() {
        Ok(cli) => cli,
        Err(error) => {
            if wants_json_errors()
                && !matches!(
                    error.kind(),
                    ErrorKind::DisplayHelp | ErrorKind::DisplayVersion
                )
            {
                output::print_json_parse_error(&error);
                std::process::exit(exit_code::USER_INPUT_FAILURE);
            }
            error.exit();
        }
    };
    if let Err(error) = cli::run(cli.command) {
        let exit_code = error.exit_code();
        if cli.json {
            output::print_json_error(error);
        } else {
            eprintln!("error[{}]: {error}", error.code());
        }
        std::process::exit(exit_code);
    }
}

fn wants_json_errors() -> bool {
    std::env::args_os().any(|arg| arg == OsStr::new("--json"))
}

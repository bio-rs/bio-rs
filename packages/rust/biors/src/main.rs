mod commands;
mod errors;
mod exit_code;
mod input;
mod output;

use clap::Parser;

fn main() {
    let cli = commands::Cli::parse();
    if let Err(error) = commands::run(cli.command) {
        let exit_code = error.exit_code();
        if cli.json {
            output::print_json_error(error);
        } else {
            eprintln!("error: {error}");
        }
        std::process::exit(exit_code);
    }
}

mod cli;
mod errors;
mod exit_code;
mod input;
mod output;

use clap::Parser;

fn main() {
    let cli = cli::Cli::parse();
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

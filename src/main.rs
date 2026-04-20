mod app;
mod cli;
mod commands;
mod config;
mod core;
mod model;
mod output;
mod roadmap;
#[cfg(test)]
mod test_support;
mod version;

use clap::Parser;
use std::process::ExitCode;

use crate::app::run;
use crate::cli::Cli;
use crate::output::{danger, output_error};

fn main() -> ExitCode {
    let cli = Cli::parse();
    let json = cli.json;

    match run(cli) {
        Ok(outcome) => {
            if let Some(output) = outcome.output {
                println!("{output}");
            }
            outcome.exit_code
        }
        Err(error) => {
            if json {
                println!("{}", output_error(error.code, error.message));
            } else {
                eprintln!("{}", danger(&format!("ish: {}", error.message)));
            }
            ExitCode::FAILURE
        }
    }
}

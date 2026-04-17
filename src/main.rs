mod cli;
mod config;
mod core;
mod model;
mod output;

use clap::Parser;

/// A terminal-based issue tracker.
#[derive(Parser)]
#[command(name = "ish", version, about)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(clap::Subcommand)]
enum Commands {}

fn main() {
    let _cli = Cli::parse();
    println!("ish: no command specified. Run `ish --help` for usage.");
}

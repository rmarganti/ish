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
enum Commands {
    /// Print the current ish version.
    Version,
}

fn version_output() -> String {
    format!("ish {}", env!("CARGO_PKG_VERSION"))
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Version) => println!("{}", version_output()),
        None => println!("ish: no command specified. Run `ish --help` for usage."),
    }
}

#[cfg(test)]
mod tests {
    use super::version_output;

    #[test]
    fn version_output_uses_package_version() {
        assert_eq!(
            version_output(),
            format!("ish {}", env!("CARGO_PKG_VERSION"))
        );
    }
}

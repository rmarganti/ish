pub mod context;
pub mod error;

pub use context::{AppContext, current_dir, load_config_from_current_dir};
pub use error::{
    AppError, RunOutcome, classify_app_error, json_output_error, store_app_error, success_outcome,
};

use crate::cli::{Cli, Commands};
use crate::output::output_message;

pub fn run(cli: Cli) -> Result<RunOutcome, AppError> {
    match cli.command {
        Commands::Init => crate::commands::init_command(cli.json).map(success_outcome),
        Commands::Create(args) => {
            crate::commands::create_command(args, cli.json).map(success_outcome)
        }
        Commands::List(args) => crate::commands::list_command(args, cli.json).map(success_outcome),
        Commands::Update(args) => {
            crate::commands::update_command(args, cli.json).map(success_outcome)
        }
        Commands::Show(args) => crate::commands::show_command(args, cli.json).map(success_outcome),
        Commands::Delete(args) => {
            crate::commands::delete_command(args, cli.json).map(success_outcome)
        }
        Commands::Archive => crate::commands::archive_command(cli.json).map(success_outcome),
        Commands::Check(args) => crate::commands::check_command(args, cli.json),
        Commands::Prime => crate::commands::prime_command(cli.json).map(success_outcome),
        Commands::Roadmap(args) => {
            crate::commands::roadmap_command(args, cli.json).map(success_outcome)
        }
        Commands::Version => {
            if cli.json {
                Ok(success_outcome(Some(
                    output_message(crate::commands::version_output()).map_err(json_output_error)?,
                )))
            } else {
                Ok(success_outcome(Some(crate::commands::version_output())))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::run;
    use crate::cli::{Cli, Commands};
    use crate::commands::version_output;
    use serde_json::Value;

    #[test]
    fn run_version_wraps_output_in_json_mode() {
        let output = run(Cli {
            json: true,
            command: Commands::Version,
        })
        .expect("version command should succeed")
        .output
        .expect("version command should print output");

        let parsed: Value = serde_json::from_str(&output).expect("json should parse");
        assert_eq!(parsed, Value::String(version_output()));
    }
}

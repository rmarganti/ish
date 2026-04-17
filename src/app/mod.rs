pub mod context;
pub mod error;

pub use context::{AppContext, current_dir, load_config_from_current_dir};
pub use error::{
    AppError, RunOutcome, classify_app_error, json_output_error, store_app_error, success_outcome,
};

use crate::cli::{Cli, Commands};
use crate::output::{ErrorCode, output_error, output_message, warning};

pub fn run(cli: Cli) -> Result<RunOutcome, AppError> {
    match cli.command {
        Some(Commands::Init) => crate::commands::init_command(cli.json).map(success_outcome),
        Some(Commands::Create(args)) => {
            crate::commands::create_command(args, cli.json).map(success_outcome)
        }
        Some(Commands::List(args)) => {
            crate::commands::list_command(args, cli.json).map(success_outcome)
        }
        Some(Commands::Update(args)) => {
            crate::commands::update_command(args, cli.json).map(success_outcome)
        }
        Some(Commands::Show(args)) => {
            crate::commands::show_command(args, cli.json).map(success_outcome)
        }
        Some(Commands::Delete(args)) => {
            crate::commands::delete_command(args, cli.json).map(success_outcome)
        }
        Some(Commands::Archive) => crate::commands::archive_command(cli.json).map(success_outcome),
        Some(Commands::Check(args)) => crate::commands::check_command(args, cli.json),
        Some(Commands::Prime) => crate::commands::prime_command(cli.json).map(success_outcome),
        Some(Commands::Roadmap(args)) => {
            crate::commands::roadmap_command(args, cli.json).map(success_outcome)
        }
        Some(Commands::Version) => {
            if cli.json {
                Ok(success_outcome(Some(
                    output_message(crate::commands::version_output()).map_err(json_output_error)?,
                )))
            } else {
                Ok(success_outcome(Some(crate::commands::version_output())))
            }
        }
        None => {
            let message = "ish: no command specified. Run `ish --help` for usage.";
            if cli.json {
                Ok(success_outcome(Some(output_error(
                    ErrorCode::Validation,
                    message,
                ))))
            } else {
                Ok(success_outcome(Some(warning(message))))
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
            command: Some(Commands::Version),
        })
        .expect("version command should succeed")
        .output
        .expect("version command should print output");

        let parsed: Value = serde_json::from_str(&output).expect("json should parse");
        assert_eq!(parsed["success"], Value::Bool(true));
        assert_eq!(parsed["message"], Value::String(version_output()));
    }

    #[test]
    fn run_without_command_returns_validation_error_in_json_mode() {
        let output = run(Cli {
            json: true,
            command: None,
        })
        .expect("run should succeed")
        .output
        .expect("run should print output");

        let parsed: Value = serde_json::from_str(&output).expect("json should parse");
        assert_eq!(parsed["success"], Value::Bool(false));
        assert_eq!(parsed["code"], Value::String("validation".to_string()));
    }
}

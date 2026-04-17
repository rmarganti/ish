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
        Some(Commands::Init) => crate::init_command(cli.json).map(success_outcome),
        Some(Commands::Create(args)) => crate::create_command(args, cli.json).map(success_outcome),
        Some(Commands::List(args)) => crate::list_command(args, cli.json).map(success_outcome),
        Some(Commands::Update(args)) => crate::update_command(args, cli.json).map(success_outcome),
        Some(Commands::Show(args)) => crate::show_command(args, cli.json).map(success_outcome),
        Some(Commands::Delete(args)) => crate::delete_command(args, cli.json).map(success_outcome),
        Some(Commands::Archive) => crate::archive_command(cli.json).map(success_outcome),
        Some(Commands::Check(args)) => crate::check_command(args, cli.json),
        Some(Commands::Prime) => crate::prime_command(cli.json).map(success_outcome),
        Some(Commands::Roadmap(args)) => {
            crate::roadmap_command(args, cli.json).map(success_outcome)
        }
        Some(Commands::Version) => {
            if cli.json {
                Ok(success_outcome(Some(
                    output_message(crate::version_output()).map_err(json_output_error)?,
                )))
            } else {
                Ok(success_outcome(Some(crate::version_output())))
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

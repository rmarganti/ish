use crate::app::{AppContext, AppError, RunOutcome, json_output_error};
use crate::cli::CheckArgs;
use std::process::ExitCode;

mod config;
mod render;

use config::{link_issue_count, validate_config};
use render::{render_check_human, render_check_json};

pub(crate) fn check_command(args: CheckArgs, json: bool) -> Result<RunOutcome, AppError> {
    let context = AppContext::load()?;
    let config = context.config;
    let mut store = context.store;
    let config_checks = validate_config(&config);
    let initial_links = store.check_all_links();
    let issues_found = config_checks.issue_count() + link_issue_count(&initial_links);

    let fixed_links = if args.fix {
        Some(store.fix_broken_links().map_err(|error| {
            AppError::new(
                crate::output::ErrorCode::FileError,
                format!("failed to fix broken links: {error}"),
            )
        })?)
    } else {
        None
    };
    let final_links = if args.fix {
        store.check_all_links()
    } else {
        initial_links.clone()
    };

    let output = if json {
        render_check_json(&config_checks, &initial_links, &final_links, fixed_links)
            .map_err(json_output_error)?
    } else {
        render_check_human(&config_checks, &initial_links, &final_links, fixed_links)
    };

    Ok(RunOutcome {
        output: Some(output),
        exit_code: if issues_found == 0 {
            ExitCode::SUCCESS
        } else {
            ExitCode::FAILURE
        },
    })
}

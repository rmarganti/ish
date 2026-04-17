use crate::app::{AppContext, AppError, classify_app_error, json_output_error};
use crate::cli::RoadmapArgs;
use crate::output::{ErrorCode, output_success};
use crate::roadmap::{RoadmapOptions, roadmap_output};
use serde_json::Value;

pub(crate) fn roadmap_command(args: RoadmapArgs, json: bool) -> Result<Option<String>, AppError> {
    let current_dir = AppContext::load()?.current_dir;

    let output = roadmap_output(
        &current_dir,
        &RoadmapOptions {
            include_done: args.include_done,
            status: args.status,
            no_status: args.no_status,
            no_links: args.no_links,
            link_prefix: args.link_prefix,
            json,
        },
    )
    .map_err(classify_app_error)?;

    if json {
        let Some(output) = output else {
            return Ok(None);
        };
        let data: Value = serde_json::from_str(&output).map_err(|error| {
            AppError::new(
                ErrorCode::FileError,
                format!("failed to parse command JSON output: {error}"),
            )
        })?;
        Ok(Some(output_success(data).map_err(json_output_error)?))
    } else {
        Ok(output)
    }
}

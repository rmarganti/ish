use crate::app::{AppContext, AppError, json_output_error};
use crate::output::{ErrorCode, output_success, success, warning};
use serde_json::json;

pub(crate) fn archive_command(json: bool) -> Result<Option<String>, AppError> {
    let mut store = AppContext::load()?.store;
    let archived = store.archive_all_completed().map_err(|error| {
        AppError::new(
            ErrorCode::FileError,
            format!("failed to archive completed ishoos: {error}"),
        )
    })?;

    if json {
        return Ok(Some(
            output_success(json!({ "archived": archived })).map_err(json_output_error)?,
        ));
    }

    let message = if archived == 0 {
        "no completed or scrapped ishoos to archive".to_string()
    } else if archived == 1 {
        "archived 1 ishoo".to_string()
    } else {
        format!("archived {archived} ishoos")
    };

    if archived == 0 {
        Ok(Some(warning(&message)))
    } else {
        Ok(Some(success(&message)))
    }
}

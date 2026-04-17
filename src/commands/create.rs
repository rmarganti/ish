use crate::app::{AppContext, AppError, json_output_error, store_app_error};
use crate::cli::CreateArgs;
use crate::core::store::CreateIshoo;
use crate::output::{ErrorCode, muted, output_success, render_id, success};
use std::fs;
use std::io::{self, Read};

pub(crate) fn create_command(args: CreateArgs, json: bool) -> Result<Option<String>, AppError> {
    let context = AppContext::load()?;
    let mut store = context.store;

    let ishoo = store
        .create(CreateIshoo {
            title: args.title.unwrap_or_else(|| "Untitled".to_string()),
            status: args.status,
            ishoo_type: args.ishoo_type,
            priority: args.priority,
            body: resolve_create_body(args.body, args.body_file)?,
            tags: args.tags,
            parent: args.parent,
            blocking: args.blocking,
            blocked_by: args.blocked_by,
            id_prefix: args.prefix,
        })
        .map_err(store_app_error)?;

    if json {
        return Ok(Some(
            output_success(ishoo.to_json(&ishoo.etag())).map_err(json_output_error)?,
        ));
    }

    Ok(Some(success(&format!(
        "Created {} {}",
        render_id(&ishoo.id),
        muted(&ishoo.path)
    ))))
}

fn resolve_create_body(
    body: Option<String>,
    body_file: Option<String>,
) -> Result<String, AppError> {
    match (body, body_file) {
        (Some(body), None) if body == "-" => read_from_stdin("body"),
        (Some(body), None) => Ok(body),
        (None, Some(path)) => fs::read_to_string(&path).map_err(|error| {
            AppError::new(
                ErrorCode::FileError,
                format!("failed to read body file `{path}`: {error}"),
            )
        }),
        (None, None) => Ok(String::new()),
        (Some(_), Some(_)) => Err(AppError::new(
            ErrorCode::Validation,
            "`--body` and `--body-file` cannot be used together",
        )),
    }
}

fn read_from_stdin(label: &str) -> Result<String, AppError> {
    let mut stdin = String::new();
    io::stdin().read_to_string(&mut stdin).map_err(|error| {
        AppError::new(
            ErrorCode::FileError,
            format!("failed to read {label} from stdin: {error}"),
        )
    })?;
    Ok(stdin)
}

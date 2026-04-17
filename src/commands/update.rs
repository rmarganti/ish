use crate::app::{AppContext, AppError, json_output_error, store_app_error};
use crate::cli::UpdateArgs;
use crate::core::store::UpdateIshoo;
use crate::output::{ErrorCode, muted, output_success, render_id, success};
use std::fs;
use std::io::{self, Read};

pub(crate) fn update_command(args: UpdateArgs, json: bool) -> Result<Option<String>, AppError> {
    let changes = resolve_update_changes(args)?;
    let mut store = AppContext::load()?.store;
    let should_unarchive = store.is_archived(&changes.0).map_err(store_app_error)?;

    if should_unarchive {
        store
            .load_and_unarchive(&changes.0)
            .map_err(store_app_error)?;
    }

    let updated = store
        .update(&changes.0, changes.1)
        .map_err(store_app_error)?;

    if json {
        return Ok(Some(
            output_success(updated.to_json(&updated.etag())).map_err(json_output_error)?,
        ));
    }

    Ok(Some(success(&format!(
        "Updated {} {}",
        render_id(&updated.id),
        muted(&updated.path)
    ))))
}

pub(crate) fn resolve_update_changes(args: UpdateArgs) -> Result<(String, UpdateIshoo), AppError> {
    let body = resolve_optional_body(args.body, args.body_file)?;
    let body_append = resolve_optional_stdin_text(args.body_append, "body append")?;
    let body_replace = args.body_replace_old.zip(args.body_replace_new);
    let parent = if args.remove_parent {
        Some(None)
    } else {
        args.parent.map(Some)
    };
    let priority = args.priority.map(|priority| {
        if priority.eq_ignore_ascii_case("none") {
            None
        } else {
            Some(priority)
        }
    });

    let has_changes = args.status.is_some()
        || args.ishoo_type.is_some()
        || priority.is_some()
        || args.title.is_some()
        || body.is_some()
        || body_replace.is_some()
        || body_append.is_some()
        || !args.tags.is_empty()
        || !args.remove_tags.is_empty()
        || parent.is_some()
        || !args.blocking.is_empty()
        || !args.remove_blocking.is_empty()
        || !args.blocked_by.is_empty()
        || !args.remove_blocked_by.is_empty();

    if !has_changes {
        return Err(AppError::new(ErrorCode::Validation, "no changes specified"));
    }

    Ok((
        args.id,
        UpdateIshoo {
            status: args.status,
            ishoo_type: args.ishoo_type,
            priority,
            title: args.title,
            body,
            body_replace,
            body_append,
            add_tags: args.tags,
            remove_tags: args.remove_tags,
            parent,
            add_blocking: args.blocking,
            remove_blocking: args.remove_blocking,
            add_blocked_by: args.blocked_by,
            remove_blocked_by: args.remove_blocked_by,
            if_match: args.if_match,
        },
    ))
}

fn resolve_optional_body(
    body: Option<String>,
    body_file: Option<String>,
) -> Result<Option<String>, AppError> {
    match (body, body_file) {
        (Some(body), None) => Ok(Some(read_stdin_or_literal(body, "body")?)),
        (None, Some(path)) => Ok(Some(fs::read_to_string(&path).map_err(|error| {
            AppError::new(
                ErrorCode::FileError,
                format!("failed to read body file `{path}`: {error}"),
            )
        })?)),
        (None, None) => Ok(None),
        (Some(_), Some(_)) => Err(AppError::new(
            ErrorCode::Validation,
            "`--body` and `--body-file` cannot be used together",
        )),
    }
}

fn resolve_optional_stdin_text(
    value: Option<String>,
    label: &str,
) -> Result<Option<String>, AppError> {
    value
        .map(|text| read_stdin_or_literal(text, label))
        .transpose()
}

fn read_stdin_or_literal(value: String, label: &str) -> Result<String, AppError> {
    if value != "-" {
        return Ok(value);
    }

    read_text_input(io::stdin(), label)
}

pub(crate) fn read_text_input<R: Read>(mut reader: R, label: &str) -> Result<String, AppError> {
    let mut stdin = String::new();
    reader.read_to_string(&mut stdin).map_err(|error| {
        AppError::new(
            ErrorCode::FileError,
            format!("failed to read {label} from stdin: {error}"),
        )
    })?;
    Ok(stdin)
}

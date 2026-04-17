use crate::app::{AppContext, AppError, json_output_error, store_app_error};
use crate::cli::DeleteArgs;
use crate::core::store::{LinkRef, Store, StoreError};
use crate::model::ishoo::Ishoo;
use crate::output::{ErrorCode, muted, output_success, render_id, success, warning};
use serde::Serialize;
use std::collections::HashSet;
use std::io::{self, BufRead, Write};

#[derive(Debug, Clone)]
pub(crate) struct DeleteTarget {
    pub(crate) ishoo: Ishoo,
    pub(crate) incoming_links: Vec<LinkRef>,
}

#[derive(Debug, Serialize)]
struct DeleteJson {
    deleted: Vec<crate::model::ishoo::IshooJson>,
    count: usize,
    cleaned_links: usize,
}

pub(crate) fn delete_command(args: DeleteArgs, json: bool) -> Result<Option<String>, AppError> {
    let mut stdin = io::stdin().lock();
    let mut stdout = io::stdout().lock();
    delete_command_with_io(args, json, &mut stdin, &mut stdout)
}

pub(crate) fn delete_command_with_io<R: BufRead, W: Write>(
    args: DeleteArgs,
    json: bool,
    input: &mut R,
    output: &mut W,
) -> Result<Option<String>, AppError> {
    let mut store = AppContext::load()?.store;
    let targets = resolve_delete_targets(&store, &args.ids)?;
    let cleaned_links = targets
        .iter()
        .map(|target| target.incoming_links.len())
        .sum::<usize>();

    if !args.force && !json && !confirm_delete(&targets, input, output)? {
        return Ok(Some(warning("Delete cancelled")));
    }

    let mut deleted = Vec::with_capacity(targets.len());
    for target in &targets {
        deleted.push(store.delete(&target.ishoo.id).map_err(store_app_error)?);
    }

    if json {
        return Ok(Some(
            output_success(DeleteJson {
                deleted: deleted
                    .iter()
                    .map(|ishoo| ishoo.to_json(&ishoo.etag()))
                    .collect(),
                count: deleted.len(),
                cleaned_links,
            })
            .map_err(json_output_error)?,
        ));
    }

    Ok(Some(render_delete_success(&deleted, cleaned_links)))
}

fn resolve_delete_targets(store: &Store, ids: &[String]) -> Result<Vec<DeleteTarget>, AppError> {
    let mut ordered_ids = Vec::new();
    let mut seen = HashSet::new();

    for id in ids {
        let normalized = store.normalize_id(id);
        if seen.insert(normalized.clone()) {
            ordered_ids.push(normalized);
        }
    }

    let target_ids = ordered_ids.iter().cloned().collect::<HashSet<_>>();
    ordered_ids
        .into_iter()
        .map(|id| {
            let ishoo = store
                .get(&id)
                .cloned()
                .ok_or_else(|| store_app_error(StoreError::NotFound(id.clone())))?;
            let incoming_links = store
                .find_incoming_links(&id)
                .into_iter()
                .filter(|link| !target_ids.contains(&link.source_id))
                .collect();

            Ok(DeleteTarget {
                ishoo,
                incoming_links,
            })
        })
        .collect()
}

pub(crate) fn confirm_delete<R: BufRead, W: Write>(
    targets: &[DeleteTarget],
    input: &mut R,
    output: &mut W,
) -> Result<bool, AppError> {
    let total_incoming = targets
        .iter()
        .map(|target| target.incoming_links.len())
        .sum::<usize>();
    let issue_label = if targets.len() == 1 {
        "ishoo"
    } else {
        "ishoos"
    };

    writeln!(output, "Delete {} {issue_label}?", targets.len()).map_err(prompt_io_error)?;
    for target in targets {
        writeln!(
            output,
            "- {} | title: {} | path: {} | incoming links: {}",
            target.ishoo.id,
            target.ishoo.title,
            target.ishoo.path,
            target.incoming_links.len()
        )
        .map_err(prompt_io_error)?;
    }

    if total_incoming > 0 {
        writeln!(
            output,
            "Warning: deleting these ishoos will remove {total_incoming} incoming link(s) from remaining ishoos."
        )
        .map_err(prompt_io_error)?;
    }

    write!(output, "Proceed? [y/N]: ").map_err(prompt_io_error)?;
    output.flush().map_err(prompt_io_error)?;

    let mut response = String::new();
    input.read_line(&mut response).map_err(prompt_io_error)?;
    let response = response.trim().to_ascii_lowercase();

    Ok(matches!(response.as_str(), "y" | "yes"))
}

fn render_delete_success(deleted: &[Ishoo], cleaned_links: usize) -> String {
    if deleted.len() == 1 {
        let deleted = &deleted[0];
        let suffix = if cleaned_links == 0 {
            String::new()
        } else {
            format!(" and cleaned {cleaned_links} incoming link(s)")
        };
        return success(&format!(
            "Deleted {} {}{suffix}",
            render_id(&deleted.id),
            muted(&deleted.path)
        ));
    }

    let suffix = if cleaned_links == 0 {
        String::new()
    } else {
        format!(" and cleaned {cleaned_links} incoming link(s)")
    };
    success(&format!("Deleted {} ishoos{suffix}", deleted.len()))
}

fn prompt_io_error(error: io::Error) -> AppError {
    AppError::new(
        ErrorCode::FileError,
        format!("failed to read delete confirmation: {error}"),
    )
}

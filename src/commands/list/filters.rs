use crate::app::AppError;
use crate::cli::ListArgs;
use crate::config::Config;
use crate::core::store::Store;
use crate::core::{SortMode, sort_ishes};
use crate::model::ish::Ish;
use crate::output::ErrorCode;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ArchiveVisibility {
    ActiveOnly,
    ArchivedOnly,
    All,
}

pub(super) fn archive_visibility(args: &ListArgs) -> ArchiveVisibility {
    if args.all {
        ArchiveVisibility::All
    } else if args.archived {
        ArchiveVisibility::ArchivedOnly
    } else {
        ArchiveVisibility::ActiveOnly
    }
}

pub(super) fn validate_list_args(args: &ListArgs, config: &Config) -> Result<(), AppError> {
    validate_named_filters(
        "status",
        args.status.iter().chain(args.no_status.iter()),
        |value| config.is_valid_status(value),
    )?;
    validate_named_filters(
        "type",
        args.ish_type.iter().chain(args.no_type.iter()),
        |value| config.is_valid_type(value),
    )?;
    validate_named_filters(
        "priority",
        args.priority.iter().chain(args.no_priority.iter()),
        |value| config.is_valid_priority(value),
    )?;
    Ok(())
}

fn validate_named_filters<'a, I, F>(label: &str, values: I, mut is_valid: F) -> Result<(), AppError>
where
    I: IntoIterator<Item = &'a String>,
    F: FnMut(&str) -> bool,
{
    for value in values {
        if !is_valid(value) {
            return Err(AppError::new(
                ErrorCode::Validation,
                format!("invalid {label}: {value}"),
            ));
        }
    }

    Ok(())
}

pub(super) fn filter_ishes<'a>(
    all_ishes: &'a [Ish],
    store: &Store,
    config: &Config,
    args: &ListArgs,
) -> Vec<&'a Ish> {
    let normalized_parent = args
        .parent
        .as_deref()
        .map(|parent| store.normalize_id(parent));
    let search = args.search.as_deref().map(str::to_ascii_lowercase);
    let archive_visibility = archive_visibility(args);

    all_ishes
        .iter()
        .filter(|ish| {
            match_filters(
                ish,
                store,
                config,
                args,
                archive_visibility,
                normalized_parent.as_deref(),
                search.as_deref(),
            )
        })
        .collect()
}

fn match_filters(
    ish: &Ish,
    store: &Store,
    config: &Config,
    args: &ListArgs,
    archive_visibility: ArchiveVisibility,
    normalized_parent: Option<&str>,
    search: Option<&str>,
) -> bool {
    let priority = ish.priority.as_deref().unwrap_or("normal");

    if archive_visibility == ArchiveVisibility::ActiveOnly && ish.is_archived() {
        return false;
    }
    if archive_visibility == ArchiveVisibility::ArchivedOnly && !ish.is_archived() {
        return false;
    }

    if !args.status.is_empty() && !args.status.iter().any(|status| status == &ish.status) {
        return false;
    }
    if args.no_status.iter().any(|status| status == &ish.status) {
        return false;
    }
    if !args.ish_type.is_empty()
        && !args
            .ish_type
            .iter()
            .any(|ish_type| ish_type == &ish.ish_type)
    {
        return false;
    }
    if args
        .no_type
        .iter()
        .any(|ish_type| ish_type == &ish.ish_type)
    {
        return false;
    }
    if !args.priority.is_empty() && !args.priority.iter().any(|candidate| candidate == priority) {
        return false;
    }
    if args
        .no_priority
        .iter()
        .any(|candidate| candidate == priority)
    {
        return false;
    }
    if !args.tag.is_empty() && !args.tag.iter().any(|tag| ish.has_tag(tag)) {
        return false;
    }
    if args.no_tag.iter().any(|tag| ish.has_tag(tag)) {
        return false;
    }
    if args.has_parent && ish.parent.is_none() {
        return false;
    }
    if args.no_parent && ish.parent.is_some() {
        return false;
    }
    if normalized_parent.is_some_and(|parent| ish.parent.as_deref() != Some(parent)) {
        return false;
    }
    if args.has_blocking && ish.blocking.is_empty() {
        return false;
    }
    if args.no_blocking && !ish.blocking.is_empty() {
        return false;
    }
    if args.is_blocked && !store.is_blocked(&ish.id) {
        return false;
    }
    if args.ready && !is_ready(ish, store, config) {
        return false;
    }
    if search.is_some_and(|query| !matches_search(ish, query)) {
        return false;
    }

    true
}

fn is_ready(ish: &Ish, store: &Store, config: &Config) -> bool {
    !ish.is_archived()
        && ish.status != "in-progress"
        && ish.status != "draft"
        && !config.is_archive_status(&ish.status)
        && !store.is_blocked(&ish.id)
        && store.implicit_status(&ish.id).is_none()
}

fn matches_search(ish: &Ish, query: &str) -> bool {
    ish.title.to_ascii_lowercase().contains(query)
        || ish.slug.to_ascii_lowercase().contains(query)
        || ish.body.to_ascii_lowercase().contains(query)
}

pub(super) fn sort_ish_refs<'a>(
    ishes: &[&'a Ish],
    sort_mode: SortMode,
    config: &Config,
) -> Vec<&'a Ish> {
    let owned = ishes.iter().map(|ish| (*ish).clone()).collect::<Vec<_>>();
    let sorted = sort_ishes(
        &owned,
        sort_mode,
        &config.status_names(),
        &config.priority_names(),
        &config.type_names(),
    );
    let mut by_id = ishes
        .iter()
        .map(|ish| (ish.id.as_str(), *ish))
        .collect::<HashMap<_, _>>();

    sorted
        .into_iter()
        .filter_map(|ish| by_id.remove(ish.id.as_str()))
        .collect()
}

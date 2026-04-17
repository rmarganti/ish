use crate::app::AppError;
use crate::cli::ListArgs;
use crate::config::Config;
use crate::core::store::Store;
use crate::core::{SortMode, sort_ishoos};
use crate::model::ishoo::Ishoo;
use crate::output::ErrorCode;
use std::collections::HashMap;

pub(super) fn validate_list_args(args: &ListArgs, config: &Config) -> Result<(), AppError> {
    validate_named_filters(
        "status",
        args.status.iter().chain(args.no_status.iter()),
        |value| config.is_valid_status(value),
    )?;
    validate_named_filters(
        "type",
        args.ishoo_type.iter().chain(args.no_type.iter()),
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

pub(super) fn filter_ishoos<'a>(
    all_ishoos: &'a [Ishoo],
    store: &Store,
    config: &Config,
    args: &ListArgs,
) -> Vec<&'a Ishoo> {
    let normalized_parent = args
        .parent
        .as_deref()
        .map(|parent| store.normalize_id(parent));
    let search = args.search.as_deref().map(str::to_ascii_lowercase);

    all_ishoos
        .iter()
        .filter(|ishoo| {
            match_filters(
                ishoo,
                store,
                config,
                args,
                normalized_parent.as_deref(),
                search.as_deref(),
            )
        })
        .collect()
}

fn match_filters(
    ishoo: &Ishoo,
    store: &Store,
    config: &Config,
    args: &ListArgs,
    normalized_parent: Option<&str>,
    search: Option<&str>,
) -> bool {
    let priority = ishoo.priority.as_deref().unwrap_or("normal");

    if !args.status.is_empty() && !args.status.iter().any(|status| status == &ishoo.status) {
        return false;
    }
    if args.no_status.iter().any(|status| status == &ishoo.status) {
        return false;
    }
    if !args.ishoo_type.is_empty()
        && !args
            .ishoo_type
            .iter()
            .any(|ishoo_type| ishoo_type == &ishoo.ishoo_type)
    {
        return false;
    }
    if args
        .no_type
        .iter()
        .any(|ishoo_type| ishoo_type == &ishoo.ishoo_type)
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
    if !args.tag.is_empty() && !args.tag.iter().any(|tag| ishoo.has_tag(tag)) {
        return false;
    }
    if args.no_tag.iter().any(|tag| ishoo.has_tag(tag)) {
        return false;
    }
    if args.has_parent && ishoo.parent.is_none() {
        return false;
    }
    if args.no_parent && ishoo.parent.is_some() {
        return false;
    }
    if normalized_parent.is_some_and(|parent| ishoo.parent.as_deref() != Some(parent)) {
        return false;
    }
    if args.has_blocking && ishoo.blocking.is_empty() {
        return false;
    }
    if args.no_blocking && !ishoo.blocking.is_empty() {
        return false;
    }
    if args.is_blocked && !store.is_blocked(&ishoo.id) {
        return false;
    }
    if args.ready && !is_ready(ishoo, store, config) {
        return false;
    }
    if search.is_some_and(|query| !matches_search(ishoo, query)) {
        return false;
    }

    true
}

fn is_ready(ishoo: &Ishoo, store: &Store, config: &Config) -> bool {
    ishoo.status != "in-progress"
        && ishoo.status != "draft"
        && !config.is_archive_status(&ishoo.status)
        && !store.is_blocked(&ishoo.id)
        && store.implicit_status(&ishoo.id).is_none()
}

fn matches_search(ishoo: &Ishoo, query: &str) -> bool {
    ishoo.title.to_ascii_lowercase().contains(query)
        || ishoo.slug.to_ascii_lowercase().contains(query)
        || ishoo.body.to_ascii_lowercase().contains(query)
}

pub(super) fn sort_ishoo_refs<'a>(
    ishoos: &[&'a Ishoo],
    sort_mode: SortMode,
    config: &Config,
) -> Vec<&'a Ishoo> {
    let owned = ishoos
        .iter()
        .map(|ishoo| (*ishoo).clone())
        .collect::<Vec<_>>();
    let sorted = sort_ishoos(
        &owned,
        sort_mode,
        &config.status_names(),
        &config.priority_names(),
        &config.type_names(),
    );
    let mut by_id = ishoos
        .iter()
        .map(|ishoo| (ishoo.id.as_str(), *ishoo))
        .collect::<HashMap<_, _>>();

    sorted
        .into_iter()
        .filter_map(|ishoo| by_id.remove(ishoo.id.as_str()))
        .collect()
}

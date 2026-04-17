mod cli;
mod config;
mod core;
mod model;
mod output;
mod roadmap;

use clap::Parser;
use serde::Serialize;
use serde_json::json;
use serde_json::{Value, to_value};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::io::{self, BufRead, Read, Write};
use std::path::Path;
use std::process::ExitCode;

use crate::cli::{
    CheckArgs, Cli, Commands, CreateArgs, DeleteArgs, ListArgs, ListSortArg, RoadmapArgs, ShowArgs,
    UpdateArgs, prime_output,
};
use crate::config::{CONFIG_FILE_NAME, Config, find_config};
use crate::core::store::{
    CreateIshoo, LinkCheckResult, LinkCycle, LinkRef, LinkType, Store, StoreError, UpdateIshoo,
};
use crate::core::{SortMode, sort_ishoos};
use crate::model::ishoo::Ishoo;
use crate::output::{
    ErrorCode, build_tree, danger, detect_terminal_width, heading, is_supported_color_name, muted,
    output_error, output_message, output_success, output_success_multiple, render_id,
    render_markdown_with_width, render_priority, render_status, render_tree, render_type, success,
    warning,
};
use crate::roadmap::{RoadmapOptions, roadmap_output};

const STORE_DIRECTORY: &str = ".ish";
const STORE_GITIGNORE_NAME: &str = ".gitignore";
const STORE_GITIGNORE_CONTENT: &str = ".conversations/\n";

#[derive(Debug, Clone)]
struct DeleteTarget {
    ishoo: Ishoo,
    incoming_links: Vec<LinkRef>,
}

#[derive(Debug, Serialize)]
struct DeleteJson {
    deleted: Vec<crate::model::ishoo::IshooJson>,
    count: usize,
    cleaned_links: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct AppError {
    code: ErrorCode,
    message: String,
}

struct RunOutcome {
    output: Option<String>,
    exit_code: ExitCode,
}

impl AppError {
    fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }
}

fn version_output() -> String {
    format!("ish {}", env!("CARGO_PKG_VERSION"))
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    let json = cli.json;

    match run(cli) {
        Ok(outcome) => {
            if let Some(output) = outcome.output {
                println!("{output}");
            }
            outcome.exit_code
        }
        Err(error) => {
            if json {
                println!("{}", output_error(error.code, error.message));
            } else {
                eprintln!("{}", danger(&format!("ish: {}", error.message)));
            }
            ExitCode::FAILURE
        }
    }
}

fn run(cli: Cli) -> Result<RunOutcome, AppError> {
    match cli.command {
        Some(Commands::Init) => init_command(cli.json).map(success_outcome),
        Some(Commands::Create(args)) => create_command(args, cli.json).map(success_outcome),
        Some(Commands::List(args)) => list_command(args, cli.json).map(success_outcome),
        Some(Commands::Update(args)) => update_command(args, cli.json).map(success_outcome),
        Some(Commands::Show(args)) => show_command(args, cli.json).map(success_outcome),
        Some(Commands::Delete(args)) => delete_command(args, cli.json).map(success_outcome),
        Some(Commands::Archive) => archive_command(cli.json).map(success_outcome),
        Some(Commands::Check(args)) => check_command(args, cli.json),
        Some(Commands::Prime) => prime_command(cli.json).map(success_outcome),
        Some(Commands::Roadmap(args)) => roadmap_command(args, cli.json).map(success_outcome),
        Some(Commands::Version) => {
            if cli.json {
                Ok(success_outcome(Some(
                    output_message(version_output()).map_err(json_output_error)?,
                )))
            } else {
                Ok(success_outcome(Some(version_output())))
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

fn success_outcome(output: Option<String>) -> RunOutcome {
    RunOutcome {
        output,
        exit_code: ExitCode::SUCCESS,
    }
}

fn create_command(args: CreateArgs, json: bool) -> Result<Option<String>, AppError> {
    let current_dir = std::env::current_dir().map_err(|error| {
        AppError::new(
            ErrorCode::FileError,
            format!("failed to determine current directory: {error}"),
        )
    })?;
    let Some(config_path) = find_config(&current_dir) else {
        return Err(AppError::new(
            ErrorCode::NotFound,
            "no `.ish.yml` found in the current directory or its parents",
        ));
    };

    let config = Config::load(&config_path).map_err(|error| {
        AppError::new(
            ErrorCode::FileError,
            format!("failed to load `{}`: {error}", config_path.display()),
        )
    })?;

    let store_root = config_path
        .parent()
        .ok_or_else(|| {
            AppError::new(
                ErrorCode::FileError,
                format!("invalid config path: {}", config_path.display()),
            )
        })?
        .join(&config.ish.path);
    let mut store = Store::new(&store_root, config).map_err(store_open_error(&store_root))?;
    store.load().map_err(store_open_error(&store_root))?;

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

fn list_command(args: ListArgs, json: bool) -> Result<Option<String>, AppError> {
    let (_, config, store) = load_store_from_current_dir()?;
    validate_list_args(&args, &config)?;

    let all_ishoos = store.all().into_iter().cloned().collect::<Vec<_>>();
    let filtered = filter_ishoos(&all_ishoos, &store, &config, &args);
    let sort_mode = args
        .sort
        .map(ListSortArg::into_sort_mode)
        .unwrap_or(SortMode::Default);
    let sorted = sort_ishoo_refs(&filtered, sort_mode, &config);

    if json {
        let ishoos = sorted
            .into_iter()
            .map(|ishoo| list_json_value(ishoo, args.full))
            .collect::<Result<Vec<_>, _>>()?;
        return Ok(Some(
            output_success_multiple(ishoos).map_err(json_output_error)?,
        ));
    }

    if args.quiet {
        let output = sorted
            .iter()
            .map(|ishoo| ishoo.id.as_str())
            .collect::<Vec<_>>()
            .join("\n");
        return Ok((!output.is_empty()).then_some(output));
    }

    if sorted.is_empty() {
        return Ok(Some(warning("No ishoos found")));
    }

    let all_refs = all_ishoos.iter().collect::<Vec<_>>();
    let implicit_statuses = sorted
        .iter()
        .filter_map(|ishoo| {
            store
                .implicit_status(&ishoo.id)
                .map(|(status, _)| (ishoo.id.clone(), status))
        })
        .collect::<HashMap<_, _>>();
    let tree = build_tree(
        &sorted,
        &all_refs,
        |items| sort_ishoo_refs(items, sort_mode, &config),
        &implicit_statuses,
    );

    Ok(Some(render_tree(
        &tree,
        &config,
        max_tree_id_width(&tree),
        tree_has_tags(&tree),
        detect_terminal_width(),
    )))
}

fn show_command(args: ShowArgs, json: bool) -> Result<Option<String>, AppError> {
    let (_, config, store) = load_store_from_current_dir()?;
    let ishoos = resolve_show_ishoos(&store, &args.ids)?;

    if json {
        let rendered = ishoos
            .iter()
            .map(|ishoo| serde_json::to_value(ishoo.to_json(&ishoo.etag())))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|error| {
                AppError::new(
                    ErrorCode::FileError,
                    format!("failed to serialize show output: {error}"),
                )
            })?;
        return Ok(Some(
            output_success_multiple(rendered).map_err(json_output_error)?,
        ));
    }

    if args.etag_only {
        return Ok(Some(
            ishoos
                .iter()
                .map(|ishoo| ishoo.etag())
                .collect::<Vec<_>>()
                .join("\n"),
        ));
    }

    if args.body_only {
        return Ok(Some(render_show_sections(
            &ishoos
                .iter()
                .map(|ishoo| ishoo.body.clone())
                .collect::<Vec<_>>(),
        )));
    }

    if args.raw {
        return Ok(Some(render_show_sections(
            &ishoos
                .iter()
                .map(|ishoo| ishoo.render())
                .collect::<Vec<_>>(),
        )));
    }

    Ok(Some(render_show_sections(
        &ishoos
            .iter()
            .map(|ishoo| render_show_human(ishoo, &store, &config))
            .collect::<Vec<_>>(),
    )))
}

fn resolve_show_ishoos(store: &Store, ids: &[String]) -> Result<Vec<Ishoo>, AppError> {
    let mut ordered = Vec::new();
    let mut seen = HashSet::new();

    for id in ids {
        let normalized = store.normalize_id(id);
        if seen.insert(normalized.clone()) {
            let ishoo = store
                .get(&normalized)
                .cloned()
                .ok_or_else(|| store_app_error(StoreError::NotFound(normalized.clone())))?;
            ordered.push(ishoo);
        }
    }

    Ok(ordered)
}

fn render_show_sections(sections: &[String]) -> String {
    sections.join("\n════════════════════════════════════════════════════════════════\n")
}

fn render_show_human(ishoo: &Ishoo, store: &Store, config: &Config) -> String {
    let priority = ishoo.priority.as_deref().unwrap_or("normal");
    let mut lines = vec![format!(
        "{} {} {} {} {}{}",
        render_id(&ishoo.id),
        render_status(config, &ishoo.status),
        render_type(config, &ishoo.ishoo_type),
        render_priority(config, priority),
        heading(&ishoo.title),
        if ishoo.tags.is_empty() {
            String::new()
        } else {
            format!(" {}", muted(&format!("#{}", ishoo.tags.join(" #"))))
        }
    )];

    lines.push(format!("Path: {}", ishoo.path));
    lines.push(format!(
        "Created: {} | Updated: {} | ETag: {}",
        ishoo.created_at.to_rfc3339(),
        ishoo.updated_at.to_rfc3339(),
        ishoo.etag()
    ));
    lines.push("─".repeat(64));

    let mut relationships = Vec::new();
    relationships.push(format_relationship(
        "Parent",
        ishoo.parent.as_deref().map(str::to_string),
    ));
    relationships.push(format_relationships("Blocking", &ishoo.blocking));
    relationships.push(format_relationships("Blocked by", &ishoo.blocked_by));
    relationships.push(format_relationships(
        "Incoming",
        &store
            .find_incoming_links(&ishoo.id)
            .into_iter()
            .map(|link| format!("{} ({})", link.source_id, link_type_label(link.link_type)))
            .collect::<Vec<_>>(),
    ));
    if let Some((status, source_id)) = store.implicit_status(&ishoo.id) {
        relationships.push(format!("Inherited status: {} from {}", status, source_id));
    }
    if store.is_blocked(&ishoo.id) {
        relationships.push(format_relationships(
            "Active blockers",
            &store.find_all_blockers(&ishoo.id),
        ));
    }
    lines.extend(relationships);
    lines.push("─".repeat(64));

    let width = detect_terminal_width().saturating_sub(2).max(20);
    let body = render_markdown_with_width(&ishoo.body, width);
    if body.is_empty() {
        lines.push(muted("(no body)"));
    } else {
        lines.push(body.trim_end().to_string());
    }

    lines.join("\n")
}

fn format_relationship(label: &str, value: Option<String>) -> String {
    match value {
        Some(value) => format!("{label}: {value}"),
        None => format!("{label}: none"),
    }
}

fn format_relationships(label: &str, values: &[String]) -> String {
    if values.is_empty() {
        format!("{label}: none")
    } else {
        format!("{label}: {}", values.join(", "))
    }
}

fn update_command(args: UpdateArgs, json: bool) -> Result<Option<String>, AppError> {
    let changes = resolve_update_changes(args)?;
    let (_, _, mut store) = load_store_from_current_dir()?;
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

fn validate_list_args(args: &ListArgs, config: &Config) -> Result<(), AppError> {
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

fn filter_ishoos<'a>(
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

fn sort_ishoo_refs<'a>(
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

fn list_json_value(ishoo: &Ishoo, full: bool) -> Result<Value, AppError> {
    let mut value = to_value(ishoo.to_json(&ishoo.etag())).map_err(|error| {
        AppError::new(
            ErrorCode::FileError,
            format!("failed to serialize list output: {error}"),
        )
    })?;

    if !full && let Some(object) = value.as_object_mut() {
        object.remove("body");
    }

    Ok(value)
}

fn tree_has_tags(tree: &[crate::output::TreeNode<'_>]) -> bool {
    tree.iter()
        .any(|node| !node.ishoo.tags.is_empty() || tree_has_tags(&node.children))
}

fn max_tree_id_width(tree: &[crate::output::TreeNode<'_>]) -> usize {
    tree.iter()
        .map(|node| node.ishoo.id.len().max(max_tree_id_width(&node.children)))
        .max()
        .unwrap_or(0)
}

fn delete_command(args: DeleteArgs, json: bool) -> Result<Option<String>, AppError> {
    let mut stdin = io::stdin().lock();
    let mut stdout = io::stdout().lock();
    delete_command_with_io(args, json, &mut stdin, &mut stdout)
}

fn delete_command_with_io<R: BufRead, W: Write>(
    args: DeleteArgs,
    json: bool,
    input: &mut R,
    output: &mut W,
) -> Result<Option<String>, AppError> {
    let (_, _, mut store) = load_store_from_current_dir()?;
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

fn confirm_delete<R: BufRead, W: Write>(
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

fn resolve_update_changes(args: UpdateArgs) -> Result<(String, UpdateIshoo), AppError> {
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

    read_from_stdin(label)
}

fn read_from_stdin(label: &str) -> Result<String, AppError> {
    read_text_input(io::stdin(), label)
}

fn read_text_input<R: Read>(mut reader: R, label: &str) -> Result<String, AppError> {
    let mut stdin = String::new();
    reader.read_to_string(&mut stdin).map_err(|error| {
        AppError::new(
            ErrorCode::FileError,
            format!("failed to read {label} from stdin: {error}"),
        )
    })?;
    Ok(stdin)
}

fn prompt_io_error(error: io::Error) -> AppError {
    AppError::new(
        ErrorCode::FileError,
        format!("failed to read delete confirmation: {error}"),
    )
}

fn init_command(json: bool) -> Result<Option<String>, AppError> {
    let current_dir = std::env::current_dir().map_err(|error| {
        AppError::new(
            ErrorCode::FileError,
            format!("failed to determine current directory: {error}"),
        )
    })?;
    let project_name = project_name(&current_dir)?;

    fs::create_dir_all(current_dir.join(STORE_DIRECTORY)).map_err(|error| {
        AppError::new(
            ErrorCode::FileError,
            format!("failed to create `{STORE_DIRECTORY}` directory: {error}"),
        )
    })?;

    let gitignore_path = current_dir.join(STORE_DIRECTORY).join(STORE_GITIGNORE_NAME);
    if !gitignore_path.exists() {
        fs::write(&gitignore_path, STORE_GITIGNORE_CONTENT).map_err(|error| {
            AppError::new(
                ErrorCode::FileError,
                format!("failed to write `{}`: {error}", gitignore_path.display()),
            )
        })?;
    }

    let config_path = current_dir.join(CONFIG_FILE_NAME);
    let message = if config_path.exists() {
        format!(
            "ish project already initialized in `{}`",
            current_dir.display()
        )
    } else {
        let mut config = Config::default_with_prefix(format!("{project_name}-"));
        config.project.name = project_name;
        config.save(&current_dir).map_err(|error| {
            AppError::new(
                ErrorCode::FileError,
                format!("failed to write `{}`: {error}", config_path.display()),
            )
        })?;
        format!("initialized ish project in `{}`", current_dir.display())
    };

    if json {
        Ok(Some(output_message(message).map_err(json_output_error)?))
    } else {
        Ok(Some(success(&message)))
    }
}

fn archive_command(json: bool) -> Result<Option<String>, AppError> {
    let (_, _, mut store) = load_store_from_current_dir()?;
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

fn check_command(args: CheckArgs, json: bool) -> Result<RunOutcome, AppError> {
    let (_, config, mut store) = load_store_from_current_dir()?;
    let config_checks = validate_config(&config);
    let initial_links = store.check_all_links();
    let issues_found = config_checks.issue_count() + link_issue_count(&initial_links);

    let fixed_links = if args.fix {
        Some(store.fix_broken_links().map_err(|error| {
            AppError::new(
                ErrorCode::FileError,
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

fn project_name(dir: &Path) -> Result<String, AppError> {
    dir.file_name()
        .and_then(|name| name.to_str())
        .filter(|name| !name.is_empty())
        .map(str::to_owned)
        .ok_or_else(|| {
            AppError::new(
                ErrorCode::Validation,
                "failed to derive project name from current directory",
            )
        })
}

fn roadmap_command(args: RoadmapArgs, json: bool) -> Result<Option<String>, AppError> {
    let (current_dir, _, _) = load_store_from_current_dir()?;

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

fn prime_command(json: bool) -> Result<Option<String>, AppError> {
    let current_dir = std::env::current_dir().map_err(|error| {
        AppError::new(
            ErrorCode::FileError,
            format!("failed to determine current directory: {error}"),
        )
    })?;
    let Some(config_path) = find_config(&current_dir) else {
        return Ok(None);
    };

    let config = Config::load(&config_path).map_err(|error| {
        AppError::new(
            ErrorCode::FileError,
            format!("failed to load `{}`: {error}", config_path.display()),
        )
    })?;

    let output = prime_output(&config);
    if json {
        Ok(Some(output_message(output).map_err(json_output_error)?))
    } else {
        Ok(Some(output))
    }
}

fn load_store_from_current_dir() -> Result<(std::path::PathBuf, Config, Store), AppError> {
    let current_dir = std::env::current_dir().map_err(|error| {
        AppError::new(
            ErrorCode::FileError,
            format!("failed to determine current directory: {error}"),
        )
    })?;
    let Some(config_path) = find_config(&current_dir) else {
        return Err(AppError::new(
            ErrorCode::NotFound,
            "no `.ish.yml` found in the current directory or its parents",
        ));
    };

    let config = Config::load(&config_path).map_err(|error| {
        AppError::new(
            ErrorCode::FileError,
            format!("failed to load `{}`: {error}", config_path.display()),
        )
    })?;
    let store_root = config_path
        .parent()
        .ok_or_else(|| {
            AppError::new(
                ErrorCode::FileError,
                format!("invalid config path: {}", config_path.display()),
            )
        })?
        .join(&config.ish.path);
    let mut store = Store::new(&store_root, config.clone()).map_err(|error| {
        AppError::new(
            ErrorCode::FileError,
            format!("failed to open store `{}`: {error}", store_root.display()),
        )
    })?;
    store.load().map_err(|error| {
        AppError::new(
            ErrorCode::FileError,
            format!("failed to load store `{}`: {error}", store_root.display()),
        )
    })?;

    Ok((current_dir, config, store))
}

#[derive(Debug, Clone)]
struct ConfigChecks {
    default_status: Option<String>,
    default_type: Option<String>,
    invalid_colors: Vec<String>,
}

impl ConfigChecks {
    fn issue_count(&self) -> usize {
        usize::from(self.default_status.is_some())
            + usize::from(self.default_type.is_some())
            + self.invalid_colors.len()
    }
}

fn validate_config(config: &Config) -> ConfigChecks {
    let mut invalid_colors = Vec::new();

    for status_name in config.status_names() {
        if let Some(status) = config.get_status(status_name)
            && !is_supported_color_name(status.color)
        {
            invalid_colors.push(format!(
                "status `{}` uses unsupported color `{}`",
                status.name, status.color
            ));
        }
    }

    for type_name in config.type_names() {
        if let Some(ishoo_type) = config.get_type(type_name)
            && !is_supported_color_name(ishoo_type.color)
        {
            invalid_colors.push(format!(
                "type `{}` uses unsupported color `{}`",
                ishoo_type.name, ishoo_type.color
            ));
        }
    }

    for priority_name in config.priority_names() {
        if let Some(priority) = config.get_priority(priority_name)
            && !is_supported_color_name(priority.color)
        {
            invalid_colors.push(format!(
                "priority `{}` uses unsupported color `{}`",
                priority.name, priority.color
            ));
        }
    }

    ConfigChecks {
        default_status: (!config.is_valid_status(&config.ish.default_status))
            .then(|| format!("invalid default_status `{}`", config.ish.default_status)),
        default_type: (!config.is_valid_type(&config.ish.default_type))
            .then(|| format!("invalid default_type `{}`", config.ish.default_type)),
        invalid_colors,
    }
}

fn link_issue_count(result: &LinkCheckResult) -> usize {
    result.broken_links.len() + result.self_links.len() + result.cycles.len()
}

fn render_check_human(
    config_checks: &ConfigChecks,
    initial_links: &LinkCheckResult,
    final_links: &LinkCheckResult,
    fixed_links: Option<usize>,
) -> String {
    let mut lines = vec![format_check_line(
        config_checks.default_status.is_none(),
        "config default_status",
        config_checks.default_status.as_deref().unwrap_or("valid"),
    )];
    lines.push(format_check_line(
        config_checks.default_type.is_none(),
        "config default_type",
        config_checks.default_type.as_deref().unwrap_or("valid"),
    ));

    let color_details = if config_checks.invalid_colors.is_empty() {
        "all configured colors are supported".to_string()
    } else {
        config_checks.invalid_colors.join("; ")
    };
    lines.push(format_check_line(
        config_checks.invalid_colors.is_empty(),
        "config colors",
        &color_details,
    ));

    lines.push(format_check_line(
        initial_links.broken_links.is_empty(),
        "broken links",
        &link_ref_summary(&initial_links.broken_links),
    ));
    lines.push(format_check_line(
        initial_links.self_links.is_empty(),
        "self-references",
        &link_ref_summary(&initial_links.self_links),
    ));
    lines.push(format_check_line(
        initial_links.cycles.is_empty(),
        "cycles",
        &cycle_summary(&initial_links.cycles),
    ));

    if let Some(fixed_links) = fixed_links {
        let fix_message = if fixed_links == 0 {
            warning("No broken links or self-references needed fixing")
        } else {
            success(&format!("Applied --fix to {fixed_links} link(s)"))
        };
        lines.push(fix_message);
        lines.push(format_check_line(
            final_links.broken_links.is_empty(),
            "remaining broken links",
            &link_ref_summary(&final_links.broken_links),
        ));
        lines.push(format_check_line(
            final_links.self_links.is_empty(),
            "remaining self-references",
            &link_ref_summary(&final_links.self_links),
        ));
        lines.push(format_check_line(
            final_links.cycles.is_empty(),
            "remaining cycles",
            &cycle_summary(&final_links.cycles),
        ));
    }

    let issues_found = config_checks.issue_count() + link_issue_count(initial_links);
    let remaining = config_checks.issue_count() + link_issue_count(final_links);
    let fixed = fixed_links.unwrap_or(0);
    let summary = if issues_found == 0 {
        success("Summary: no issues found")
    } else if fixed_links.is_some() {
        warning(&format!(
            "Summary: {issues_found} issue(s) found, {fixed} fixed, {remaining} remaining"
        ))
    } else {
        warning(&format!("Summary: {issues_found} issue(s) found"))
    };
    lines.push(summary);

    lines.join("\n")
}

fn format_check_line(ok: bool, label: &str, details: &str) -> String {
    let icon = if ok { "✓" } else { "✗" };
    let text = format!("{icon} {label}: {details}");
    if ok { success(&text) } else { danger(&text) }
}

fn link_ref_summary(links: &[LinkRef]) -> String {
    if links.is_empty() {
        return "none".to_string();
    }

    links
        .iter()
        .map(|link| {
            format!(
                "{} {} {}",
                link.source_id,
                link_type_label(link.link_type),
                link.target_id
            )
        })
        .collect::<Vec<_>>()
        .join(", ")
}

fn cycle_summary(cycles: &[LinkCycle]) -> String {
    if cycles.is_empty() {
        return "none".to_string();
    }

    cycles
        .iter()
        .map(|cycle| {
            format!(
                "{}: {}",
                link_type_label(cycle.link_type),
                cycle.path.join(" -> ")
            )
        })
        .collect::<Vec<_>>()
        .join(", ")
}

fn link_type_label(link_type: LinkType) -> &'static str {
    match link_type {
        LinkType::Parent => "parent",
        LinkType::Blocking => "blocking",
        LinkType::BlockedBy => "blocked_by",
    }
}

fn render_check_json(
    config_checks: &ConfigChecks,
    initial_links: &LinkCheckResult,
    final_links: &LinkCheckResult,
    fixed_links: Option<usize>,
) -> Result<String, String> {
    output_success(json!({
        "checks": {
            "config": {
                "default_status": {
                    "ok": config_checks.default_status.is_none(),
                    "message": config_checks.default_status.clone().unwrap_or_else(|| "valid".to_string()),
                },
                "default_type": {
                    "ok": config_checks.default_type.is_none(),
                    "message": config_checks.default_type.clone().unwrap_or_else(|| "valid".to_string()),
                },
                "colors": {
                    "ok": config_checks.invalid_colors.is_empty(),
                    "issues": config_checks.invalid_colors,
                }
            },
            "links": {
                "initial": link_checks_json(initial_links),
                "final": link_checks_json(final_links),
            }
        },
        "summary": {
            "issues_found": config_checks.issue_count() + link_issue_count(initial_links),
            "fixed_links": fixed_links.unwrap_or(0),
            "remaining_issues": config_checks.issue_count() + link_issue_count(final_links),
        }
    }))
}

fn link_checks_json(result: &LinkCheckResult) -> Value {
    json!({
        "broken_links": result.broken_links.iter().map(link_ref_json).collect::<Vec<_>>(),
        "self_links": result.self_links.iter().map(link_ref_json).collect::<Vec<_>>(),
        "cycles": result.cycles.iter().map(cycle_json).collect::<Vec<_>>(),
    })
}

fn link_ref_json(link: &LinkRef) -> Value {
    json!({
        "source_id": link.source_id,
        "link_type": link_type_label(link.link_type),
        "target_id": link.target_id,
    })
}

fn cycle_json(cycle: &LinkCycle) -> Value {
    json!({
        "link_type": link_type_label(cycle.link_type),
        "path": cycle.path,
    })
}

fn classify_app_error(message: String) -> AppError {
    let code = if message.contains("no `.ish.yml` found") {
        ErrorCode::NotFound
    } else if message.contains("etag") || message.contains("conflict") {
        ErrorCode::Conflict
    } else if message.contains("invalid") {
        ErrorCode::Validation
    } else {
        ErrorCode::FileError
    };

    AppError::new(code, message)
}

fn store_open_error(store_root: &Path) -> impl Fn(StoreError) -> AppError + '_ {
    move |error| {
        AppError::new(
            ErrorCode::FileError,
            format!("failed to open store `{}`: {error}", store_root.display()),
        )
    }
}

fn store_app_error(error: StoreError) -> AppError {
    let code = match error {
        StoreError::InvalidStatus(_)
        | StoreError::InvalidType(_)
        | StoreError::InvalidPriority(_)
        | StoreError::InvalidTag(_)
        | StoreError::ParentNotAllowed(_)
        | StoreError::InvalidParentType { .. }
        | StoreError::Body(_) => ErrorCode::Validation,
        StoreError::NotFound(_) => ErrorCode::NotFound,
        StoreError::ETagMismatch { .. } => ErrorCode::Conflict,
        StoreError::Io(_)
        | StoreError::InvalidPath(_)
        | StoreError::InvalidFrontmatter(_)
        | StoreError::Yaml { .. } => ErrorCode::FileError,
    };

    AppError::new(code, error.to_string())
}

fn json_output_error(message: String) -> AppError {
    AppError::new(ErrorCode::FileError, message)
}

#[cfg(test)]
mod tests {
    use super::{
        CheckArgs, Cli, Commands, CreateArgs, DeleteArgs, DeleteTarget, ListArgs, RoadmapArgs,
        STORE_GITIGNORE_CONTENT, ShowArgs, UpdateArgs, archive_command, check_command,
        confirm_delete, create_command, delete_command_with_io, init_command, list_command,
        prime_command, roadmap_command, run, show_command, update_command, version_output,
    };
    use crate::config::{CONFIG_FILE_NAME, Config};
    use crate::core::store::{LinkRef, LinkType};
    use crate::model::ishoo::Ishoo;
    use chrono::Utc;
    use clap::Parser;
    use serde_json::Value;
    use std::fs;
    use std::io::Cursor;
    use std::path::{Path, PathBuf};
    use std::process::ExitCode;
    use std::sync::{Mutex, MutexGuard, OnceLock};
    use std::time::{SystemTime, UNIX_EPOCH};

    struct TestDir {
        path: PathBuf,
    }

    impl TestDir {
        fn new() -> Self {
            let unique = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("system time should be after unix epoch")
                .as_nanos();
            let path = std::env::temp_dir().join(format!("ish-main-test-{unique}"));
            fs::create_dir_all(&path).expect("temp dir should be created");

            Self { path }
        }

        fn path(&self) -> &Path {
            &self.path
        }
    }

    impl Drop for TestDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.path);
        }
    }

    struct WorkingDirGuard {
        _lock: MutexGuard<'static, ()>,
        original: PathBuf,
    }

    impl WorkingDirGuard {
        fn change_to(path: &Path) -> Self {
            let lock = cwd_lock()
                .lock()
                .expect("working directory test lock should not be poisoned");
            let original = std::env::current_dir().expect("current directory should be readable");
            std::env::set_current_dir(path).expect("current directory should be changed");
            Self {
                _lock: lock,
                original,
            }
        }
    }

    impl Drop for WorkingDirGuard {
        fn drop(&mut self) {
            let _ = std::env::set_current_dir(&self.original);
        }
    }

    fn cwd_lock() -> &'static Mutex<()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
    }

    fn sample_delete_target(incoming_links: usize) -> DeleteTarget {
        DeleteTarget {
            ishoo: Ishoo {
                id: "ish-target".to_string(),
                slug: "target".to_string(),
                path: "ish-target--target.md".to_string(),
                title: "Target".to_string(),
                status: "todo".to_string(),
                ishoo_type: "task".to_string(),
                priority: Some("normal".to_string()),
                tags: Vec::new(),
                created_at: Utc::now(),
                updated_at: Utc::now(),
                order: None,
                body: "Target body.".to_string(),
                parent: None,
                blocking: Vec::new(),
                blocked_by: Vec::new(),
            },
            incoming_links: (0..incoming_links)
                .map(|index| LinkRef {
                    source_id: format!("ish-ref-{index}"),
                    link_type: LinkType::Blocking,
                    target_id: "ish-target".to_string(),
                })
                .collect(),
        }
    }

    fn write_test_ishoo(
        root: &Path,
        id: &str,
        title: &str,
        status: &str,
        ishoo_type: &str,
        priority: Option<&str>,
        body: &str,
        parent: Option<&str>,
        blocking: &[&str],
        blocked_by: &[&str],
        tags: &[&str],
    ) {
        let mut content = format!(
            "---\n# {id}\ntitle: {title}\nstatus: {status}\ntype: {ishoo_type}\ncreated_at: 2026-01-01T00:00:00Z\nupdated_at: 2026-01-01T00:00:00Z\n"
        );

        if let Some(priority) = priority {
            content.push_str(&format!("priority: {priority}\n"));
        }
        if !tags.is_empty() {
            content.push_str("tags:\n");
            for tag in tags {
                content.push_str(&format!("  - {tag}\n"));
            }
        }
        if let Some(parent) = parent {
            content.push_str(&format!("parent: {parent}\n"));
        }
        if !blocking.is_empty() {
            content.push_str("blocking:\n");
            for blocked in blocking {
                content.push_str(&format!("  - {blocked}\n"));
            }
        }
        if !blocked_by.is_empty() {
            content.push_str("blocked_by:\n");
            for blocker in blocked_by {
                content.push_str(&format!("  - {blocker}\n"));
            }
        }

        content.push_str("---\n\n");
        content.push_str(body);
        content.push('\n');

        fs::write(
            root.join(format!(
                "{id}--{}.md",
                title.to_ascii_lowercase().replace(' ', "-")
            )),
            content,
        )
        .expect("ishoo file should be written");
    }

    #[test]
    fn version_output_uses_package_version() {
        assert_eq!(
            version_output(),
            format!("ish {}", env!("CARGO_PKG_VERSION"))
        );
    }

    #[test]
    fn run_prime_returns_rendered_guide_when_config_exists() {
        let temp = TestDir::new();
        let mut config = Config::default();
        config.project.name = "Prime Test".to_string();
        config.save(temp.path()).expect("config should save");
        let _guard = WorkingDirGuard::change_to(temp.path());

        let output = run(Cli {
            json: false,
            command: Some(Commands::Prime),
        })
        .expect("prime command should succeed")
        .output
        .expect("prime command should print output");

        assert!(output.contains("# ish Agent Guide"));
        assert!(output.contains("Prime Test"));
    }

    #[test]
    fn create_command_uses_defaults_and_writes_file() {
        let temp = TestDir::new();
        let mut config = Config::default_with_prefix("demo");
        config.project.name = "Create Test".to_string();
        config.save(temp.path()).expect("config should save");
        let _guard = WorkingDirGuard::change_to(temp.path());

        let output = create_command(
            CreateArgs {
                title: Some("Ship feature".to_string()),
                status: None,
                ishoo_type: None,
                priority: None,
                body: None,
                body_file: None,
                tags: Vec::new(),
                parent: None,
                blocking: Vec::new(),
                blocked_by: Vec::new(),
                prefix: None,
            },
            false,
        )
        .expect("create command should succeed")
        .expect("create command should print output");

        assert!(output.contains("Created"));

        let files = fs::read_dir(temp.path().join(".ish"))
            .expect("store root should exist")
            .map(|entry| entry.expect("entry should read").path())
            .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("md"))
            .collect::<Vec<_>>();
        assert_eq!(files.len(), 1);

        let contents = fs::read_to_string(&files[0]).expect("created file should be readable");
        assert!(contents.contains("# demo-"));
        assert!(contents.contains("title: Ship feature"));
        assert!(contents.contains("status: todo"));
        assert!(contents.contains("type: task"));
        assert!(contents.contains("priority: normal"));
    }

    #[test]
    fn create_command_supports_body_file_tags_and_relations_in_json_mode() {
        let temp = TestDir::new();
        let mut config = Config::default_with_prefix("ish");
        config.project.name = "Create JSON Test".to_string();
        config.save(temp.path()).expect("config should save");
        let store_root = temp.path().join(".ish");
        fs::create_dir_all(&store_root).expect("store root should exist");
        fs::write(
            store_root.join("ish-parent--parent.md"),
            "---\n# ish-parent\ntitle: Parent\nstatus: todo\ntype: feature\ncreated_at: 2026-01-01T00:00:00Z\nupdated_at: 2026-01-01T00:00:00Z\n---\n\nParent body.\n",
        )
        .expect("parent file should exist");
        fs::write(
            store_root.join("ish-blocker--blocker.md"),
            "---\n# ish-blocker\ntitle: Blocker\nstatus: todo\ntype: task\ncreated_at: 2026-01-01T00:00:00Z\nupdated_at: 2026-01-01T00:00:00Z\n---\n\nBlocker body.\n",
        )
        .expect("blocker file should exist");
        fs::write(
            store_root.join("ish-dependency--dependency.md"),
            "---\n# ish-dependency\ntitle: Dependency\nstatus: todo\ntype: task\ncreated_at: 2026-01-01T00:00:00Z\nupdated_at: 2026-01-01T00:00:00Z\n---\n\nDependency body.\n",
        )
        .expect("dependency file should exist");
        let body_path = temp.path().join("body.md");
        fs::write(&body_path, "Body from file\n").expect("body file should be written");
        let _guard = WorkingDirGuard::change_to(temp.path());

        let output = create_command(
            CreateArgs {
                title: Some("Wire command".to_string()),
                status: Some("in-progress".to_string()),
                ishoo_type: Some("task".to_string()),
                priority: Some("high".to_string()),
                body: None,
                body_file: Some(body_path.display().to_string()),
                tags: vec!["cli".to_string(), "json".to_string()],
                parent: Some("parent".to_string()),
                blocking: vec!["blocker".to_string()],
                blocked_by: vec!["dependency".to_string()],
                prefix: Some("feat".to_string()),
            },
            true,
        )
        .expect("create command should succeed")
        .expect("create command should print output");

        let parsed: Value = serde_json::from_str(&output).expect("json should parse");
        assert_eq!(parsed["success"], Value::Bool(true));
        assert_eq!(
            parsed["data"]["title"],
            Value::String("Wire command".to_string())
        );
        assert_eq!(
            parsed["data"]["status"],
            Value::String("in-progress".to_string())
        );
        assert_eq!(parsed["data"]["type"], Value::String("task".to_string()));
        assert_eq!(
            parsed["data"]["priority"],
            Value::String("high".to_string())
        );
        assert_eq!(
            parsed["data"]["body"],
            Value::String("Body from file\n".to_string())
        );
        assert_eq!(
            parsed["data"]["parent"],
            Value::String("ish-parent".to_string())
        );
        assert_eq!(
            parsed["data"]["blocking"][0],
            Value::String("ish-blocker".to_string())
        );
        assert_eq!(
            parsed["data"]["blocked_by"][0],
            Value::String("ish-dependency".to_string())
        );
        assert_eq!(parsed["data"]["tags"][0], Value::String("cli".to_string()));
        assert!(
            parsed["data"]["id"]
                .as_str()
                .expect("id should be present")
                .starts_with("feat-")
        );
    }

    #[test]
    fn create_command_defaults_title_to_untitled() {
        let temp = TestDir::new();
        let config = Config::default();
        config.save(temp.path()).expect("config should save");
        let _guard = WorkingDirGuard::change_to(temp.path());

        create_command(
            CreateArgs {
                title: None,
                status: None,
                ishoo_type: None,
                priority: None,
                body: None,
                body_file: None,
                tags: Vec::new(),
                parent: None,
                blocking: Vec::new(),
                blocked_by: Vec::new(),
                prefix: None,
            },
            false,
        )
        .expect("create command should succeed");

        let files = fs::read_dir(temp.path().join(".ish"))
            .expect("store root should exist")
            .map(|entry| entry.expect("entry should read").path())
            .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("md"))
            .collect::<Vec<_>>();
        assert_eq!(files.len(), 1);
        let contents = fs::read_to_string(&files[0]).expect("created file should be readable");
        assert!(contents.contains("title: Untitled"));
    }

    #[test]
    fn list_command_json_filters_and_omits_body_by_default() {
        let temp = TestDir::new();
        let config = Config::default();
        config.save(temp.path()).expect("config should save");
        let store_root = temp.path().join(".ish");
        fs::create_dir_all(&store_root).expect("store root should exist");
        write_test_ishoo(
            &store_root,
            "ish-alpha",
            "Alpha task",
            "todo",
            "task",
            Some("high"),
            "Matches search body.",
            None,
            &[],
            &[],
            &["cli"],
        );
        write_test_ishoo(
            &store_root,
            "ish-beta",
            "Beta bug",
            "todo",
            "bug",
            Some("normal"),
            "Other body.",
            None,
            &[],
            &[],
            &["backend"],
        );
        let _guard = WorkingDirGuard::change_to(temp.path());

        let output = list_command(
            ListArgs {
                status: vec!["todo".to_string()],
                no_status: Vec::new(),
                ishoo_type: vec!["task".to_string()],
                no_type: Vec::new(),
                priority: vec!["high".to_string()],
                no_priority: Vec::new(),
                tag: vec!["CLI".to_string()],
                no_tag: Vec::new(),
                has_parent: false,
                no_parent: false,
                parent: None,
                has_blocking: false,
                no_blocking: false,
                is_blocked: false,
                ready: false,
                search: Some("matches".to_string()),
                sort: Some(super::ListSortArg::Id),
                quiet: false,
                full: false,
            },
            true,
        )
        .expect("list command should succeed")
        .expect("list command should print output");

        let parsed: Value = serde_json::from_str(&output).expect("json should parse");
        assert_eq!(parsed["success"], Value::Bool(true));
        assert_eq!(parsed["count"], Value::from(1));
        assert_eq!(parsed["ishoos"][0]["id"], "ish-alpha");
        assert!(parsed["ishoos"][0].get("body").is_none());
    }

    #[test]
    fn list_command_full_json_includes_body() {
        let temp = TestDir::new();
        let config = Config::default();
        config.save(temp.path()).expect("config should save");
        let store_root = temp.path().join(".ish");
        fs::create_dir_all(&store_root).expect("store root should exist");
        write_test_ishoo(
            &store_root,
            "ish-alpha",
            "Alpha task",
            "todo",
            "task",
            Some("normal"),
            "Detailed body.",
            None,
            &[],
            &[],
            &[],
        );
        let _guard = WorkingDirGuard::change_to(temp.path());

        let output = list_command(
            ListArgs {
                status: Vec::new(),
                no_status: Vec::new(),
                ishoo_type: Vec::new(),
                no_type: Vec::new(),
                priority: Vec::new(),
                no_priority: Vec::new(),
                tag: Vec::new(),
                no_tag: Vec::new(),
                has_parent: false,
                no_parent: false,
                parent: None,
                has_blocking: false,
                no_blocking: false,
                is_blocked: false,
                ready: false,
                search: None,
                sort: Some(super::ListSortArg::Id),
                quiet: false,
                full: true,
            },
            true,
        )
        .expect("list command should succeed")
        .expect("list command should print output");

        let parsed: Value = serde_json::from_str(&output).expect("json should parse");
        assert_eq!(parsed["ishoos"][0]["body"], "Detailed body.");
    }

    #[test]
    fn list_command_ready_excludes_blocked_in_progress_archived_and_implicitly_completed() {
        let temp = TestDir::new();
        let config = Config::default();
        config.save(temp.path()).expect("config should save");
        let store_root = temp.path().join(".ish");
        fs::create_dir_all(&store_root).expect("store root should exist");
        write_test_ishoo(
            &store_root,
            "ish-ready",
            "Ready item",
            "todo",
            "task",
            Some("normal"),
            "Body.",
            None,
            &[],
            &[],
            &[],
        );
        write_test_ishoo(
            &store_root,
            "ish-blocker",
            "Blocker",
            "in-progress",
            "task",
            Some("normal"),
            "Body.",
            None,
            &[],
            &[],
            &[],
        );
        write_test_ishoo(
            &store_root,
            "ish-blocked",
            "Blocked item",
            "todo",
            "task",
            Some("normal"),
            "Body.",
            None,
            &[],
            &["ish-blocker"],
            &[],
        );
        write_test_ishoo(
            &store_root,
            "ish-active",
            "Active item",
            "in-progress",
            "task",
            Some("normal"),
            "Body.",
            None,
            &[],
            &[],
            &[],
        );
        write_test_ishoo(
            &store_root,
            "ish-parent",
            "Completed parent",
            "completed",
            "feature",
            Some("normal"),
            "Body.",
            None,
            &[],
            &[],
            &[],
        );
        write_test_ishoo(
            &store_root,
            "ish-child",
            "Child item",
            "todo",
            "task",
            Some("normal"),
            "Body.",
            Some("ish-parent"),
            &[],
            &[],
            &[],
        );
        let _guard = WorkingDirGuard::change_to(temp.path());

        let output = list_command(
            ListArgs {
                status: Vec::new(),
                no_status: Vec::new(),
                ishoo_type: Vec::new(),
                no_type: Vec::new(),
                priority: Vec::new(),
                no_priority: Vec::new(),
                tag: Vec::new(),
                no_tag: Vec::new(),
                has_parent: false,
                no_parent: false,
                parent: None,
                has_blocking: false,
                no_blocking: false,
                is_blocked: false,
                ready: true,
                search: None,
                sort: Some(super::ListSortArg::Id),
                quiet: true,
                full: false,
            },
            false,
        )
        .expect("list command should succeed")
        .expect("list command should print output");

        assert_eq!(output.trim(), "ish-ready");
    }

    #[test]
    fn list_command_human_output_renders_tree_with_context_parent() {
        let temp = TestDir::new();
        let config = Config::default();
        config.save(temp.path()).expect("config should save");
        let store_root = temp.path().join(".ish");
        fs::create_dir_all(&store_root).expect("store root should exist");
        write_test_ishoo(
            &store_root,
            "ish-parent",
            "Parent",
            "todo",
            "feature",
            Some("normal"),
            "Parent body.",
            None,
            &[],
            &[],
            &[],
        );
        write_test_ishoo(
            &store_root,
            "ish-child",
            "Child",
            "todo",
            "task",
            Some("high"),
            "Child body.",
            Some("ish-parent"),
            &[],
            &[],
            &["cli"],
        );
        let _guard = WorkingDirGuard::change_to(temp.path());

        let output = list_command(
            ListArgs {
                status: Vec::new(),
                no_status: Vec::new(),
                ishoo_type: Vec::new(),
                no_type: Vec::new(),
                priority: Vec::new(),
                no_priority: Vec::new(),
                tag: vec!["cli".to_string()],
                no_tag: Vec::new(),
                has_parent: false,
                no_parent: false,
                parent: None,
                has_blocking: false,
                no_blocking: false,
                is_blocked: false,
                ready: false,
                search: None,
                sort: None,
                quiet: false,
                full: false,
            },
            false,
        )
        .expect("list command should succeed")
        .expect("list command should print output");

        assert!(output.contains("ish-parent"));
        assert!(output.contains("ish-child"));
        assert!(output.contains("└──"));
    }

    #[test]
    fn show_command_supports_json_raw_body_and_etag_output() {
        let temp = TestDir::new();
        let config = Config::default();
        config.save(temp.path()).expect("config should save");
        let store_root = temp.path().join(".ish");
        fs::create_dir_all(&store_root).expect("store root should exist");
        write_test_ishoo(
            &store_root,
            "ish-parent",
            "Parent",
            "todo",
            "feature",
            Some("normal"),
            "Parent body.",
            None,
            &[],
            &[],
            &["context"],
        );
        write_test_ishoo(
            &store_root,
            "ish-blocker",
            "Blocker",
            "todo",
            "task",
            Some("high"),
            "Blocker body.",
            None,
            &[],
            &[],
            &[],
        );
        write_test_ishoo(
            &store_root,
            "ish-target",
            "Target",
            "todo",
            "task",
            Some("normal"),
            "# Heading\n\nBody text.",
            Some("ish-parent"),
            &["ish-blocker"],
            &[],
            &["cli", "show"],
        );
        let _guard = WorkingDirGuard::change_to(temp.path());

        let json_output = show_command(
            ShowArgs {
                ids: vec!["target".to_string()],
                raw: false,
                body_only: false,
                etag_only: false,
            },
            true,
        )
        .expect("show command should succeed")
        .expect("show command should print output");
        let parsed: Value = serde_json::from_str(&json_output).expect("json should parse");
        assert_eq!(parsed["success"], Value::Bool(true));
        assert_eq!(parsed["count"], Value::from(1));
        assert_eq!(parsed["ishoos"][0]["id"], "ish-target");
        assert_eq!(parsed["ishoos"][0]["parent"], "ish-parent");
        assert_eq!(parsed["ishoos"][0]["blocking"][0], "ish-blocker");
        assert!(parsed["ishoos"][0].get("etag").is_some());

        let raw_output = show_command(
            ShowArgs {
                ids: vec!["target".to_string()],
                raw: true,
                body_only: false,
                etag_only: false,
            },
            false,
        )
        .expect("show command should succeed")
        .expect("show command should print output");
        assert!(raw_output.starts_with("---\n# ish-target"));
        assert!(raw_output.contains("title: Target"));

        let body_output = show_command(
            ShowArgs {
                ids: vec!["target".to_string()],
                raw: false,
                body_only: true,
                etag_only: false,
            },
            false,
        )
        .expect("show command should succeed")
        .expect("show command should print output");
        assert_eq!(body_output, "# Heading\n\nBody text.");

        let etag_output = show_command(
            ShowArgs {
                ids: vec!["target".to_string()],
                raw: false,
                body_only: false,
                etag_only: true,
            },
            false,
        )
        .expect("show command should succeed")
        .expect("show command should print output");
        assert_eq!(etag_output.len(), 16);
        assert!(etag_output.chars().all(|ch| ch.is_ascii_hexdigit()));
    }

    #[test]
    fn show_command_human_output_renders_header_relationships_and_separator() {
        let temp = TestDir::new();
        let config = Config::default();
        config.save(temp.path()).expect("config should save");
        let store_root = temp.path().join(".ish");
        fs::create_dir_all(&store_root).expect("store root should exist");
        write_test_ishoo(
            &store_root,
            "ish-parent",
            "Parent",
            "completed",
            "feature",
            Some("normal"),
            "Parent body.",
            None,
            &[],
            &[],
            &[],
        );
        write_test_ishoo(
            &store_root,
            "ish-blocker",
            "Blocker",
            "todo",
            "task",
            Some("high"),
            "Blocker body.",
            None,
            &[],
            &[],
            &[],
        );
        write_test_ishoo(
            &store_root,
            "ish-child",
            "Child",
            "todo",
            "task",
            Some("normal"),
            "Child body.",
            Some("ish-parent"),
            &["ish-blocker"],
            &[],
            &["demo"],
        );
        let _guard = WorkingDirGuard::change_to(temp.path());

        let output = show_command(
            ShowArgs {
                ids: vec!["child".to_string(), "parent".to_string()],
                raw: false,
                body_only: false,
                etag_only: false,
            },
            false,
        )
        .expect("show command should succeed")
        .expect("show command should print output");

        assert!(output.contains("ish-child"));
        assert!(output.contains("Path: ish-child--child.md"));
        assert!(output.contains("Parent: ish-parent"));
        assert!(output.contains("Blocking: ish-blocker"));
        assert!(output.contains("Inherited status: completed from ish-parent"));
        assert!(output.contains("#demo"));
        assert!(output.contains("Child body."));
        assert!(output.contains("════════"));
    }

    #[test]
    fn confirm_delete_prints_title_path_and_incoming_link_count() {
        let mut input = Cursor::new(b"yes\n".to_vec());
        let mut output = Vec::new();

        let confirmed = confirm_delete(&[sample_delete_target(2)], &mut input, &mut output)
            .expect("confirmation prompt should succeed");
        let rendered = String::from_utf8(output).expect("prompt should be utf8");

        assert!(confirmed);
        assert!(rendered.contains("Delete 1 ishoo?"));
        assert!(rendered.contains("title: Target"));
        assert!(rendered.contains("path: ish-target--target.md"));
        assert!(rendered.contains("incoming links: 2"));
        assert!(rendered.contains("remove 2 incoming link(s)"));
        assert!(rendered.contains("Proceed? [y/N]:"));
    }

    #[test]
    fn delete_command_force_removes_file_and_cleans_references() {
        let temp = TestDir::new();
        let config = Config::default();
        config.save(temp.path()).expect("config should save");
        let store_root = temp.path().join(".ish");
        fs::create_dir_all(&store_root).expect("store root should exist");
        fs::write(
            store_root.join("ish-target--target.md"),
            "---\n# ish-target\ntitle: Target\nstatus: todo\ntype: task\ncreated_at: 2026-01-01T00:00:00Z\nupdated_at: 2026-01-01T00:00:00Z\n---\n\nTarget body.\n",
        )
        .expect("target file should exist");
        fs::write(
            store_root.join("ish-ref--ref.md"),
            "---\n# ish-ref\ntitle: Ref\nstatus: todo\ntype: task\ncreated_at: 2026-01-01T00:00:00Z\nupdated_at: 2026-01-01T00:00:00Z\nparent: ish-target\nblocking:\n  - ish-target\nblocked_by:\n  - ish-target\n---\n\nRef body.\n",
        )
        .expect("ref file should exist");
        let _guard = WorkingDirGuard::change_to(temp.path());

        let output = delete_command_with_io(
            DeleteArgs {
                ids: vec!["target".to_string()],
                force: true,
            },
            false,
            &mut Cursor::new(Vec::new()),
            &mut Vec::new(),
        )
        .expect("delete command should succeed")
        .expect("delete command should print output");

        assert!(output.contains("Deleted"));
        assert!(output.contains("cleaned 3 incoming link(s)"));
        assert!(!store_root.join("ish-target--target.md").exists());

        let ref_contents =
            fs::read_to_string(store_root.join("ish-ref--ref.md")).expect("ref file should exist");
        assert!(!ref_contents.contains("parent: ish-target"));
        assert!(!ref_contents.contains("- ish-target"));
    }

    #[test]
    fn delete_command_returns_cancelled_message_without_deleting() {
        let temp = TestDir::new();
        let config = Config::default();
        config.save(temp.path()).expect("config should save");
        let store_root = temp.path().join(".ish");
        fs::create_dir_all(&store_root).expect("store root should exist");
        fs::write(
            store_root.join("ish-target--target.md"),
            "---\n# ish-target\ntitle: Target\nstatus: todo\ntype: task\ncreated_at: 2026-01-01T00:00:00Z\nupdated_at: 2026-01-01T00:00:00Z\n---\n\nTarget body.\n",
        )
        .expect("target file should exist");
        let _guard = WorkingDirGuard::change_to(temp.path());
        let mut prompt_output = Vec::new();

        let output = delete_command_with_io(
            DeleteArgs {
                ids: vec!["target".to_string()],
                force: false,
            },
            false,
            &mut Cursor::new(b"n\n".to_vec()),
            &mut prompt_output,
        )
        .expect("delete command should succeed")
        .expect("delete command should print output");

        assert!(output.contains("Delete cancelled"));
        assert!(store_root.join("ish-target--target.md").exists());
        let prompt_output = String::from_utf8(prompt_output).expect("prompt should be utf8");
        assert!(prompt_output.contains("title: Target"));
    }

    #[test]
    fn delete_command_json_implies_force_and_returns_deleted_items() {
        let temp = TestDir::new();
        let config = Config::default();
        config.save(temp.path()).expect("config should save");
        let store_root = temp.path().join(".ish");
        fs::create_dir_all(&store_root).expect("store root should exist");
        fs::write(
            store_root.join("ish-target--target.md"),
            "---\n# ish-target\ntitle: Target\nstatus: todo\ntype: task\ncreated_at: 2026-01-01T00:00:00Z\nupdated_at: 2026-01-01T00:00:00Z\n---\n\nTarget body.\n",
        )
        .expect("target file should exist");
        let _guard = WorkingDirGuard::change_to(temp.path());
        let mut prompt_output = Vec::new();

        let output = delete_command_with_io(
            DeleteArgs {
                ids: vec!["target".to_string()],
                force: false,
            },
            true,
            &mut Cursor::new(Vec::new()),
            &mut prompt_output,
        )
        .expect("delete command should succeed")
        .expect("delete command should print output");

        let parsed: Value = serde_json::from_str(&output).expect("json should parse");
        assert_eq!(parsed["success"], Value::Bool(true));
        assert_eq!(parsed["data"]["count"], Value::from(1));
        assert_eq!(parsed["data"]["cleaned_links"], Value::from(0));
        assert_eq!(parsed["data"]["deleted"][0]["id"], "ish-target");
        assert!(prompt_output.is_empty());
        assert!(!store_root.join("ish-target--target.md").exists());
    }

    #[test]
    fn cli_parses_delete_alias() {
        let cli = Cli::try_parse_from(["ish", "rm", "target"])
            .expect("delete alias should parse successfully");

        match cli.command {
            Some(Commands::Delete(args)) => {
                assert_eq!(args.ids, vec!["target".to_string()]);
                assert!(!args.force);
            }
            _ => panic!("expected delete command"),
        }
    }

    #[test]
    fn cli_parses_list_alias() {
        let cli = Cli::try_parse_from(["ish", "ls", "--ready"])
            .expect("list alias should parse successfully");

        match cli.command {
            Some(Commands::List(args)) => {
                assert!(args.ready);
                assert!(!args.quiet);
            }
            _ => panic!("expected list command"),
        }
    }

    #[test]
    fn cli_parses_show_flags() {
        let cli = Cli::try_parse_from(["ish", "show", "abcd", "efgh", "--body-only"])
            .expect("show command should parse successfully");

        match cli.command {
            Some(Commands::Show(args)) => {
                assert_eq!(args.ids, vec!["abcd".to_string(), "efgh".to_string()]);
                assert!(args.body_only);
                assert!(!args.raw);
                assert!(!args.etag_only);
            }
            _ => panic!("expected show command"),
        }
    }

    #[test]
    fn create_command_rejects_invalid_status() {
        let temp = TestDir::new();
        let config = Config::default();
        config.save(temp.path()).expect("config should save");
        let _guard = WorkingDirGuard::change_to(temp.path());

        let error = create_command(
            CreateArgs {
                title: Some("Broken".to_string()),
                status: Some("invalid".to_string()),
                ishoo_type: None,
                priority: None,
                body: None,
                body_file: None,
                tags: Vec::new(),
                parent: None,
                blocking: Vec::new(),
                blocked_by: Vec::new(),
                prefix: None,
            },
            false,
        )
        .expect_err("create command should fail");

        assert_eq!(error.code, crate::output::ErrorCode::Validation);
        assert!(error.message.contains("invalid status"));
    }

    #[test]
    fn update_command_applies_field_changes_and_returns_json() {
        let temp = TestDir::new();
        let config = Config::default();
        config.save(temp.path()).expect("config should save");
        let store_root = temp.path().join(".ish");
        fs::create_dir_all(&store_root).expect("store root should exist");
        write_test_ishoo(
            &store_root,
            "ish-parent",
            "Parent",
            "todo",
            "feature",
            Some("normal"),
            "Parent body.",
            None,
            &[],
            &[],
            &[],
        );
        write_test_ishoo(
            &store_root,
            "ish-blocker",
            "Blocker",
            "todo",
            "task",
            Some("normal"),
            "Blocker body.",
            None,
            &[],
            &[],
            &[],
        );
        write_test_ishoo(
            &store_root,
            "ish-dependency",
            "Dependency",
            "todo",
            "task",
            Some("normal"),
            "Dependency body.",
            None,
            &[],
            &[],
            &[],
        );
        write_test_ishoo(
            &store_root,
            "ish-target",
            "Original title",
            "todo",
            "task",
            Some("normal"),
            "Alpha target",
            None,
            &[],
            &[],
            &["old-tag"],
        );
        let _guard = WorkingDirGuard::change_to(temp.path());

        let output = update_command(
            UpdateArgs {
                id: "target".to_string(),
                status: Some("in-progress".to_string()),
                ishoo_type: Some("bug".to_string()),
                priority: Some("high".to_string()),
                title: Some("Updated title".to_string()),
                body: None,
                body_file: None,
                body_replace_old: Some("target".to_string()),
                body_replace_new: Some("replacement".to_string()),
                body_append: Some("Appended text".to_string()),
                parent: Some("parent".to_string()),
                remove_parent: false,
                blocking: vec!["blocker".to_string()],
                remove_blocking: Vec::new(),
                blocked_by: vec!["dependency".to_string()],
                remove_blocked_by: Vec::new(),
                tags: vec!["new-tag".to_string()],
                remove_tags: vec!["old-tag".to_string()],
                if_match: None,
            },
            true,
        )
        .expect("update command should succeed")
        .expect("update command should print output");

        let parsed: Value = serde_json::from_str(&output).expect("json should parse");
        assert_eq!(parsed["success"], Value::Bool(true));
        assert_eq!(
            parsed["data"]["id"],
            Value::String("ish-target".to_string())
        );
        assert_eq!(
            parsed["data"]["status"],
            Value::String("in-progress".to_string())
        );
        assert_eq!(parsed["data"]["type"], Value::String("bug".to_string()));
        assert_eq!(
            parsed["data"]["priority"],
            Value::String("high".to_string())
        );
        assert_eq!(
            parsed["data"]["title"],
            Value::String("Updated title".to_string())
        );
        assert_eq!(
            parsed["data"]["body"],
            Value::String("Alpha replacement\n\nAppended text".to_string())
        );
        assert_eq!(
            parsed["data"]["parent"],
            Value::String("ish-parent".to_string())
        );
        assert_eq!(
            parsed["data"]["blocking"][0],
            Value::String("ish-blocker".to_string())
        );
        assert_eq!(
            parsed["data"]["blocked_by"][0],
            Value::String("ish-dependency".to_string())
        );
        assert_eq!(
            parsed["data"]["tags"][0],
            Value::String("new-tag".to_string())
        );
        assert!(store_root.join("ish-target--updated-title.md").exists());
        assert!(!store_root.join("ish-target--original-title.md").exists());
    }

    #[test]
    fn update_command_supports_body_append_priority_none_and_relation_removals() {
        let temp = TestDir::new();
        let config = Config::default();
        config.save(temp.path()).expect("config should save");
        let store_root = temp.path().join(".ish");
        fs::create_dir_all(&store_root).expect("store root should exist");
        write_test_ishoo(
            &store_root,
            "ish-target",
            "Target",
            "todo",
            "task",
            Some("normal"),
            "Body.",
            Some("ish-parent"),
            &["ish-blocker"],
            &["ish-dependency"],
            &["cli"],
        );
        write_test_ishoo(
            &store_root,
            "ish-parent",
            "Parent",
            "todo",
            "feature",
            Some("normal"),
            "Parent.",
            None,
            &[],
            &[],
            &[],
        );
        let _guard = WorkingDirGuard::change_to(temp.path());

        let changes = super::resolve_update_changes(UpdateArgs {
            id: "target".to_string(),
            status: None,
            ishoo_type: None,
            priority: Some("none".to_string()),
            title: None,
            body: None,
            body_file: None,
            body_replace_old: None,
            body_replace_new: None,
            body_append: Some("From stdin".to_string()),
            parent: None,
            remove_parent: true,
            blocking: Vec::new(),
            remove_blocking: vec!["blocker".to_string()],
            blocked_by: Vec::new(),
            remove_blocked_by: vec!["dependency".to_string()],
            tags: Vec::new(),
            remove_tags: vec!["cli".to_string()],
            if_match: None,
        })
        .expect("changes should resolve");

        let (_, _, mut store) = super::load_store_from_current_dir().expect("store should load");
        let updated = store
            .update(&changes.0, changes.1)
            .expect("store update should succeed");

        assert_eq!(updated.priority, None);
        assert_eq!(updated.parent, None);
        assert!(updated.blocking.is_empty());
        assert!(updated.blocked_by.is_empty());
        assert!(updated.tags.is_empty());
        assert_eq!(updated.body, "Body.\n\nFrom stdin");
    }

    #[test]
    fn read_text_input_reads_body_append_from_stdin() {
        let value = super::read_text_input(Cursor::new(b"stdin body\n".to_vec()), "body append")
            .expect("stdin text should be read");

        assert_eq!(value, "stdin body\n");
    }

    #[test]
    fn update_command_rejects_when_no_changes_specified() {
        let error = super::resolve_update_changes(UpdateArgs {
            id: "target".to_string(),
            status: None,
            ishoo_type: None,
            priority: None,
            title: None,
            body: None,
            body_file: None,
            body_replace_old: None,
            body_replace_new: None,
            body_append: None,
            parent: None,
            remove_parent: false,
            blocking: Vec::new(),
            remove_blocking: Vec::new(),
            blocked_by: Vec::new(),
            remove_blocked_by: Vec::new(),
            tags: Vec::new(),
            remove_tags: Vec::new(),
            if_match: None,
        })
        .expect_err("missing updates should fail");

        assert_eq!(error.code, crate::output::ErrorCode::Validation);
        assert_eq!(error.message, "no changes specified");
    }

    #[test]
    fn update_command_reports_etag_conflict() {
        let temp = TestDir::new();
        let config = Config::default();
        config.save(temp.path()).expect("config should save");
        let store_root = temp.path().join(".ish");
        fs::create_dir_all(&store_root).expect("store root should exist");
        write_test_ishoo(
            &store_root,
            "ish-target",
            "Target",
            "todo",
            "task",
            Some("normal"),
            "Body.",
            None,
            &[],
            &[],
            &[],
        );
        let _guard = WorkingDirGuard::change_to(temp.path());

        let error = update_command(
            UpdateArgs {
                id: "target".to_string(),
                status: Some("in-progress".to_string()),
                ishoo_type: None,
                priority: None,
                title: None,
                body: None,
                body_file: None,
                body_replace_old: None,
                body_replace_new: None,
                body_append: None,
                parent: None,
                remove_parent: false,
                blocking: Vec::new(),
                remove_blocking: Vec::new(),
                blocked_by: Vec::new(),
                remove_blocked_by: Vec::new(),
                tags: Vec::new(),
                remove_tags: Vec::new(),
                if_match: Some("deadbeefdeadbeef".to_string()),
            },
            false,
        )
        .expect_err("etag mismatch should fail");

        assert_eq!(error.code, crate::output::ErrorCode::Conflict);
        assert!(error.message.contains("etag mismatch"));
    }

    #[test]
    fn update_command_auto_unarchives_before_updating() {
        let temp = TestDir::new();
        let config = Config::default();
        config.save(temp.path()).expect("config should save");
        let archive_root = temp.path().join(".ish").join("archive");
        fs::create_dir_all(&archive_root).expect("archive root should exist");
        write_test_ishoo(
            &archive_root,
            "ish-target",
            "Archived title",
            "completed",
            "task",
            Some("normal"),
            "Archived body.",
            None,
            &[],
            &[],
            &[],
        );
        let _guard = WorkingDirGuard::change_to(temp.path());

        let output = update_command(
            UpdateArgs {
                id: "target".to_string(),
                status: Some("todo".to_string()),
                ishoo_type: None,
                priority: None,
                title: Some("Restored title".to_string()),
                body: None,
                body_file: None,
                body_replace_old: None,
                body_replace_new: None,
                body_append: None,
                parent: None,
                remove_parent: false,
                blocking: Vec::new(),
                remove_blocking: Vec::new(),
                blocked_by: Vec::new(),
                remove_blocked_by: Vec::new(),
                tags: Vec::new(),
                remove_tags: Vec::new(),
                if_match: None,
            },
            false,
        )
        .expect("update command should succeed")
        .expect("update command should print output");

        assert!(output.contains("Updated"));
        assert!(
            temp.path()
                .join(".ish/ish-target--restored-title.md")
                .exists()
        );
        assert!(
            !temp
                .path()
                .join(".ish/archive/ish-target--archived-title.md")
                .exists()
        );
    }

    #[test]
    fn run_archive_moves_completed_ishoos() {
        let temp = TestDir::new();
        let mut config = Config::default();
        config.project.name = "Archive Test".to_string();
        config.save(temp.path()).expect("config should save");
        let store_root = temp.path().join(".ish");
        fs::create_dir_all(&store_root).expect("store root should exist");
        fs::write(
            store_root.join("ish-done--completed.md"),
            "---\n# ish-done\ntitle: Done\nstatus: completed\ntype: task\ncreated_at: 2026-01-01T00:00:00Z\nupdated_at: 2026-01-01T00:00:00Z\n---\n\nDone body.\n",
        )
        .expect("completed file should exist");
        fs::write(
            store_root.join("ish-todo--todo.md"),
            "---\n# ish-todo\ntitle: Todo\nstatus: todo\ntype: task\ncreated_at: 2026-01-01T00:00:00Z\nupdated_at: 2026-01-01T00:00:00Z\n---\n\nTodo body.\n",
        )
        .expect("todo file should exist");
        let _guard = WorkingDirGuard::change_to(temp.path());

        let output = run(Cli {
            json: false,
            command: Some(Commands::Archive),
        })
        .expect("archive command should succeed")
        .output
        .expect("archive command should print output");

        assert!(output.contains("archived 1 ishoo"));
        assert!(store_root.join("archive/ish-done--completed.md").exists());
        assert!(!store_root.join("ish-done--completed.md").exists());
        assert!(store_root.join("ish-todo--todo.md").exists());
    }

    #[test]
    fn archive_command_returns_noop_message_when_nothing_matches() {
        let temp = TestDir::new();
        let mut config = Config::default();
        config.project.name = "Archive Empty Test".to_string();
        config.save(temp.path()).expect("config should save");
        let store_root = temp.path().join(".ish");
        fs::create_dir_all(&store_root).expect("store root should exist");
        fs::write(
            store_root.join("ish-todo--todo.md"),
            "---\n# ish-todo\ntitle: Todo\nstatus: todo\ntype: task\ncreated_at: 2026-01-01T00:00:00Z\nupdated_at: 2026-01-01T00:00:00Z\n---\n\nTodo body.\n",
        )
        .expect("todo file should exist");
        let _guard = WorkingDirGuard::change_to(temp.path());

        let output = archive_command(false)
            .expect("archive command should succeed")
            .expect("archive command should print output");

        assert!(output.contains("no completed or scrapped ishoos to archive"));
        assert!(!store_root.join("archive").exists());
    }

    #[test]
    fn archive_command_wraps_archived_count_in_json_mode() {
        let temp = TestDir::new();
        let mut config = Config::default();
        config.project.name = "Archive JSON Test".to_string();
        config.save(temp.path()).expect("config should save");
        let store_root = temp.path().join(".ish");
        fs::create_dir_all(&store_root).expect("store root should exist");
        fs::write(
            store_root.join("ish-nope--scrapped.md"),
            "---\n# ish-nope\ntitle: Nope\nstatus: scrapped\ntype: task\ncreated_at: 2026-01-01T00:00:00Z\nupdated_at: 2026-01-01T00:00:00Z\n---\n\nNope body.\n",
        )
        .expect("scrapped file should exist");
        let _guard = WorkingDirGuard::change_to(temp.path());

        let output = archive_command(true)
            .expect("archive command should succeed")
            .expect("archive command should print output");

        let parsed: Value = serde_json::from_str(&output).expect("json should parse");
        assert_eq!(parsed["success"], Value::Bool(true));
        assert_eq!(parsed["data"]["archived"], Value::from(1));
    }

    #[test]
    fn run_init_creates_project_files_with_defaults() {
        let temp = TestDir::new();
        let project_dir = temp.path().join("demo-project");
        fs::create_dir_all(&project_dir).expect("project dir should exist");
        let _guard = WorkingDirGuard::change_to(&project_dir);

        let output = run(Cli {
            json: false,
            command: Some(Commands::Init),
        })
        .expect("init command should succeed")
        .output
        .expect("init command should print output");

        assert!(output.contains("initialized ish project"));
        assert_eq!(
            fs::read_to_string(project_dir.join(".ish").join(".gitignore"))
                .expect("gitignore should be written"),
            STORE_GITIGNORE_CONTENT
        );

        let config = Config::load(project_dir.join(CONFIG_FILE_NAME)).expect("config should load");
        assert_eq!(config.ish.path, ".ish");
        assert_eq!(config.ish.prefix, "demo-project-");
        assert_eq!(config.project.name, "demo-project");
    }

    #[test]
    fn prime_command_silently_exits_without_config() {
        let temp = TestDir::new();
        let _guard = WorkingDirGuard::change_to(temp.path());

        assert!(
            prime_command(false)
                .expect("prime command should succeed")
                .is_none()
        );
    }

    #[test]
    fn init_command_is_idempotent_and_preserves_existing_config() {
        let temp = TestDir::new();
        let project_dir = temp.path().join("custom-project");
        fs::create_dir_all(&project_dir).expect("project dir should exist");
        let _guard = WorkingDirGuard::change_to(&project_dir);

        let mut config = Config::default_with_prefix("custom");
        config.project.name = "Custom Name".to_string();
        config.save(&project_dir).expect("config should save");

        let output = init_command(false)
            .expect("init command should succeed")
            .expect("init command should print output");

        assert!(output.contains("already initialized"));
        let loaded = Config::load(project_dir.join(CONFIG_FILE_NAME)).expect("config should load");
        assert_eq!(loaded.ish.prefix, "custom");
        assert_eq!(loaded.project.name, "Custom Name");
        assert_eq!(
            fs::read_to_string(project_dir.join(".ish").join(".gitignore"))
                .expect("gitignore should be written"),
            STORE_GITIGNORE_CONTENT
        );
    }

    #[test]
    fn run_roadmap_returns_rendered_output_when_config_exists() {
        let temp = TestDir::new();
        let mut config = Config::default();
        config.project.name = "Roadmap Test".to_string();
        config.save(temp.path()).expect("config should save");
        let store_root = temp.path().join(".ish");
        fs::create_dir_all(&store_root).expect("store root should exist");
        fs::write(
            store_root.join("ish-m1--milestone.md"),
            "---\n# ish-m1\ntitle: Milestone\nstatus: todo\ntype: milestone\ncreated_at: 2026-01-01T00:00:00Z\nupdated_at: 2026-01-01T00:00:00Z\n---\n\nMilestone body.\n",
        )
        .expect("milestone file should exist");
        let _guard = WorkingDirGuard::change_to(temp.path());

        let output = run(Cli {
            command: Some(Commands::Roadmap(RoadmapArgs {
                include_done: false,
                status: Vec::new(),
                no_status: Vec::new(),
                no_links: true,
                link_prefix: None,
            })),
            json: false,
        })
        .expect("roadmap command should succeed")
        .output
        .expect("roadmap command should print output");

        assert!(output.contains("# Roadmap"));
        assert!(output.contains("Milestone: Milestone (ish-m1)"));
    }

    #[test]
    fn roadmap_command_errors_without_config() {
        let temp = TestDir::new();
        let _guard = WorkingDirGuard::change_to(temp.path());

        let error = roadmap_command(
            RoadmapArgs {
                include_done: false,
                status: Vec::new(),
                no_status: Vec::new(),
                no_links: false,
                link_prefix: None,
            },
            false,
        )
        .expect_err("roadmap command should fail without config");

        assert!(error.message.contains("no `.ish.yml` found"));
    }

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
    fn init_command_wraps_message_in_json_mode() {
        let temp = TestDir::new();
        let project_dir = temp.path().join("json-project");
        fs::create_dir_all(&project_dir).expect("project dir should exist");
        let _guard = WorkingDirGuard::change_to(&project_dir);

        let output = init_command(true)
            .expect("init command should succeed")
            .expect("init command should print output");

        let parsed: Value = serde_json::from_str(&output).expect("json should parse");
        assert_eq!(parsed["success"], Value::Bool(true));
        assert!(
            parsed["message"]
                .as_str()
                .expect("message should be present")
                .contains("initialized ish project")
        );
    }

    #[test]
    fn run_roadmap_wraps_nested_json_in_response() {
        let temp = TestDir::new();
        let mut config = Config::default();
        config.project.name = "Roadmap JSON Test".to_string();
        config.save(temp.path()).expect("config should save");
        let store_root = temp.path().join(".ish");
        fs::create_dir_all(&store_root).expect("store root should exist");
        fs::write(
            store_root.join("ish-m1--milestone.md"),
            "---\n# ish-m1\ntitle: Milestone\nstatus: todo\ntype: milestone\ncreated_at: 2026-01-01T00:00:00Z\nupdated_at: 2026-01-01T00:00:00Z\n---\n\nMilestone body.\n",
        )
        .expect("milestone file should exist");
        let _guard = WorkingDirGuard::change_to(temp.path());

        let output = run(Cli {
            json: true,
            command: Some(Commands::Roadmap(RoadmapArgs {
                include_done: false,
                status: Vec::new(),
                no_status: Vec::new(),
                no_links: true,
                link_prefix: None,
            })),
        })
        .expect("roadmap command should succeed")
        .output
        .expect("roadmap command should print output");

        let parsed: Value = serde_json::from_str(&output).expect("json should parse");
        assert_eq!(parsed["success"], Value::Bool(true));
        assert_eq!(parsed["data"]["milestones"][0]["milestone"]["id"], "ish-m1");
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

    #[test]
    fn prime_command_wraps_markdown_in_json_mode() {
        let temp = TestDir::new();
        let mut config = Config::default();
        config.project.name = "Prime JSON Test".to_string();
        config.save(temp.path()).expect("config should save");
        let _guard = WorkingDirGuard::change_to(temp.path());

        let output = prime_command(true)
            .expect("prime command should succeed")
            .expect("prime command should print output");

        let parsed: Value = serde_json::from_str(&output).expect("json should parse");
        assert_eq!(parsed["success"], Value::Bool(true));
        assert!(
            parsed["message"]
                .as_str()
                .expect("message should be present")
                .contains("# ish Agent Guide")
        );
    }

    #[test]
    fn check_command_reports_link_issues_and_returns_failure_exit_code() {
        let temp = TestDir::new();
        let mut config = Config::default();
        config.project.name = "Check Test".to_string();
        config.save(temp.path()).expect("config should save");
        let store_root = temp.path().join(".ish");
        fs::create_dir_all(&store_root).expect("store root should exist");
        fs::write(
            store_root.join("ish-a--a.md"),
            "---\n# ish-a\ntitle: A\nstatus: todo\ntype: task\ncreated_at: 2026-01-01T00:00:00Z\nupdated_at: 2026-01-01T00:00:00Z\nblocking:\n  - ish-missing\n---\n\nA body.\n",
        )
        .expect("issue file should exist");
        let _guard = WorkingDirGuard::change_to(temp.path());

        let outcome =
            check_command(CheckArgs { fix: false }, false).expect("check command should succeed");
        let output = outcome.output.expect("check command should print output");

        assert_eq!(outcome.exit_code, ExitCode::FAILURE);
        assert!(output.contains("✗ broken links"));
        assert!(output.contains("ish-a blocking ish-missing"));
        assert!(output.contains("Summary: 1 issue(s) found"));
    }

    #[test]
    fn check_command_fix_removes_broken_links_but_still_reports_failure() {
        let temp = TestDir::new();
        let mut config = Config::default();
        config.project.name = "Check Fix Test".to_string();
        config.save(temp.path()).expect("config should save");
        let store_root = temp.path().join(".ish");
        fs::create_dir_all(&store_root).expect("store root should exist");
        fs::write(
            store_root.join("ish-a--a.md"),
            "---\n# ish-a\ntitle: A\nstatus: todo\ntype: task\ncreated_at: 2026-01-01T00:00:00Z\nupdated_at: 2026-01-01T00:00:00Z\nblocking:\n  - ish-a\n  - ish-missing\n---\n\nA body.\n",
        )
        .expect("issue file should exist");
        let _guard = WorkingDirGuard::change_to(temp.path());

        let outcome =
            check_command(CheckArgs { fix: true }, false).expect("check command should succeed");
        let output = outcome.output.expect("check command should print output");
        let contents = fs::read_to_string(store_root.join("ish-a--a.md"))
            .expect("updated ishoo should be readable");

        assert_eq!(outcome.exit_code, ExitCode::FAILURE);
        assert!(output.contains("Applied --fix to 2 link(s)"));
        assert!(output.contains("✓ remaining broken links: none"));
        assert!(!contents.contains("ish-missing"));
        assert!(!contents.contains("- ish-a"));
    }

    #[test]
    fn check_command_wraps_results_in_json_mode() {
        let temp = TestDir::new();
        let mut config = Config::default();
        config.project.name = "Check JSON Test".to_string();
        config.save(temp.path()).expect("config should save");
        let store_root = temp.path().join(".ish");
        fs::create_dir_all(&store_root).expect("store root should exist");
        fs::write(
            store_root.join("ish-a--a.md"),
            "---\n# ish-a\ntitle: A\nstatus: todo\ntype: task\ncreated_at: 2026-01-01T00:00:00Z\nupdated_at: 2026-01-01T00:00:00Z\nblocked_by:\n  - ish-missing\n---\n\nA body.\n",
        )
        .expect("issue file should exist");
        let _guard = WorkingDirGuard::change_to(temp.path());

        let outcome = run(Cli {
            json: true,
            command: Some(Commands::Check(CheckArgs { fix: false })),
        })
        .expect("check command should succeed");
        let output = outcome.output.expect("check command should print output");
        let parsed: Value = serde_json::from_str(&output).expect("json should parse");

        assert_eq!(outcome.exit_code, ExitCode::FAILURE);
        assert_eq!(parsed["success"], Value::Bool(true));
        assert_eq!(parsed["data"]["summary"]["issues_found"], Value::from(1));
        assert_eq!(
            parsed["data"]["checks"]["links"]["initial"]["broken_links"][0]["link_type"],
            Value::String("blocked_by".to_string())
        );
    }

    #[test]
    fn check_command_returns_success_when_workspace_is_clean() {
        let temp = TestDir::new();
        let mut config = Config::default();
        config.project.name = "Check Clean Test".to_string();
        config.save(temp.path()).expect("config should save");
        let store_root = temp.path().join(".ish");
        fs::create_dir_all(&store_root).expect("store root should exist");
        fs::write(
            store_root.join("ish-a--a.md"),
            "---\n# ish-a\ntitle: A\nstatus: todo\ntype: task\ncreated_at: 2026-01-01T00:00:00Z\nupdated_at: 2026-01-01T00:00:00Z\n---\n\nA body.\n",
        )
        .expect("clean file should exist");
        let _guard = WorkingDirGuard::change_to(temp.path());

        let outcome =
            check_command(CheckArgs { fix: false }, false).expect("check command should succeed");
        let output = outcome.output.expect("check command should print output");

        assert_eq!(outcome.exit_code, ExitCode::SUCCESS);
        assert!(output.contains("Summary: no issues found"));
    }
}

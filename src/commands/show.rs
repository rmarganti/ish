use crate::app::{AppContext, AppError, json_output_error, store_app_error};
use crate::cli::ShowArgs;
use crate::config::Config;
use crate::core::store::{LinkType, Store, StoreError};
use crate::model::ish::Ish;
use crate::output::{
    ErrorCode, detect_terminal_width, heading, muted, output_success_multiple, render_id,
    render_markdown_with_width, render_priority, render_status, render_type,
};
use std::collections::HashSet;

pub(crate) fn show_command(args: ShowArgs, json: bool) -> Result<Option<String>, AppError> {
    let context = AppContext::load()?;
    let config = context.config;
    let store = context.store;
    let ishes = resolve_show_ishes(&store, &args.ids)?;

    if json {
        let rendered = ishes
            .iter()
            .map(|ish| serde_json::to_value(ish.to_json(&ish.etag())))
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
            ishes
                .iter()
                .map(|ish| ish.etag())
                .collect::<Vec<_>>()
                .join("\n"),
        ));
    }

    if args.body_only {
        return Ok(Some(render_show_sections(
            &ishes.iter().map(|ish| ish.body.clone()).collect::<Vec<_>>(),
        )));
    }

    if args.raw {
        return Ok(Some(render_show_sections(
            &ishes.iter().map(|ish| ish.render()).collect::<Vec<_>>(),
        )));
    }

    Ok(Some(render_show_sections(
        &ishes
            .iter()
            .map(|ish| render_show_human(ish, &store, &config))
            .collect::<Vec<_>>(),
    )))
}

fn resolve_show_ishes(store: &Store, ids: &[String]) -> Result<Vec<Ish>, AppError> {
    let mut ordered = Vec::new();
    let mut seen = HashSet::new();

    for id in ids {
        let normalized = store.normalize_id(id);
        if seen.insert(normalized.clone()) {
            let ish = store
                .get(&normalized)
                .cloned()
                .ok_or_else(|| store_app_error(StoreError::NotFound(normalized.clone())))?;
            ordered.push(ish);
        }
    }

    Ok(ordered)
}

fn render_show_sections(sections: &[String]) -> String {
    sections.join("\n════════════════════════════════════════════════════════════════\n")
}

fn render_show_human(ish: &Ish, store: &Store, config: &Config) -> String {
    let priority = ish.priority.as_deref().unwrap_or("normal");
    let mut lines = vec![format!(
        "{} {} {} {} {}{}",
        render_id(&ish.id),
        render_status(config, &ish.status),
        render_type(config, &ish.ish_type),
        render_priority(config, priority),
        heading(&ish.title),
        if ish.tags.is_empty() {
            String::new()
        } else {
            format!(" {}", muted(&format!("#{}", ish.tags.join(" #"))))
        }
    )];

    lines.push(format!("Path: {}", ish.path));
    lines.push(format!(
        "Created: {} | Updated: {} | ETag: {}",
        ish.created_at.to_rfc3339(),
        ish.updated_at.to_rfc3339(),
        ish.etag()
    ));
    lines.push("─".repeat(64));

    let mut relationships = Vec::new();
    relationships.push(format_relationship(
        "Parent",
        ish.parent.as_deref().map(str::to_string),
    ));
    relationships.push(format_relationships("Blocking", &ish.blocking));
    relationships.push(format_relationships("Blocked by", &ish.blocked_by));
    relationships.push(format_relationships(
        "Incoming",
        &store
            .find_incoming_links(&ish.id)
            .into_iter()
            .map(|link| format!("{} ({})", link.source_id, link_type_label(link.link_type)))
            .collect::<Vec<_>>(),
    ));
    if let Some((status, source_id)) = store.implicit_status(&ish.id) {
        relationships.push(format!("Inherited status: {} from {}", status, source_id));
    }
    if store.is_blocked(&ish.id) {
        relationships.push(format_relationships(
            "Active blockers",
            &store.find_all_blockers(&ish.id),
        ));
    }
    lines.extend(relationships);
    lines.push("─".repeat(64));

    let width = detect_terminal_width().saturating_sub(2).max(20);
    let body = render_markdown_with_width(&ish.body, width);
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

fn link_type_label(link_type: LinkType) -> &'static str {
    match link_type {
        LinkType::Parent => "parent",
        LinkType::Blocking => "blocking",
        LinkType::BlockedBy => "blocked_by",
    }
}

#[cfg(test)]
mod tests {
    use super::show_command;
    use crate::cli::ShowArgs;
    use crate::config::Config;
    use crate::test_support::{TestDir, WorkingDirGuard, write_test_ish};
    use serde_json::Value;
    use std::fs;

    #[test]
    fn show_command_supports_json_raw_body_and_etag_output() {
        let temp = TestDir::new();
        let config = Config::default();
        config.save(temp.path()).expect("config should save");
        let store_root = temp.path().join(".ish");
        fs::create_dir_all(&store_root).expect("store root should exist");
        write_test_ish(
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
        write_test_ish(
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
        write_test_ish(
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
        assert_eq!(parsed[0]["id"], "ish-target");
        assert_eq!(parsed[0]["parent"], "ish-parent");
        assert_eq!(parsed[0]["blocking"][0], "ish-blocker");
        assert!(parsed[0].get("etag").is_some());

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
        write_test_ish(
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
        write_test_ish(
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
        write_test_ish(
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
}

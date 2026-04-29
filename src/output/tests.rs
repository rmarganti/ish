use super::{
    ErrorCode, build_tree, color_name_to_color, danger, detect_terminal_width, heading, muted,
    output_error, output_message, output_success, output_success_multiple, render_id,
    render_markdown, render_markdown_with_width, render_priority, render_status, render_tree,
    render_type, success, warning,
};
use crate::config::Config;
use crate::model::ish::Ish;
use chrono::{TimeZone, Utc};
use colored::{Color, control};
use serde_json::{Value, json};
use std::{collections::HashMap, sync::Mutex};

fn sample_ish_json(id: &str) -> crate::model::ish::IshJson {
    Ish {
        id: id.to_string(),
        slug: "sample".to_string(),
        path: format!("{id}--sample.md"),
        title: "Sample".to_string(),
        status: "todo".to_string(),
        ish_type: "task".to_string(),
        priority: Some("normal".to_string()),
        tags: vec!["backend".to_string()],
        created_at: Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap(),
        updated_at: Utc.with_ymd_and_hms(2026, 1, 2, 0, 0, 0).unwrap(),
        order: None,
        body: "Body".to_string(),
        parent: None,
        blocking: Vec::new(),
        blocked_by: Vec::new(),
    }
    .to_json("etag-1")
}

fn tree_ish(
    id: &str,
    title: &str,
    parent: Option<&str>,
    tags: &[&str],
    priority: Option<&str>,
) -> Ish {
    Ish {
        id: id.to_string(),
        slug: title.to_ascii_lowercase().replace(' ', "-"),
        path: format!("{id}.md"),
        title: title.to_string(),
        status: "todo".to_string(),
        ish_type: "task".to_string(),
        priority: priority.map(str::to_string),
        tags: tags.iter().map(|tag| (*tag).to_string()).collect(),
        created_at: Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap(),
        updated_at: Utc.with_ymd_and_hms(2026, 1, 2, 0, 0, 0).unwrap(),
        order: None,
        body: String::new(),
        parent: parent.map(str::to_string),
        blocking: Vec::new(),
        blocked_by: Vec::new(),
    }
}

fn strip_ansi(text: &str) -> String {
    let mut plain = String::new();
    let mut chars = text.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '\u{1b}' && chars.peek() == Some(&'[') {
            chars.next();
            for next in chars.by_ref() {
                if next.is_ascii_alphabetic() {
                    break;
                }
            }
            continue;
        }

        plain.push(ch);
    }

    plain
}

static COLOR_OVERRIDE_LOCK: Mutex<()> = Mutex::new(());

fn with_color_override<T>(enabled: bool, f: impl FnOnce() -> T) -> T {
    let _lock = COLOR_OVERRIDE_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());

    struct ResetColorOverride(bool);

    impl Drop for ResetColorOverride {
        fn drop(&mut self) {
            control::set_override(self.0);
        }
    }

    let reset = ResetColorOverride(control::SHOULD_COLORIZE.should_colorize());
    control::set_override(enabled);

    let result = f();

    drop(reset);
    result
}

#[test]
fn output_success_wraps_structured_data() {
    let rendered = output_success(json!({"version": "0.1.0"})).expect("json should render");
    let parsed: Value = serde_json::from_str(&rendered).expect("json should parse");

    assert_eq!(parsed["version"], Value::String("0.1.0".to_string()));
}

#[test]
fn output_success_multiple_outputs_bare_array() {
    let rendered =
        output_success_multiple(vec![sample_ish_json("ish-a1")]).expect("json should render");
    let parsed: Value = serde_json::from_str(&rendered).expect("json should parse");

    assert_eq!(parsed[0]["id"], Value::String("ish-a1".to_string()));
}

#[test]
fn output_error_includes_code_and_message() {
    let parsed: Value = serde_json::from_str(&output_error(ErrorCode::Conflict, "etag mismatch"))
        .expect("json should parse");

    assert_eq!(parsed["success"], Value::Bool(false));
    assert_eq!(parsed["code"], Value::String("conflict".to_string()));
    assert_eq!(
        parsed["message"],
        Value::String("etag mismatch".to_string())
    );
}

#[test]
fn output_message_uses_message_field() {
    let rendered = output_message("ready").expect("json should render");
    let parsed: Value = serde_json::from_str(&rendered).expect("json should parse");

    assert_eq!(parsed, Value::String("ready".to_string()));
}

#[test]
fn error_code_strings_match_cli_contract() {
    assert_eq!(ErrorCode::NotFound.as_str(), "not_found");
    assert_eq!(ErrorCode::Validation.as_str(), "validation");
    assert_eq!(ErrorCode::Conflict.as_str(), "conflict");
    assert_eq!(ErrorCode::FileError.as_str(), "file_error");
}

#[test]
fn color_name_mapping_matches_config_palette() {
    assert_eq!(color_name_to_color("red"), Some(Color::Red));
    assert_eq!(color_name_to_color("yellow"), Some(Color::Yellow));
    assert_eq!(color_name_to_color("green"), Some(Color::Green));
    assert_eq!(color_name_to_color("blue"), Some(Color::Blue));
    assert_eq!(color_name_to_color("purple"), Some(Color::Magenta));
    assert_eq!(color_name_to_color("cyan"), Some(Color::Cyan));
    assert_eq!(color_name_to_color("gray"), Some(Color::BrightBlack));
    assert_eq!(color_name_to_color("white"), Some(Color::White));
    assert_eq!(color_name_to_color("unknown"), None);
}

#[test]
fn render_helpers_apply_expected_labels_and_styles() {
    let config = Config::default();
    let (
        active_status,
        archive_status,
        rendered_type,
        rendered_priority,
        rendered_id,
        rendered_muted,
        rendered_heading,
        rendered_success,
        rendered_danger,
        rendered_warning,
    ) = with_color_override(true, || {
        (
            render_status(&config, "todo"),
            render_status(&config, "completed"),
            render_type(&config, "task"),
            render_priority(&config, "high"),
            render_id("ish-abcd"),
            muted("secondary text"),
            heading("Heading"),
            success("deleted"),
            danger("failed"),
            warning("careful"),
        )
    });

    assert!(active_status.contains("[todo]"));
    assert!(archive_status.contains("[completed]"));
    assert_ne!(archive_status, active_status);
    assert!(rendered_type.contains("[task]"));
    assert!(rendered_priority.contains("[high]"));
    assert!(rendered_id.contains("ish-abcd"));
    assert!(rendered_muted.contains("secondary text"));
    assert!(rendered_heading.contains("Heading"));
    assert!(rendered_success.contains("deleted"));
    assert!(rendered_danger.contains("failed"));
    assert!(rendered_warning.contains("careful"));
}

#[test]
fn build_tree_includes_context_ancestors_and_sorts_children() {
    let root = tree_ish("ish-root", "Root", None, &[], Some("normal"));
    let alpha = tree_ish("ish-alpha", "Alpha", Some("ish-root"), &[], Some("high"));
    let beta = tree_ish("ish-beta", "Beta", Some("ish-root"), &[], Some("low"));
    let detached = tree_ish("ish-detached", "Detached", None, &[], None);
    let all = vec![&root, &alpha, &beta, &detached];
    let filtered = vec![&beta, &alpha];

    let tree = build_tree(
        &filtered,
        &all,
        |items| {
            let mut sorted = items.to_vec();
            sorted.sort_by(|left, right| left.title.cmp(&right.title));
            sorted
        },
        &HashMap::new(),
    );

    assert_eq!(tree.len(), 1);
    assert_eq!(tree[0].ish.id, "ish-root");
    assert!(tree[0].context_only);
    assert_eq!(tree[0].children.len(), 2);
    assert_eq!(tree[0].children[0].ish.id, "ish-alpha");
    assert_eq!(tree[0].children[1].ish.id, "ish-beta");
}

#[test]
fn render_tree_uses_connectors_implicit_status_tags_and_truncation() {
    let config = Config::default();
    let root = tree_ish("ish-root", "Root", None, &[], Some("normal"));
    let child = tree_ish(
        "ish-child",
        "A very long child title for truncation",
        Some("ish-root"),
        &["backend", "ui"],
        Some("high"),
    );
    let tree = build_tree(
        &[&child],
        &[&root, &child],
        |items| items.to_vec(),
        &HashMap::from([(String::from("ish-child"), String::from("completed"))]),
    );

    let (rendered, truncated) = with_color_override(false, || {
        (
            render_tree(&tree, &config, 9, true, 120),
            render_tree(&tree, &config, 9, true, 45),
        )
    });

    assert!(rendered.contains("ish-root"));
    assert!(rendered.contains("[todo]"));
    assert!(rendered.contains("[task]"));
    assert!(rendered.contains("[normal]"));
    assert!(rendered.contains("Root"));
    assert!(rendered.contains("└── ish-child"));
    assert!(rendered.contains("[completed]"));
    assert!(rendered.contains("[high]"));
    assert!(rendered.contains("#backend"));
    assert!(truncated.contains("..."));
}

#[test]
fn render_tree_dims_context_only_ancestors() {
    let config = Config::default();
    let root = tree_ish("ish-root", "Root", None, &[], Some("normal"));
    let child = tree_ish("ish-child", "Child", Some("ish-root"), &[], Some("normal"));
    let tree = build_tree(
        &[&child],
        &[&root, &child],
        |items| items.to_vec(),
        &HashMap::new(),
    );
    let rendered = render_tree(&tree, &config, 9, false, 80);

    assert!(tree[0].context_only);
    assert!(
        rendered
            .lines()
            .next()
            .is_some_and(|line| line.contains("ish-root"))
    );
}

#[test]
fn terminal_width_has_reasonable_default() {
    assert!(detect_terminal_width() >= 1);
}

#[test]
fn render_markdown_formats_common_markdown_elements() {
    let rendered = with_color_override(true, || {
        render_markdown(
            "# Title\n\nParagraph with **bold**, *italic*, and `code`.\n\n- item one\n- item two\n\n```rust\nfn main() {}\n```\n\n[example](https://example.com)",
        )
    });

    let plain = strip_ansi(&rendered);
    assert!(plain.contains("Title"));
    assert!(plain.contains("Paragraph with bold, italic, and code."));
    assert!(plain.contains("item one"));
    assert!(plain.contains("fn main() {}"));
    assert!(plain.contains("example"));
    assert!(rendered.contains('\u{1b}'));
}

#[test]
fn render_markdown_wraps_to_requested_width() {
    let rendered = with_color_override(false, || {
        render_markdown_with_width(
            "This paragraph contains enough words to wrap across multiple lines when the width is intentionally narrow.",
            24,
        )
    });

    let plain = strip_ansi(&rendered);
    assert!(plain.lines().all(|line| line.chars().count() <= 24));
    assert!(plain.lines().count() > 2);
}

#[test]
fn render_markdown_returns_empty_string_for_blank_body() {
    assert!(render_markdown("   \n\n").is_empty());
}

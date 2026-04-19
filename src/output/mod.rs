use crate::config::Config;
use crate::model::ish::{Ish, IshJson};
use colored::{Color, Colorize};
use serde::Serialize;
use std::collections::{HashMap, HashSet};
use termimad::{
    CompoundStyle, MadSkin,
    crossterm::style::{Attribute, Color as MadColor},
};
use terminal_size::{Width, terminal_size};

const DEFAULT_MARKDOWN_WIDTH: usize = 80;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCode {
    NotFound,
    Validation,
    Conflict,
    FileError,
}

impl ErrorCode {
    fn as_str(self) -> &'static str {
        match self {
            ErrorCode::NotFound => "not_found",
            ErrorCode::Validation => "validation",
            ErrorCode::Conflict => "conflict",
            ErrorCode::FileError => "file_error",
        }
    }
}

#[derive(Debug, Serialize)]
pub struct Response<T, L = IshJson> {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ishes: Option<Vec<L>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub count: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<&'static str>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct TreeNode<'a> {
    pub ish: &'a Ish,
    pub children: Vec<TreeNode<'a>>,
    pub context_only: bool,
    pub implicit_status: Option<String>,
}

pub fn output_success<T: Serialize>(data: T) -> Result<String, String> {
    serde_json::to_string_pretty(&data).map_err(|e| e.to_string())
}

pub fn output_success_multiple<T: Serialize>(ishes: Vec<T>) -> Result<String, String> {
    serde_json::to_string_pretty(&ishes).map_err(|e| e.to_string())
}

pub fn output_message(message: impl Into<String>) -> Result<String, String> {
    serde_json::to_string_pretty(&message.into()).map_err(|e| e.to_string())
}

pub fn output_error(code: ErrorCode, message: impl Into<String>) -> String {
    render(Response::<()> {
        success: false,
        message: Some(message.into()),
        data: None,
        ishes: None,
        count: None,
        code: Some(code.as_str()),
    })
    .expect("error responses should serialize")
}

#[allow(dead_code)]
pub fn build_tree<'a, F>(
    filtered_ishes: &[&'a Ish],
    all_ishes: &[&'a Ish],
    sort_fn: F,
    implicit_statuses: &HashMap<String, String>,
) -> Vec<TreeNode<'a>>
where
    F: Fn(&[&'a Ish]) -> Vec<&'a Ish>,
{
    let filtered_ids = filtered_ishes
        .iter()
        .map(|ish| ish.id.as_str())
        .collect::<HashSet<_>>();
    let by_id = all_ishes
        .iter()
        .map(|ish| (ish.id.as_str(), *ish))
        .collect::<HashMap<_, _>>();
    let mut included_ids = filtered_ids.iter().copied().collect::<HashSet<_>>();

    for ish in filtered_ishes {
        let mut next_parent = ish.parent.as_deref();
        while let Some(parent_id) = next_parent {
            let Some(parent) = by_id.get(parent_id).copied() else {
                break;
            };
            if !included_ids.insert(parent.id.as_str()) {
                break;
            }
            next_parent = parent.parent.as_deref();
        }
    }

    let mut children_by_parent = HashMap::<Option<&'a str>, Vec<&'a Ish>>::new();
    for included_id in &included_ids {
        let Some(ish) = by_id.get(included_id).copied() else {
            continue;
        };
        let parent_key = ish
            .parent
            .as_deref()
            .filter(|parent| included_ids.contains(parent));
        children_by_parent.entry(parent_key).or_default().push(ish);
    }

    fn build_nodes<'a, F>(
        parent_id: Option<&'a str>,
        children_by_parent: &HashMap<Option<&'a str>, Vec<&'a Ish>>,
        filtered_ids: &HashSet<&'a str>,
        implicit_statuses: &HashMap<String, String>,
        sort_fn: &F,
    ) -> Vec<TreeNode<'a>>
    where
        F: Fn(&[&'a Ish]) -> Vec<&'a Ish>,
    {
        let Some(children) = children_by_parent.get(&parent_id) else {
            return Vec::new();
        };

        sort_fn(children)
            .into_iter()
            .map(|ish| TreeNode {
                children: build_nodes(
                    Some(ish.id.as_str()),
                    children_by_parent,
                    filtered_ids,
                    implicit_statuses,
                    sort_fn,
                ),
                context_only: !filtered_ids.contains(ish.id.as_str()),
                implicit_status: implicit_statuses.get(&ish.id).cloned(),
                ish,
            })
            .collect()
    }

    build_nodes(
        None,
        &children_by_parent,
        &filtered_ids,
        implicit_statuses,
        &sort_fn,
    )
}

#[allow(dead_code)]
pub fn render_tree(
    tree: &[TreeNode<'_>],
    config: &Config,
    max_id_width: usize,
    has_tags: bool,
    term_width: usize,
) -> String {
    let mut lines = Vec::new();
    let width = term_width.max(1);

    struct RenderTreeContext<'a> {
        config: &'a Config,
        max_id_width: usize,
        has_tags: bool,
        term_width: usize,
    }

    let context = RenderTreeContext {
        config,
        max_id_width,
        has_tags,
        term_width: width,
    };

    fn render_node(
        node: &TreeNode<'_>,
        context: &RenderTreeContext<'_>,
        ancestors_have_more: &[bool],
        is_last: bool,
        lines: &mut Vec<String>,
    ) {
        let prefix = tree_prefix(ancestors_have_more, is_last);
        let prefix_width = visible_width(&prefix);
        let status = node.implicit_status.as_deref().unwrap_or(&node.ish.status);
        let priority = node.ish.priority.as_deref().unwrap_or("normal");
        let id_plain = format!("{:width$}", node.ish.id, width = context.max_id_width);
        let fixed_plain = format!(
            "{} [{}] [{}] [{}] ",
            id_plain, status, node.ish.ish_type, priority
        );
        let available_tail = context
            .term_width
            .saturating_sub(prefix_width + visible_width(&fixed_plain));
        let tail = truncate_visible(
            &format!(
                "{}{}",
                node.ish.title,
                if context.has_tags {
                    format_tags(&node.ish.tags)
                } else {
                    String::new()
                }
            ),
            available_tail,
        );
        let mut line = format!(
            "{prefix}{} {} {} {} {}",
            render_id(&id_plain),
            render_status(context.config, status),
            render_type(context.config, &node.ish.ish_type),
            render_priority(context.config, priority),
            render_tail(&tail, node.ish.tags.len())
        );

        if node.context_only {
            line = line.dimmed().to_string();
        }

        lines.push(line);

        let mut next_ancestors = ancestors_have_more.to_vec();
        next_ancestors.push(!is_last);
        for (index, child) in node.children.iter().enumerate() {
            render_node(
                child,
                context,
                &next_ancestors,
                index + 1 == node.children.len(),
                lines,
            );
        }
    }

    for (index, node) in tree.iter().enumerate() {
        render_node(node, &context, &[], index + 1 == tree.len(), &mut lines);
    }

    lines.join("\n")
}

#[allow(dead_code)]
pub fn detect_terminal_width() -> usize {
    terminal_size()
        .map(|(Width(width), _)| usize::from(width))
        .unwrap_or(80)
}

#[allow(dead_code)]
pub fn render_status(config: &Config, status: &str) -> String {
    let rendered = render_badge(status, config.get_status(status).map(|status| status.color));
    if config.is_archive_status(status) {
        rendered.dimmed().to_string()
    } else {
        rendered.to_string()
    }
}

#[allow(dead_code)]
pub fn render_type(config: &Config, ish_type: &str) -> String {
    render_badge(
        ish_type,
        config.get_type(ish_type).map(|ish_type| ish_type.color),
    )
    .to_string()
}

#[allow(dead_code)]
pub fn render_priority(config: &Config, priority: &str) -> String {
    render_badge(
        priority,
        config.get_priority(priority).map(|priority| priority.color),
    )
    .to_string()
}

#[allow(dead_code)]
pub fn render_id(id: &str) -> String {
    id.bold().white().to_string()
}

#[allow(dead_code)]
pub fn muted(text: &str) -> String {
    text.dimmed().to_string()
}

#[allow(dead_code)]
pub fn heading(text: &str) -> String {
    text.bold().to_string()
}

pub fn success(text: &str) -> String {
    text.green().bold().to_string()
}

pub fn danger(text: &str) -> String {
    text.red().bold().to_string()
}

pub fn warning(text: &str) -> String {
    text.yellow().bold().to_string()
}

#[allow(dead_code)]
pub fn render_markdown(markdown: &str) -> String {
    render_markdown_with_width(markdown, DEFAULT_MARKDOWN_WIDTH)
}

#[allow(dead_code)]
pub fn render_markdown_with_width(markdown: &str, width: usize) -> String {
    if markdown.trim().is_empty() {
        return String::new();
    }

    let width = width.max(3);
    markdown_skin().text(markdown, Some(width)).to_string()
}

fn markdown_skin() -> MadSkin {
    let mut skin = MadSkin::default();
    skin.bold.set_fg(MadColor::Yellow);
    skin.italic.add_attr(Attribute::Underlined);
    skin.inline_code = CompoundStyle::with_fgbg(MadColor::Black, MadColor::Grey);
    skin.code_block.set_fg(MadColor::Grey);
    skin.code_block.set_bg(MadColor::Black);
    skin.headers[0].set_fg(MadColor::Cyan);
    skin.headers[1].set_fg(MadColor::Cyan);
    skin.headers[2].set_fg(MadColor::Blue);
    skin.bullet.set_fg(MadColor::Cyan);
    skin.quote_mark.set_fg(MadColor::DarkGrey);
    skin
}

fn render<T: Serialize, L: Serialize>(response: Response<T, L>) -> Result<String, String> {
    serde_json::to_string_pretty(&response)
        .map_err(|error| format!("failed to serialize JSON output: {error}"))
}

fn tree_prefix(ancestors_have_more: &[bool], is_last: bool) -> String {
    if ancestors_have_more.is_empty() {
        return String::new();
    }

    let mut prefix = String::new();
    for has_more in &ancestors_have_more[..ancestors_have_more.len() - 1] {
        if *has_more {
            prefix.push_str("│   ");
        } else {
            prefix.push_str("    ");
        }
    }
    prefix.push_str(if is_last { "└── " } else { "├── " });
    prefix
}

fn render_tail(tail: &str, tag_count: usize) -> String {
    if tag_count == 0 {
        return tail.to_string();
    }

    let mut parts = tail.split(" #");
    let title = parts.next().unwrap_or_default();
    let tags = parts.collect::<Vec<_>>();
    if tags.is_empty() {
        return title.to_string();
    }

    format!("{}{}", title, muted(&format!(" #{}", tags.join(" #"))))
}

fn format_tags(tags: &[String]) -> String {
    if tags.is_empty() {
        String::new()
    } else {
        format!(" #{}", tags.join(" #"))
    }
}

fn truncate_visible(text: &str, width: usize) -> String {
    let text_width = visible_width(text);
    if text_width <= width {
        return text.to_string();
    }

    if width == 0 {
        return String::new();
    }

    if width <= 3 {
        return ".".repeat(width);
    }

    let mut truncated = String::new();
    for ch in text.chars().take(width - 3) {
        truncated.push(ch);
    }
    truncated.push_str("...");
    truncated
}

fn visible_width(text: &str) -> usize {
    text.chars().count()
}

#[cfg_attr(not(test), allow(dead_code))]
fn render_badge(label: &str, color_name: Option<&str>) -> colored::ColoredString {
    let badge = format!("[{label}]");
    match color_name.and_then(color_name_to_color) {
        Some(color) => badge.color(color).bold(),
        None => badge.bold(),
    }
}

#[cfg_attr(not(test), allow(dead_code))]
fn color_name_to_color(color_name: &str) -> Option<Color> {
    match color_name {
        "red" => Some(Color::Red),
        "yellow" => Some(Color::Yellow),
        "green" => Some(Color::Green),
        "blue" => Some(Color::Blue),
        "purple" => Some(Color::Magenta),
        "cyan" => Some(Color::Cyan),
        "gray" => Some(Color::BrightBlack),
        "white" => Some(Color::White),
        _ => None,
    }
}

pub(crate) fn is_supported_color_name(color_name: &str) -> bool {
    color_name_to_color(color_name).is_some()
}

#[cfg(test)]
mod tests {
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
        let parsed: Value =
            serde_json::from_str(&output_error(ErrorCode::Conflict, "etag mismatch"))
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
}

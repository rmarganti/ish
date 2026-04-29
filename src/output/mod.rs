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
pub(crate) fn color_name_to_color(color_name: &str) -> Option<Color> {
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

pub(crate) fn color_name_to_ratatui(color_name: &str) -> Option<ratatui::style::Color> {
    use ratatui::style::Color as RatatuiColor;

    match color_name_to_color(color_name) {
        Some(Color::Red) => Some(RatatuiColor::Red),
        Some(Color::Yellow) => Some(RatatuiColor::Yellow),
        Some(Color::Green) => Some(RatatuiColor::Green),
        Some(Color::Blue) => Some(RatatuiColor::Blue),
        Some(Color::Magenta) => Some(RatatuiColor::Magenta),
        Some(Color::Cyan) => Some(RatatuiColor::Cyan),
        Some(Color::BrightBlack) => Some(RatatuiColor::DarkGray),
        Some(Color::White) => Some(RatatuiColor::White),
        _ => None,
    }
}

pub(crate) fn is_supported_color_name(color_name: &str) -> bool {
    color_name_to_color(color_name).is_some()
}

#[cfg(test)]
mod tests;

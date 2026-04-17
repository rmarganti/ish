use chrono::{DateTime, Utc};
use fnv::FnvHasher;
use nanoid::nanoid;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::hash::Hasher;
use std::path::Path;

const ID_ALPHABET: [char; 36] = [
    'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's',
    't', 'u', 'v', 'w', 'x', 'y', 'z', '0', '1', '2', '3', '4', '5', '6', '7', '8', '9',
];

const BLOCKED_SUBSTRINGS: &[&str] = &[
    "anal", "anus", "arse", "ass", "bitch", "boob", "butt", "cock", "cunt", "damn", "dick",
    "dildo", "dyke", "fag", "fuck", "hell", "jizz", "knob", "penis", "piss", "poop", "porn",
    "pussy", "sex", "shit", "slut", "tit", "twat", "vag",
];

/// The core issue type. An Ishoo is stored as a markdown file with YAML frontmatter.
#[derive(Debug, Clone, PartialEq)]
pub struct Ishoo {
    /// Unique identifier (e.g. "ish-a1b2").
    pub id: String,
    /// URL-friendly slug derived from the title.
    pub slug: String,
    /// Relative file path (e.g. "ish-a1b2--my-issue.md").
    pub path: String,
    /// Human-readable title.
    pub title: String,
    /// Current status.
    pub status: String,
    /// Issue type (task, bug, feature, epic, milestone).
    pub ishoo_type: String,
    /// Priority level.
    pub priority: Option<String>,
    /// Freeform tags.
    pub tags: Vec<String>,
    /// When the issue was created.
    pub created_at: DateTime<Utc>,
    /// When the issue was last updated.
    pub updated_at: DateTime<Utc>,
    /// Fractional index for manual ordering.
    pub order: Option<String>,
    /// Markdown body content (everything after the frontmatter).
    pub body: String,
    /// Parent issue ID.
    pub parent: Option<String>,
    /// IDs of issues this issue blocks.
    pub blocking: Vec<String>,
    /// IDs of issues that block this issue.
    pub blocked_by: Vec<String>,
}

/// The subset of `Ishoo` fields that are serialized into YAML frontmatter.
/// Excludes `id`, `slug`, `path`, and `body`.
#[derive(Debug, Serialize, Deserialize)]
struct Frontmatter {
    title: String,
    status: String,
    #[serde(rename = "type")]
    ishoo_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    priority: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    tags: Vec<String>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    order: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    parent: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    blocking: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    blocked_by: Vec<String>,
}

/// JSON output representation (all fields + computed etag).
#[derive(Debug, Serialize)]
pub struct IshooJson {
    pub id: String,
    pub slug: String,
    pub path: String,
    pub title: String,
    pub status: String,
    #[serde(rename = "type")]
    pub ishoo_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order: Option<String>,
    pub body: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub blocking: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub blocked_by: Vec<String>,
    pub etag: String,
}

#[derive(Debug)]
pub enum ParseError {
    /// The frontmatter delimiters (`---`) are missing or malformed.
    MissingFrontmatter,
    /// The `# {id}` comment line is missing from the frontmatter.
    MissingId,
    /// YAML deserialization failed.
    Yaml(serde_yaml::Error),
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::MissingFrontmatter => write!(f, "missing or malformed YAML frontmatter"),
            ParseError::MissingId => write!(f, "missing `# <id>` comment in frontmatter"),
            ParseError::Yaml(e) => write!(f, "YAML parse error: {e}"),
        }
    }
}

impl std::error::Error for ParseError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ParseError::Yaml(e) => Some(e),
            _ => None,
        }
    }
}

impl Ishoo {
    /// Parse an `Ishoo` from the raw content of a `.md` file.
    ///
    /// The `filename` is used to derive `id`, `slug`, and `path`.
    /// Expected file format:
    ///
    /// ```text
    /// ---
    /// # {id}
    /// title: ...
    /// status: ...
    /// ...
    /// ---
    ///
    /// Markdown body here.
    /// ```
    pub fn parse(filename: &str, content: &str) -> Result<Self, ParseError> {
        let (id, yaml_str, body) = split_frontmatter(content)?;
        let fm: Frontmatter = serde_yaml::from_str(&yaml_str).map_err(ParseError::Yaml)?;

        let (parsed_id, slug) = parse_filename(filename);
        // Prefer the ID from the frontmatter comment, fall back to filename.
        let id = if id.is_empty() {
            parsed_id
        } else {
            id.to_string()
        };

        Ok(Ishoo {
            id,
            slug,
            path: filename.to_string(),
            title: fm.title,
            status: fm.status,
            ishoo_type: fm.ishoo_type,
            priority: fm.priority,
            tags: fm.tags,
            created_at: fm.created_at,
            updated_at: fm.updated_at,
            order: fm.order,
            body,
            parent: fm.parent,
            blocking: fm.blocking,
            blocked_by: fm.blocked_by,
        })
    }

    /// Render the `Ishoo` back to the `.md` file format.
    pub fn render(&self) -> String {
        self.try_render().expect("failed to serialize frontmatter")
    }

    fn try_render(&self) -> Result<String, serde_yaml::Error> {
        let fm = Frontmatter {
            title: self.title.clone(),
            status: self.status.clone(),
            ishoo_type: self.ishoo_type.clone(),
            priority: self.priority.clone(),
            tags: self.tags.clone(),
            created_at: self.created_at,
            updated_at: self.updated_at,
            order: self.order.clone(),
            parent: self.parent.clone(),
            blocking: self.blocking.clone(),
            blocked_by: self.blocked_by.clone(),
        };

        let yaml = serde_yaml::to_string(&fm)?;

        let mut out = String::new();
        out.push_str("---\n");
        out.push_str(&format!("# {}\n", self.id));
        out.push_str(&yaml);
        out.push_str("---\n");

        if !self.body.is_empty() {
            out.push('\n');
            out.push_str(&self.body);
            if !self.body.ends_with('\n') {
                out.push('\n');
            }
        }

        Ok(out)
    }

    pub fn etag(&self) -> String {
        match self.try_render() {
            Ok(rendered) => {
                let mut hasher = FnvHasher::default();
                hasher.write(rendered.as_bytes());
                format!("{:016x}", hasher.finish())
            }
            Err(_) => "0000000000000000".to_string(),
        }
    }

    /// Convert to JSON-serializable representation.
    pub fn to_json(&self, etag: &str) -> IshooJson {
        IshooJson {
            id: self.id.clone(),
            slug: self.slug.clone(),
            path: self.path.clone(),
            title: self.title.clone(),
            status: self.status.clone(),
            ishoo_type: self.ishoo_type.clone(),
            priority: self.priority.clone(),
            tags: self.tags.clone(),
            created_at: self.created_at,
            updated_at: self.updated_at,
            order: self.order.clone(),
            body: self.body.clone(),
            parent: self.parent.clone(),
            blocking: self.blocking.clone(),
            blocked_by: self.blocked_by.clone(),
            etag: etag.to_string(),
        }
    }
}

/// Split a file's content into `(id, yaml, body)`.
///
/// Expects the format:
/// ```text
/// ---
/// # {id}
/// yaml...
/// ---
///
/// body...
/// ```
fn split_frontmatter(content: &str) -> Result<(String, String, String), ParseError> {
    let trimmed = content.trim_start();

    if !trimmed.starts_with("---") {
        return Err(ParseError::MissingFrontmatter);
    }

    // Skip past the opening `---` line.
    let after_open = &trimmed[3..];
    let after_open = after_open.strip_prefix('\n').unwrap_or(after_open);

    // Find the closing `---`.
    let close_pos = after_open
        .find("\n---")
        .ok_or(ParseError::MissingFrontmatter)?;

    let frontmatter_block = &after_open[..close_pos];
    let after_close = &after_open[close_pos + 4..]; // skip "\n---"

    // Extract the `# {id}` line.
    let mut id = String::new();
    let mut yaml_lines = Vec::new();

    for line in frontmatter_block.lines() {
        if line.starts_with("# ") && id.is_empty() {
            id = line[2..].trim().to_string();
        } else {
            yaml_lines.push(line);
        }
    }

    if id.is_empty() {
        return Err(ParseError::MissingId);
    }

    let yaml_str = yaml_lines.join("\n");

    // Body is everything after the closing `---`, trimmed of surrounding whitespace.
    let body = after_close.trim().to_string();

    Ok((id, yaml_str, body))
}

/// Parse an issue filename into `(id, slug)`.
///
/// Supports `{id}--{slug}.md` and `{id}.md`.
pub fn new_id(prefix: &str, length: usize) -> String {
    loop {
        let suffix = nanoid!(length, &ID_ALPHABET);
        let id = if prefix.is_empty() {
            suffix
        } else {
            format!("{prefix}-{suffix}")
        };

        if !contains_blocked_substring(&id) {
            return id;
        }
    }
}

pub fn parse_filename(filename: &str) -> (String, String) {
    let file_name = Path::new(filename)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or(filename);
    let name = file_name.strip_suffix(".md").unwrap_or(file_name);

    if let Some((id, slug)) = name.split_once("--") {
        (id.to_string(), slug.to_string())
    } else {
        (name.to_string(), String::new())
    }
}

pub fn build_filename(id: &str, slug: &str) -> String {
    if slug.is_empty() {
        format!("{id}.md")
    } else {
        format!("{id}--{slug}.md")
    }
}

pub fn slugify(title: &str) -> String {
    let mut slug = String::new();

    for ch in title.chars().flat_map(char::to_lowercase) {
        if ch.is_ascii_lowercase() || ch.is_ascii_digit() {
            slug.push(ch);
            continue;
        }

        if (ch.is_whitespace() || ch == '_' || ch == '-')
            && !slug.is_empty()
            && !slug.ends_with('-')
        {
            slug.push('-');
        }
    }

    if slug.len() > 50 {
        slug.truncate(50);
    }

    slug.trim_matches('-').to_string()
}

fn contains_blocked_substring(value: &str) -> bool {
    let lowered = value.to_ascii_lowercase();

    BLOCKED_SUBSTRINGS
        .iter()
        .any(|blocked| lowered.contains(blocked))
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn sample_content() -> String {
        r#"---
# ish-a1b2
title: Fix the widget
status: todo
type: bug
priority: high
tags:
    - ui
    - regression
created_at: 2026-01-15T10:30:00Z
updated_at: 2026-01-16T14:00:00Z
parent: ish-p001
blocking:
    - ish-x001
blocked_by:
    - ish-y001
---

## Description

The widget is broken.

## Steps to Reproduce

1. Click the button
2. Observe the error
"#
        .to_string()
    }

    fn sample_ishoo() -> Ishoo {
        Ishoo {
            id: "ish-a1b2".to_string(),
            slug: "fix-the-widget".to_string(),
            path: "ish-a1b2--fix-the-widget.md".to_string(),
            title: "Fix the widget".to_string(),
            status: "todo".to_string(),
            ishoo_type: "bug".to_string(),
            priority: Some("high".to_string()),
            tags: vec!["ui".to_string(), "regression".to_string()],
            created_at: Utc.with_ymd_and_hms(2026, 1, 15, 10, 30, 0).unwrap(),
            updated_at: Utc.with_ymd_and_hms(2026, 1, 16, 14, 0, 0).unwrap(),
            order: None,
            body: "## Description\n\nThe widget is broken.\n\n## Steps to Reproduce\n\n1. Click the button\n2. Observe the error".to_string(),
            parent: Some("ish-p001".to_string()),
            blocking: vec!["ish-x001".to_string()],
            blocked_by: vec!["ish-y001".to_string()],
        }
    }

    #[test]
    fn test_parse_basic() {
        let ishoo =
            Ishoo::parse("ish-a1b2--fix-the-widget.md", &sample_content()).expect("parse failed");

        assert_eq!(ishoo.id, "ish-a1b2");
        assert_eq!(ishoo.slug, "fix-the-widget");
        assert_eq!(ishoo.path, "ish-a1b2--fix-the-widget.md");
        assert_eq!(ishoo.title, "Fix the widget");
        assert_eq!(ishoo.status, "todo");
        assert_eq!(ishoo.ishoo_type, "bug");
        assert_eq!(ishoo.priority, Some("high".to_string()));
        assert_eq!(ishoo.tags, vec!["ui", "regression"]);
        assert_eq!(ishoo.parent, Some("ish-p001".to_string()));
        assert_eq!(ishoo.blocking, vec!["ish-x001"]);
        assert_eq!(ishoo.blocked_by, vec!["ish-y001"]);
        assert!(ishoo.body.contains("The widget is broken."));
        assert!(ishoo.body.contains("Steps to Reproduce"));
    }

    #[test]
    fn test_parse_minimal() {
        let content = r#"---
# ish-min1
title: Minimal issue
status: todo
type: task
created_at: 2026-01-01T00:00:00Z
updated_at: 2026-01-01T00:00:00Z
---
"#;
        let ishoo = Ishoo::parse("ish-min1.md", content).expect("parse failed");

        assert_eq!(ishoo.id, "ish-min1");
        assert_eq!(ishoo.slug, "");
        assert_eq!(ishoo.title, "Minimal issue");
        assert_eq!(ishoo.priority, None);
        assert!(ishoo.tags.is_empty());
        assert!(ishoo.body.is_empty());
        assert!(ishoo.parent.is_none());
        assert!(ishoo.blocking.is_empty());
        assert!(ishoo.blocked_by.is_empty());
    }

    #[test]
    fn test_render_round_trip() {
        let original = sample_ishoo();
        let rendered = original.render();
        let parsed = Ishoo::parse("ish-a1b2--fix-the-widget.md", &rendered)
            .expect("round-trip parse failed");

        assert_eq!(original.id, parsed.id);
        assert_eq!(original.title, parsed.title);
        assert_eq!(original.status, parsed.status);
        assert_eq!(original.ishoo_type, parsed.ishoo_type);
        assert_eq!(original.priority, parsed.priority);
        assert_eq!(original.tags, parsed.tags);
        assert_eq!(original.parent, parsed.parent);
        assert_eq!(original.blocking, parsed.blocking);
        assert_eq!(original.blocked_by, parsed.blocked_by);
        assert_eq!(original.body, parsed.body);
    }

    #[test]
    fn test_render_empty_body() {
        let mut ishoo = sample_ishoo();
        ishoo.body = String::new();

        let rendered = ishoo.render();
        assert!(rendered.ends_with("---\n"));

        let parsed = Ishoo::parse("ish-a1b2--fix-the-widget.md", &rendered).expect("parse failed");
        assert!(parsed.body.is_empty());
    }

    #[test]
    fn test_render_minimal_fields() {
        let ishoo = Ishoo {
            id: "ish-x1".to_string(),
            slug: "simple".to_string(),
            path: "ish-x1--simple.md".to_string(),
            title: "Simple".to_string(),
            status: "todo".to_string(),
            ishoo_type: "task".to_string(),
            priority: None,
            tags: vec![],
            created_at: Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap(),
            updated_at: Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap(),
            order: None,
            body: String::new(),
            parent: None,
            blocking: vec![],
            blocked_by: vec![],
        };

        let rendered = ishoo.render();

        // Optional fields should not appear in the YAML.
        assert!(!rendered.contains("priority:"));
        assert!(!rendered.contains("tags:"));
        assert!(!rendered.contains("parent:"));
        assert!(!rendered.contains("blocking:"));
        assert!(!rendered.contains("blocked_by:"));
        assert!(!rendered.contains("order:"));
    }

    #[test]
    fn test_parse_missing_frontmatter() {
        let content = "Just some markdown\n\nNo frontmatter here.";
        let result = Ishoo::parse("bad.md", content);
        assert!(matches!(result, Err(ParseError::MissingFrontmatter)));
    }

    #[test]
    fn test_parse_missing_id() {
        let content = r#"---
title: No ID
status: todo
type: task
created_at: 2026-01-01T00:00:00Z
updated_at: 2026-01-01T00:00:00Z
---
"#;
        let result = Ishoo::parse("no-id.md", content);
        assert!(matches!(result, Err(ParseError::MissingId)));
    }

    #[test]
    fn test_parse_filename_with_slug() {
        let (id, slug) = parse_filename("ish-a1b2--fix-the-widget.md");
        assert_eq!(id, "ish-a1b2");
        assert_eq!(slug, "fix-the-widget");
    }

    #[test]
    fn test_parse_filename_without_slug() {
        let (id, slug) = parse_filename("ish-a1b2.md");
        assert_eq!(id, "ish-a1b2");
        assert_eq!(slug, "");
    }

    #[test]
    fn test_parse_filename_uses_basename() {
        let (id, slug) = parse_filename(".beans/ish-a1b2--fix-the-widget.md");
        assert_eq!(id, "ish-a1b2");
        assert_eq!(slug, "fix-the-widget");
    }

    #[test]
    fn test_build_filename_with_slug() {
        assert_eq!(
            build_filename("ish-a1b2", "fix-the-widget"),
            "ish-a1b2--fix-the-widget.md"
        );
    }

    #[test]
    fn test_build_filename_without_slug() {
        assert_eq!(build_filename("ish-a1b2", ""), "ish-a1b2.md");
    }

    #[test]
    fn test_filename_round_trip_with_slug() {
        let filename = build_filename("ish-a1b2", "fix-the-widget");
        let (id, slug) = parse_filename(&filename);

        assert_eq!(id, "ish-a1b2");
        assert_eq!(slug, "fix-the-widget");
    }

    #[test]
    fn test_filename_round_trip_without_slug() {
        let filename = build_filename("ish-a1b2", "");
        let (id, slug) = parse_filename(&filename);

        assert_eq!(id, "ish-a1b2");
        assert_eq!(slug, "");
    }

    #[test]
    fn test_slugify_normalizes_title() {
        assert_eq!(slugify("Fix the_widget"), "fix-the-widget");
    }

    #[test]
    fn test_slugify_strips_unicode_and_special_characters() {
        assert_eq!(slugify("Crème brûlée!!!"), "crme-brle");
    }

    #[test]
    fn test_slugify_collapses_repeated_separators() {
        assert_eq!(slugify("Hello___world -- again"), "hello-world-again");
    }

    #[test]
    fn test_slugify_truncates_to_fifty_characters() {
        let slug = slugify("abcdefghijklmnopqrstuvwxyz abcdefghijklmnopqrstuvwxyz");

        assert_eq!(slug, "abcdefghijklmnopqrstuvwxyz-abcdefghijklmnopqrstuvw");
        assert_eq!(slug.len(), 50);
    }

    #[test]
    fn test_new_id_uses_expected_format() {
        let id = new_id("ish", 6);

        assert!(id.starts_with("ish-"));
        assert_eq!(id.len(), 10);
        assert!(
            id[4..]
                .chars()
                .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit())
        );
    }

    #[test]
    fn test_generated_ids_never_include_blocked_substrings() {
        for _ in 0..256 {
            let id = new_id("ish", 8);
            assert!(
                !contains_blocked_substring(&id),
                "generated blocked id: {id}"
            );
        }
    }

    #[test]
    fn test_contains_blocked_substring_detects_offensive_words() {
        assert!(contains_blocked_substring("ish-fuck"));
        assert!(!contains_blocked_substring("ish-safe1"));
    }

    #[test]
    fn test_to_json() {
        let ishoo = sample_ishoo();
        let json = ishoo.to_json("abc123");

        assert_eq!(json.id, "ish-a1b2");
        assert_eq!(json.etag, "abc123");
        assert_eq!(json.ishoo_type, "bug");

        // Verify it serializes to JSON without error.
        let json_str = serde_json::to_string(&json).expect("JSON serialize failed");
        assert!(json_str.contains("\"type\":\"bug\""));
        assert!(json_str.contains("\"etag\":\"abc123\""));
    }

    #[test]
    fn test_etag_is_deterministic_for_same_content() {
        let first = sample_ishoo();
        let second = sample_ishoo();

        assert_eq!(first.etag(), second.etag());
    }

    #[test]
    fn test_etag_changes_when_content_changes() {
        let first = sample_ishoo();
        let mut second = sample_ishoo();
        second.title = "Fix the other widget".to_string();

        assert_ne!(first.etag(), second.etag());
    }

    #[test]
    fn test_etag_matches_expected_hash() {
        let ishoo = sample_ishoo();

        assert_eq!(ishoo.etag(), "bf9cff0b18dcadde");
    }

    #[test]
    fn test_round_trip_parse_render_parse() {
        let content = sample_content();
        let first = Ishoo::parse("ish-a1b2--fix-the-widget.md", &content).expect("first parse");
        let rendered = first.render();
        let second = Ishoo::parse("ish-a1b2--fix-the-widget.md", &rendered).expect("second parse");

        assert_eq!(first, second);
    }
}

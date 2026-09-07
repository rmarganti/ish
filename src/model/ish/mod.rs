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

const ORDER_ALPHABET: &[u8; 62] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";

const BLOCKED_SUBSTRINGS: &[&str] = &[
    "anal", "anus", "arse", "ass", "bitch", "boob", "butt", "cock", "cunt", "damn", "dick",
    "dildo", "dyke", "fag", "fuck", "hell", "jizz", "knob", "penis", "piss", "poop", "porn",
    "pussy", "sex", "shit", "slut", "tit", "twat", "vag",
];

/// The core issue type. An Ish is stored as a markdown file with YAML frontmatter.
#[derive(Debug, Clone, PartialEq)]
pub struct Ish {
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
    pub ish_type: String,
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

/// The subset of `Ish` fields that are serialized into YAML frontmatter.
/// Excludes `id`, `slug`, `path`, and `body`.
#[derive(Debug, Serialize, Deserialize)]
struct Frontmatter {
    title: String,
    status: String,
    #[serde(rename = "type")]
    ish_type: String,
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
pub struct IshJson {
    pub id: String,
    pub slug: String,
    pub path: String,
    pub archived: bool,
    pub title: String,
    pub status: String,
    #[serde(rename = "type")]
    pub ish_type: String,
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TagError {
    InvalidTag,
}

impl fmt::Display for TagError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TagError::InvalidTag => write!(f, "invalid tag format"),
        }
    }
}

impl std::error::Error for TagError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BodyError {
    EmptyNeedle,
    NotFound,
    MultipleMatches,
}

impl fmt::Display for BodyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BodyError::EmptyNeedle => write!(f, "replacement text cannot be empty"),
            BodyError::NotFound => write!(f, "replacement target not found"),
            BodyError::MultipleMatches => write!(f, "replacement target matched multiple times"),
        }
    }
}

impl std::error::Error for BodyError {}

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

impl Ish {
    /// Parse an `Ish` from the raw content of a `.md` file.
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

        Ok(Ish {
            id,
            slug,
            path: filename.to_string(),
            title: fm.title,
            status: fm.status,
            ish_type: fm.ish_type,
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

    /// Render the `Ish` back to the `.md` file format.
    pub fn render(&self) -> String {
        self.try_render().expect("failed to serialize frontmatter")
    }

    fn try_render(&self) -> Result<String, serde_yaml::Error> {
        let fm = Frontmatter {
            title: self.title.clone(),
            status: self.status.clone(),
            ish_type: self.ish_type.clone(),
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

    pub fn is_archived(&self) -> bool {
        self.path.starts_with("archive/")
    }

    /// Convert to JSON-serializable representation.
    pub fn to_json(&self, etag: &str) -> IshJson {
        IshJson {
            id: self.id.clone(),
            slug: self.slug.clone(),
            path: self.path.clone(),
            archived: self.is_archived(),
            title: self.title.clone(),
            status: self.status.clone(),
            ish_type: self.ish_type.clone(),
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

    pub fn has_tag(&self, tag: &str) -> bool {
        let normalized = normalize_tag(tag);
        self.tags.iter().any(|existing| existing == &normalized)
    }

    pub fn add_tag(&mut self, tag: &str) -> Result<bool, TagError> {
        let normalized = normalize_tag(tag);

        if !validate_tag(&normalized) {
            return Err(TagError::InvalidTag);
        }

        if self.tags.iter().any(|existing| existing == &normalized) {
            return Ok(false);
        }

        self.tags.push(normalized);
        Ok(true)
    }

    pub fn remove_tag(&mut self, tag: &str) -> bool {
        let normalized = normalize_tag(tag);
        let original_len = self.tags.len();
        self.tags.retain(|existing| existing != &normalized);
        self.tags.len() != original_len
    }
}

pub fn validate_tag(tag: &str) -> bool {
    let mut chars = tag.chars();
    let Some(first) = chars.next() else {
        return false;
    };

    if !first.is_ascii_lowercase() {
        return false;
    }

    let mut previous_was_hyphen = false;

    for ch in chars {
        if ch == '-' {
            if previous_was_hyphen {
                return false;
            }

            previous_was_hyphen = true;
            continue;
        }

        if !ch.is_ascii_lowercase() && !ch.is_ascii_digit() {
            return false;
        }

        previous_was_hyphen = false;
    }

    !previous_was_hyphen
}

pub fn normalize_tag(tag: &str) -> String {
    tag.trim().to_ascii_lowercase()
}

pub fn replace_once(text: &str, old: &str, new: &str) -> Result<String, BodyError> {
    if old.is_empty() {
        return Err(BodyError::EmptyNeedle);
    }

    let Some(first_match) = text.find(old) else {
        return Err(BodyError::NotFound);
    };

    if text[first_match + old.len()..].contains(old) {
        return Err(BodyError::MultipleMatches);
    }

    let mut replaced = String::with_capacity(text.len() - old.len() + new.len());
    replaced.push_str(&text[..first_match]);
    replaced.push_str(new);
    replaced.push_str(&text[first_match + old.len()..]);
    Ok(replaced)
}

pub fn unescape_body(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    let mut chars = text.chars();

    while let Some(ch) = chars.next() {
        if ch != '\\' {
            result.push(ch);
            continue;
        }

        match chars.next() {
            Some('n') => result.push('\n'),
            Some('t') => result.push('\t'),
            Some('\\') => result.push('\\'),
            Some(other) => {
                result.push('\\');
                result.push(other);
            }
            None => result.push('\\'),
        }
    }

    result
}

pub fn append_with_separator(text: &str, addition: &str) -> String {
    if text.is_empty() {
        return addition.to_string();
    }

    if addition.is_empty() {
        return text.to_string();
    }

    format!("{text}\n\n{addition}")
}

pub fn midpoint() -> String {
    order_char(ORDER_ALPHABET.len() / 2).to_string()
}

pub fn increment_key(key: &str) -> Option<String> {
    if !is_valid_order_key(key) {
        return None;
    }

    Some(format!("{key}{}", midpoint()))
}

pub fn decrement_key(key: &str) -> Option<String> {
    if !is_valid_order_key(key) || key.is_empty() {
        return None;
    }

    let first = order_index(key.as_bytes()[0]).expect("validated order key should decode");
    if first > 0 {
        return Some(order_char(first / 2).to_string());
    }

    (key.len() > 1)
        .then(|| ORDER_ALPHABET[0] as char)
        .map(|ch| ch.to_string())
}

pub fn order_between(a: &str, b: &str) -> Option<String> {
    if !is_valid_order_key(a) || !is_valid_order_key(b) {
        return None;
    }

    if !b.is_empty() && a >= b {
        return None;
    }

    if a.is_empty() && b.is_empty() {
        return Some(midpoint());
    }

    if b.is_empty() {
        return increment_key(a);
    }

    let a_bytes = a.as_bytes();
    let b_bytes = b.as_bytes();
    let mut shared = 0;
    while shared < a_bytes.len() && shared < b_bytes.len() && a_bytes[shared] == b_bytes[shared] {
        shared += 1;
    }

    let prefix = &a[..shared];

    if shared == a_bytes.len() {
        let tail = decrement_key(&b[shared..])?;
        return Some(format!("{prefix}{tail}"));
    }

    let lower = order_index(a_bytes[shared]).expect("validated order key should decode");
    let upper = order_index(b_bytes[shared]).expect("validated order key should decode");

    if upper > lower + 1 {
        return Some(format!("{prefix}{}", order_char((lower + upper) / 2)));
    }

    Some(format!("{a}{}", midpoint()))
}

fn is_valid_order_key(key: &str) -> bool {
    key.bytes().all(|byte| order_index(byte).is_some())
}

fn order_index(byte: u8) -> Option<usize> {
    ORDER_ALPHABET
        .iter()
        .position(|candidate| *candidate == byte)
}

fn order_char(index: usize) -> char {
    ORDER_ALPHABET[index] as char
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
    let prefix = prefix.trim_end_matches('-');

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
mod tests;

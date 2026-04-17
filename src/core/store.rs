use crate::config::Config;
use crate::model::ishoo::Ishoo;
use chrono::{DateTime, Utc};
use serde::Deserialize;
use std::collections::HashMap;
use std::ffi::OsStr;
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

const GITIGNORE_CONTENTS: &str = ".conversations/\n";

#[derive(Debug)]
pub struct Store {
    root: PathBuf,
    config: Config,
    ishoos: HashMap<String, Ishoo>,
}

#[derive(Debug)]
pub enum StoreError {
    Io(std::io::Error),
    InvalidPath(PathBuf),
    InvalidFrontmatter(PathBuf),
    Yaml {
        path: PathBuf,
        source: serde_yaml::Error,
    },
}

#[derive(Debug, Default, Deserialize)]
struct DiskFrontmatter {
    #[serde(default)]
    title: String,
    #[serde(default)]
    status: String,
    #[serde(rename = "type", default)]
    ishoo_type: String,
    #[serde(default)]
    priority: Option<String>,
    #[serde(default)]
    tags: Vec<String>,
    #[serde(default)]
    created_at: Option<DateTime<Utc>>,
    #[serde(default)]
    updated_at: Option<DateTime<Utc>>,
    #[serde(default)]
    order: Option<String>,
    #[serde(default)]
    parent: Option<String>,
    #[serde(default)]
    blocking: Vec<String>,
    #[serde(default)]
    blocked_by: Vec<String>,
}

impl Store {
    pub fn new(root: impl Into<PathBuf>, config: Config) -> Result<Self, StoreError> {
        let root = root.into();
        fs::create_dir_all(&root).map_err(StoreError::Io)?;

        let gitignore_path = root.join(".gitignore");
        if !gitignore_path.exists() {
            fs::write(gitignore_path, GITIGNORE_CONTENTS).map_err(StoreError::Io)?;
        }

        Ok(Self {
            root,
            config,
            ishoos: HashMap::new(),
        })
    }

    pub fn load(&mut self) -> Result<(), StoreError> {
        self.ishoos.clear();
        let root = self.root.clone();
        self.load_dir(&root)
    }

    pub fn all(&self) -> Vec<&Ishoo> {
        self.ishoos.values().collect()
    }

    pub fn get(&self, id: &str) -> Option<&Ishoo> {
        self.ishoos
            .get(id)
            .or_else(|| self.ishoos.get(&self.normalize_id(id)))
    }

    pub fn normalize_id(&self, id: &str) -> String {
        let prefix = self.config.ish.prefix.as_str();

        if prefix.is_empty() || id.starts_with(prefix) {
            id.to_string()
        } else {
            format!("{prefix}-{id}")
        }
    }

    fn load_dir(&mut self, dir: &Path) -> Result<(), StoreError> {
        for entry in fs::read_dir(dir).map_err(StoreError::Io)? {
            let entry = entry.map_err(StoreError::Io)?;
            let path = entry.path();
            let file_type = entry.file_type().map_err(StoreError::Io)?;

            if file_type.is_dir() {
                if path != self.root && is_hidden(&path) {
                    continue;
                }

                self.load_dir(&path)?;
                continue;
            }

            if file_type.is_file() && path.extension() == Some(OsStr::new("md")) {
                let ishoo = self.load_ishoo(&path)?;
                self.ishoos.insert(ishoo.id.clone(), ishoo);
            }
        }

        Ok(())
    }

    fn load_ishoo(&self, path: &Path) -> Result<Ishoo, StoreError> {
        let content = fs::read_to_string(path).map_err(StoreError::Io)?;
        let metadata = fs::metadata(path).map_err(StoreError::Io)?;
        let modified_at = metadata.modified().map_err(StoreError::Io)?;
        let modified_at = DateTime::<Utc>::from(modified_at);
        let (_, yaml, body) = split_frontmatter(&content)
            .ok_or_else(|| StoreError::InvalidFrontmatter(path.to_path_buf()))?;
        let fm: DiskFrontmatter =
            serde_yaml::from_str(&yaml).map_err(|source| StoreError::Yaml {
                path: path.to_path_buf(),
                source,
            })?;
        let relative_path = path
            .strip_prefix(&self.root)
            .map_err(|_| StoreError::InvalidPath(path.to_path_buf()))?;
        let relative_path = relative_path.to_string_lossy().replace('\\', "/");
        let filename = path
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or_else(|| StoreError::InvalidPath(path.to_path_buf()))?;
        let (id, slug) = crate::model::ishoo::parse_filename(filename);
        let created_at = fm.created_at.unwrap_or(modified_at);
        let updated_at = fm.updated_at.unwrap_or(created_at);

        Ok(Ishoo {
            id,
            slug,
            path: relative_path,
            title: fm.title,
            status: default_string(fm.status, &self.config.ish.default_status),
            ishoo_type: default_string(fm.ishoo_type, &self.config.ish.default_type),
            priority: Some(default_optional_string(fm.priority, "normal")),
            tags: fm.tags,
            created_at,
            updated_at,
            order: fm.order,
            body,
            parent: fm.parent,
            blocking: fm.blocking,
            blocked_by: fm.blocked_by,
        })
    }
}

impl fmt::Display for StoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StoreError::Io(error) => write!(f, "I/O error: {error}"),
            StoreError::InvalidPath(path) => write!(f, "invalid ishoo path: {}", path.display()),
            StoreError::InvalidFrontmatter(path) => {
                write!(f, "invalid frontmatter in `{}`", path.display())
            }
            StoreError::Yaml { path, source } => {
                write!(f, "failed to parse YAML in `{}`: {source}", path.display())
            }
        }
    }
}

impl std::error::Error for StoreError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            StoreError::Io(error) => Some(error),
            StoreError::Yaml { source, .. } => Some(source),
            StoreError::InvalidPath(_) | StoreError::InvalidFrontmatter(_) => None,
        }
    }
}

fn default_string(value: String, fallback: &str) -> String {
    if value.trim().is_empty() {
        fallback.to_string()
    } else {
        value
    }
}

fn default_optional_string(value: Option<String>, fallback: &str) -> String {
    match value {
        Some(value) if !value.trim().is_empty() => value,
        _ => fallback.to_string(),
    }
}

fn is_hidden(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name.starts_with('.'))
}

fn split_frontmatter(content: &str) -> Option<(String, String, String)> {
    let trimmed = content.trim_start();
    if !trimmed.starts_with("---") {
        return None;
    }

    let after_open = trimmed[3..].strip_prefix('\n').unwrap_or(&trimmed[3..]);
    let close_pos = after_open.find("\n---")?;
    let frontmatter_block = &after_open[..close_pos];
    let after_close = &after_open[close_pos + 4..];

    let mut yaml_lines = Vec::new();
    let mut id = None;

    for line in frontmatter_block.lines() {
        if id.is_none() && line.starts_with("# ") {
            id = Some(line[2..].trim().to_string());
        } else {
            yaml_lines.push(line);
        }
    }

    Some((id?, yaml_lines.join("\n"), after_close.trim().to_string()))
}

#[cfg(test)]
mod tests {
    use super::{GITIGNORE_CONTENTS, Store};
    use crate::config::Config;
    use chrono::{TimeZone, Utc};
    use std::fs;
    use std::path::{Path, PathBuf};
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
            let path = std::env::temp_dir().join(format!("ish-store-test-{unique}"));
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

    #[test]
    fn new_initializes_root_directory_and_gitignore() {
        let temp = TestDir::new();
        let root = temp.path().join(".ish");

        let _store = Store::new(&root, Config::default()).expect("store should initialize");

        assert!(root.is_dir());
        assert_eq!(
            fs::read_to_string(root.join(".gitignore")).expect("gitignore should exist"),
            GITIGNORE_CONTENTS
        );
    }

    #[test]
    fn load_reads_markdown_files_including_archive_and_skips_hidden_dirs() {
        let temp = TestDir::new();
        let root = temp.path().join(".ish");
        let archive_dir = root.join("archive");
        let hidden_dir = root.join(".conversations");

        fs::create_dir_all(&archive_dir).expect("archive dir should exist");
        fs::create_dir_all(&hidden_dir).expect("hidden dir should exist");
        write_ishoo(
            &root.join("ish-abcd--top-level.md"),
            "ish-abcd",
            "Top Level",
            "todo",
            "task",
            Some("normal"),
            "Top level body.",
        );
        write_ishoo(
            &archive_dir.join("ish-efgh--archived.md"),
            "ish-efgh",
            "Archived",
            "completed",
            "task",
            Some("low"),
            "Archived body.",
        );
        write_ishoo(
            &hidden_dir.join("ish-skip--hidden.md"),
            "ish-skip",
            "Hidden",
            "todo",
            "task",
            Some("normal"),
            "Hidden body.",
        );

        let mut store = Store::new(&root, Config::default()).expect("store should initialize");
        store.load().expect("store should load files");

        let mut ids = store
            .all()
            .into_iter()
            .map(|ishoo| ishoo.id.as_str())
            .collect::<Vec<_>>();
        ids.sort_unstable();

        assert_eq!(ids, vec!["ish-abcd", "ish-efgh"]);
        assert_eq!(
            store
                .get("ish-efgh")
                .expect("archived ishoo should load")
                .path,
            "archive/ish-efgh--archived.md"
        );
    }

    #[test]
    fn load_applies_defaults_for_empty_fields_and_uses_filename_metadata() {
        let temp = TestDir::new();
        let root = temp.path().join(".ish");
        let path = root.join("ish-abcd--needs-defaults.md");
        fs::create_dir_all(&root).expect("root dir should exist");
        fs::write(
            &path,
            "---\n# ignored-frontmatter-id\ntitle: Needs defaults\nstatus: \ntype: \npriority: \ntags: []\nupdated_at: 2026-01-02T03:04:05Z\n---\n\nBody text.\n",
        )
        .expect("ishoo file should be written");

        let mut config = Config::default_with_prefix("ish");
        config.ish.default_status = "todo".to_string();
        config.ish.default_type = "task".to_string();

        let mut store = Store::new(&root, config).expect("store should initialize");
        store.load().expect("store should load files");

        let ishoo = store.get("abcd").expect("normalized id should resolve");

        assert_eq!(ishoo.id, "ish-abcd");
        assert_eq!(ishoo.slug, "needs-defaults");
        assert_eq!(ishoo.path, "ish-abcd--needs-defaults.md");
        assert_eq!(ishoo.status, "todo");
        assert_eq!(ishoo.ishoo_type, "task");
        assert_eq!(ishoo.priority.as_deref(), Some("normal"));
        assert!(ishoo.tags.is_empty());
        assert!(ishoo.blocking.is_empty());
        assert_eq!(
            ishoo.updated_at,
            Utc.with_ymd_and_hms(2026, 1, 2, 3, 4, 5).unwrap()
        );
        assert!(ishoo.created_at <= Utc::now());
    }

    #[test]
    fn normalize_id_preserves_existing_prefix() {
        let temp = TestDir::new();
        let root = temp.path().join(".ish");
        let store =
            Store::new(&root, Config::default_with_prefix("ish")).expect("store should initialize");

        assert_eq!(store.normalize_id("abcd"), "ish-abcd");
        assert_eq!(store.normalize_id("ish-abcd"), "ish-abcd");
    }

    fn write_ishoo(
        path: &Path,
        id: &str,
        title: &str,
        status: &str,
        ishoo_type: &str,
        priority: Option<&str>,
        body: &str,
    ) {
        let priority_line = priority
            .map(|priority| format!("priority: {priority}\n"))
            .unwrap_or_default();
        let contents = format!(
            "---\n# {id}\ntitle: {title}\nstatus: {status}\ntype: {ishoo_type}\n{priority_line}created_at: 2026-01-01T00:00:00Z\nupdated_at: 2026-01-01T00:00:00Z\n---\n\n{body}\n"
        );
        fs::write(path, contents).expect("ishoo file should be written");
    }
}

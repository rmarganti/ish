use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, MutexGuard, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};

pub(crate) struct TestDir {
    path: PathBuf,
}

impl TestDir {
    pub(crate) fn new() -> Self {
        let unique = next_unique_suffix();
        let path = std::env::temp_dir().join(format!("ish-test-{unique}"));
        fs::create_dir_all(&path).expect("temp dir should be created");

        Self { path }
    }

    pub(crate) fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TestDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

pub(crate) struct WorkingDirGuard {
    _lock: MutexGuard<'static, ()>,
    original: PathBuf,
}

impl WorkingDirGuard {
    pub(crate) fn change_to(path: &Path) -> Self {
        let lock = cwd_lock()
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
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

fn next_unique_suffix() -> String {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after unix epoch")
        .as_nanos();
    let counter = COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("{}-{}-{}", std::process::id(), timestamp, counter)
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn write_test_ish(
    root: &Path,
    id: &str,
    title: &str,
    status: &str,
    ish_type: &str,
    priority: Option<&str>,
    body: &str,
    parent: Option<&str>,
    blocking: &[&str],
    blocked_by: &[&str],
    tags: &[&str],
) {
    let mut content = format!(
        "---\n# {id}\ntitle: {title}\nstatus: {status}\ntype: {ish_type}\ncreated_at: 2026-01-01T00:00:00Z\nupdated_at: 2026-01-01T00:00:00Z\n"
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
    .expect("ish file should be written");
}

#[cfg(test)]
pub(crate) mod tui {
    use crate::config::Config;
    use crate::model::ish::Ish;
    use crate::tui::{BoardState, Effect, IshType, Model, Msg, Priority, Screen};
    use chrono::{TimeZone, Utc};
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    #[derive(Debug, Clone)]
    pub(crate) struct IshBuilder {
        id: String,
        title: String,
        status: String,
        ish_type: String,
        priority: Option<String>,
        tags: Vec<String>,
        updated_at: chrono::DateTime<Utc>,
        body: String,
        path: Option<String>,
        parent: Option<String>,
        blocking: Vec<String>,
        blocked_by: Vec<String>,
    }

    impl IshBuilder {
        pub(crate) fn new(id: &str) -> Self {
            let normalized_id = if id.starts_with("ish-") {
                id.to_string()
            } else {
                format!("ish-{id}")
            };

            Self {
                title: normalized_id.clone(),
                path: Some(format!("{}--{}.md", normalized_id, normalized_id)),
                id: normalized_id,
                status: "todo".to_string(),
                ish_type: "task".to_string(),
                priority: Some("normal".to_string()),
                tags: Vec::new(),
                updated_at: Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap(),
                body: String::new(),
                parent: None,
                blocking: Vec::new(),
                blocked_by: Vec::new(),
            }
        }

        pub(crate) fn title(mut self, title: &str) -> Self {
            self.title = title.to_string();
            self
        }

        pub(crate) fn status(mut self, status: &str) -> Self {
            self.status = status.to_string();
            self
        }

        pub(crate) fn ish_type(mut self, ish_type: IshType) -> Self {
            self.ish_type = ish_type.as_str().to_string();
            self
        }

        pub(crate) fn priority(mut self, priority: Priority) -> Self {
            self.priority = Some(priority.as_str().to_string());
            self
        }

        pub(crate) fn no_priority(mut self) -> Self {
            self.priority = None;
            self
        }

        pub(crate) fn tags(mut self, tags: &[&str]) -> Self {
            self.tags = tags.iter().map(|tag| (*tag).to_string()).collect();
            self
        }

        pub(crate) fn updated_at(mut self, year: i32, month: u32, day: u32) -> Self {
            self.updated_at = Utc.with_ymd_and_hms(year, month, day, 0, 0, 0).unwrap();
            self
        }

        pub(crate) fn body(mut self, body: &str) -> Self {
            self.body = body.to_string();
            self
        }

        pub(crate) fn path(mut self, path: &str) -> Self {
            self.path = Some(path.to_string());
            self
        }

        pub(crate) fn parent(mut self, parent: &str) -> Self {
            self.parent = Some(parent.to_string());
            self
        }

        pub(crate) fn blocking(mut self, blocking: &[&str]) -> Self {
            self.blocking = blocking.iter().map(|id| (*id).to_string()).collect();
            self
        }

        pub(crate) fn blocked_by(mut self, blocked_by: &[&str]) -> Self {
            self.blocked_by = blocked_by.iter().map(|id| (*id).to_string()).collect();
            self
        }

        pub(crate) fn build(self) -> Ish {
            let slug = self.title.to_ascii_lowercase().replace(' ', "-");

            Ish {
                id: self.id.clone(),
                slug: slug.clone(),
                path: self
                    .path
                    .unwrap_or_else(|| format!("{}--{}.md", self.id, slug)),
                title: self.title,
                status: self.status,
                ish_type: self.ish_type,
                priority: self.priority,
                tags: self.tags,
                created_at: Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap(),
                updated_at: self.updated_at,
                order: None,
                body: self.body,
                parent: self.parent,
                blocking: self.blocking,
                blocked_by: self.blocked_by,
            }
        }
    }

    pub(crate) fn model_with_board(ishes: Vec<Ish>) -> Model {
        let mut model = Model::new(Config::default());
        model.issues = ishes;
        model.etags = model
            .issues
            .iter()
            .map(|ish| (ish.id.clone(), ish.etag()))
            .collect();
        model.screens = vec![Screen::Board(BoardState::default())];
        model.status_line = None;
        model.status_line_set_at = None;
        model
    }

    pub(crate) fn dispatch(mut model: Model, msgs: &[Msg]) -> (Model, Vec<Effect>) {
        let mut effects = Vec::new();

        for msg in msgs {
            let (updated_model, mut new_effects) = crate::tui::update::update(model, msg.clone());
            model = updated_model;
            effects.append(&mut new_effects);
        }

        (model, effects)
    }

    pub(crate) fn key(code: KeyCode, mods: KeyModifiers) -> KeyEvent {
        KeyEvent::new(code, mods)
    }
}

#[cfg(test)]
#[macro_export]
macro_rules! k {
    ($code:expr) => {
        $crate::test_support::tui::key($code, crossterm::event::KeyModifiers::NONE)
    };
    ($code:expr, $mods:expr) => {
        $crate::test_support::tui::key($code, $mods)
    };
}

#[cfg(test)]
mod tests {
    use super::tui::{IshBuilder, dispatch, model_with_board};
    use crate::tui::{IshType, Msg, Priority, Screen};
    use crossterm::event::{KeyCode, KeyModifiers};

    #[test]
    fn tui_ish_builder_builds_sensible_defaults_and_overrides() {
        let ish = IshBuilder::new("abcd")
            .title("Ship TUI")
            .status("in-progress")
            .ish_type(IshType::Feature)
            .priority(Priority::High)
            .tags(&["tui", "cli"])
            .updated_at(2026, 4, 25)
            .body("Body")
            .path("custom/ish-abcd--ship-tui.md")
            .parent("ish-parent")
            .blocking(&["ish-blocking"])
            .blocked_by(&["ish-blocked-by"])
            .build();

        assert_eq!(ish.id, "ish-abcd");
        assert_eq!(ish.title, "Ship TUI");
        assert_eq!(ish.status, "in-progress");
        assert_eq!(ish.ish_type, "feature");
        assert_eq!(ish.priority.as_deref(), Some("high"));
        assert_eq!(ish.tags, vec!["tui", "cli"]);
        assert_eq!(ish.path, "custom/ish-abcd--ship-tui.md");
        assert_eq!(ish.parent.as_deref(), Some("ish-parent"));
        assert_eq!(ish.blocking, vec!["ish-blocking"]);
        assert_eq!(ish.blocked_by, vec!["ish-blocked-by"]);

        let no_priority = IshBuilder::new("efgh").no_priority().build();
        assert_eq!(no_priority.priority, None);
    }

    #[test]
    fn tui_model_with_board_seeds_board_and_etags() {
        let model = model_with_board(vec![IshBuilder::new("abcd").title("Board card").build()]);

        assert!(matches!(model.screens.as_slice(), [Screen::Board(_)]));
        assert!(model.status_line.is_none());
        assert_eq!(model.etags.len(), 1);
        assert!(model.etags.contains_key("ish-abcd"));
    }

    #[test]
    fn tui_dispatch_accumulates_effects_and_key_helper_builds_events() {
        let model = model_with_board(vec![]);
        let (model, effects) = dispatch(model, &[Msg::Tick, Msg::Quit]);
        let key = k!(KeyCode::Char('j'), KeyModifiers::CONTROL);

        assert!(matches!(model.screens.as_slice(), [Screen::Board(_)]));
        assert!(effects.is_empty());
        assert_eq!(key.code, KeyCode::Char('j'));
        assert_eq!(key.modifiers, KeyModifiers::CONTROL);
    }
}

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

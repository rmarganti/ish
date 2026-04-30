use super::{
    CreateIsh, LinkCheckResult, LinkCycle, LinkRef, LinkType, Store, StoreError, UpdateIsh,
};
use crate::config::Config;
use chrono::{TimeZone, Utc};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

struct TestDir {
    path: PathBuf,
}

impl TestDir {
    fn new() -> Self {
        let unique = next_unique_suffix();
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

fn next_unique_suffix() -> String {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after unix epoch")
        .as_nanos();
    let counter = COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("{}-{}-{}", std::process::id(), timestamp, counter)
}

#[test]
fn new_initializes_root_directory() {
    let temp = TestDir::new();
    let root = temp.path().join(".ish");

    let _store = Store::new(&root, Config::default()).expect("store should initialize");

    assert!(root.is_dir());
    assert!(!root.join(".gitignore").exists());
}

#[test]
fn load_reads_markdown_files_including_archive_and_skips_hidden_dirs() {
    let temp = TestDir::new();
    let root = temp.path().join(".ish");
    let archive_dir = root.join("archive");
    let hidden_dir = root.join(".hidden");

    fs::create_dir_all(&archive_dir).expect("archive dir should exist");
    fs::create_dir_all(&hidden_dir).expect("hidden dir should exist");
    write_ish(
        &root.join("ish-abcd--top-level.md"),
        "ish-abcd",
        "Top Level",
        "todo",
        "task",
        Some("normal"),
        "Top level body.",
    );
    write_ish(
        &archive_dir.join("ish-efgh--archived.md"),
        "ish-efgh",
        "Archived",
        "completed",
        "task",
        Some("low"),
        "Archived body.",
    );
    write_ish(
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
        .map(|ish| ish.id.as_str())
        .collect::<Vec<_>>();
    ids.sort_unstable();

    assert_eq!(ids, vec!["ish-abcd", "ish-efgh"]);
    assert_eq!(
        store
            .get("ish-efgh")
            .expect("archived ish should load")
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
    .expect("ish file should be written");

    let mut config = Config::default_with_prefix("ish");
    config.ish.default_status = "todo".to_string();
    config.ish.default_type = "task".to_string();

    let mut store = Store::new(&root, config).expect("store should initialize");
    store.load().expect("store should load files");

    let ish = store.get("abcd").expect("normalized id should resolve");

    assert_eq!(ish.id, "ish-abcd");
    assert_eq!(ish.slug, "needs-defaults");
    assert_eq!(ish.path, "ish-abcd--needs-defaults.md");
    assert_eq!(ish.status, "todo");
    assert_eq!(ish.ish_type, "task");
    assert_eq!(ish.priority.as_deref(), Some("normal"));
    assert!(ish.tags.is_empty());
    assert!(ish.blocking.is_empty());
    assert_eq!(
        ish.updated_at,
        Utc.with_ymd_and_hms(2026, 1, 2, 3, 4, 5).unwrap()
    );
    assert!(ish.created_at <= Utc::now());
}

#[test]
fn load_one_reads_a_single_ish_by_full_or_short_id() {
    let temp = TestDir::new();
    let root = temp.path().join(".ish");
    let archive_dir = root.join("archive");

    fs::create_dir_all(&archive_dir).expect("archive dir should exist");
    write_ish(
        &archive_dir.join("ish-abcd--archived.md"),
        "ish-abcd",
        "Archived",
        "completed",
        "task",
        Some("normal"),
        "Archived body.",
    );

    let store = Store::new(&root, Config::default()).expect("store should initialize");

    let by_short_id = store.load_one("abcd").expect("short id should resolve");
    let by_full_id = store
        .load_one("ish-abcd")
        .expect("full id should resolve as well");

    assert_eq!(by_short_id.id, "ish-abcd");
    assert_eq!(by_short_id.path, "archive/ish-abcd--archived.md");
    assert_eq!(by_short_id.body, "Archived body.");
    assert_eq!(by_full_id.id, by_short_id.id);
    assert_eq!(by_full_id.path, by_short_id.path);
}

#[test]
fn load_one_returns_not_found_for_unknown_id() {
    let temp = TestDir::new();
    let root = temp.path().join(".ish");
    fs::create_dir_all(&root).expect("root dir should exist");

    let store = Store::new(&root, Config::default()).expect("store should initialize");
    let error = store
        .load_one("missing")
        .expect_err("unknown ids should return not found");

    assert!(matches!(error, StoreError::NotFound(ref id) if id == "ish-missing"));
}

#[test]
fn load_one_returns_parse_errors_for_corrupted_files() {
    let temp = TestDir::new();
    let root = temp.path().join(".ish");
    fs::create_dir_all(&root).expect("root dir should exist");
    fs::write(
        root.join("ish-bad1--corrupted.md"),
        "---\n# ish-bad1\ntitle: Corrupted\nstatus: todo\ntype: [\n---\n\nBody.\n",
    )
    .expect("corrupted ish file should be written");

    let store = Store::new(&root, Config::default()).expect("store should initialize");
    let error = store
        .load_one("bad1")
        .expect_err("corrupted files should surface parse errors");

    assert!(matches!(error, StoreError::Yaml { .. }));
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

#[test]
fn normalize_id_strips_trailing_dashes_from_prefix() {
    let temp = TestDir::new();
    let root = temp.path().join(".ish");
    let mut config = Config::default_with_prefix("ish");
    config.ish.prefix = "ish-".to_string();
    let store = Store::new(&root, config).expect("store should initialize");

    assert_eq!(store.normalize_id("abcd"), "ish-abcd");
    assert_eq!(store.normalize_id("ish-abcd"), "ish-abcd");
}

#[test]
fn normalize_id_requires_a_prefix_boundary() {
    let temp = TestDir::new();
    let root = temp.path().join(".ish");
    let store =
        Store::new(&root, Config::default_with_prefix("ish")).expect("store should initialize");

    assert_eq!(store.normalize_id("ishx-abcd"), "ish-ishx-abcd");
}

#[test]
fn archive_and_unarchive_move_files_and_update_store_paths() {
    let temp = TestDir::new();
    let root = temp.path().join(".ish");
    let active_path = root.join("ish-abcd--active.md");
    let archived_path = root.join("archive/ish-abcd--active.md");

    fs::create_dir_all(&root).expect("root dir should exist");
    write_ish(
        &active_path,
        "ish-abcd",
        "Active",
        "todo",
        "task",
        Some("normal"),
        "Body.",
    );

    let mut store = Store::new(&root, Config::default()).expect("store should initialize");
    store.load().expect("store should load files");

    store.archive("abcd").expect("archive should succeed");

    assert!(!active_path.exists());
    assert!(archived_path.exists());
    assert!(store.is_archived("ish-abcd").expect("ish should exist"));
    assert_eq!(
        store.get("ish-abcd").expect("ish should exist").path,
        "archive/ish-abcd--active.md"
    );

    store
        .unarchive("ish-abcd")
        .expect("unarchive should succeed");

    assert!(active_path.exists());
    assert!(!archived_path.exists());
    assert!(!store.is_archived("abcd").expect("ish should exist"));
    assert_eq!(
        store.get("ish-abcd").expect("ish should exist").path,
        "ish-abcd--active.md"
    );
}

#[test]
fn load_and_unarchive_restores_archived_file_into_store() {
    let temp = TestDir::new();
    let root = temp.path().join(".ish");
    let archive_dir = root.join("archive");
    let archived_path = archive_dir.join("ish-abcd--active.md");
    let active_path = root.join("ish-abcd--active.md");

    fs::create_dir_all(&archive_dir).expect("archive dir should exist");
    write_ish(
        &archived_path,
        "ish-abcd",
        "Active",
        "completed",
        "task",
        Some("normal"),
        "Body.",
    );

    let mut store = Store::new(&root, Config::default()).expect("store should initialize");

    store
        .load_and_unarchive("abcd")
        .expect("load and unarchive should succeed");

    assert!(active_path.exists());
    assert!(!archived_path.exists());
    assert_eq!(
        store.get("ish-abcd").expect("ish should be loaded").path,
        "ish-abcd--active.md"
    );
}

#[test]
fn archive_all_completed_moves_only_archive_statuses() {
    let temp = TestDir::new();
    let root = temp.path().join(".ish");

    fs::create_dir_all(&root).expect("root dir should exist");
    write_ish(
        &root.join("ish-todo--active.md"),
        "ish-todo",
        "Todo",
        "todo",
        "task",
        Some("normal"),
        "Todo body.",
    );
    write_ish(
        &root.join("ish-done--completed.md"),
        "ish-done",
        "Done",
        "completed",
        "task",
        Some("normal"),
        "Done body.",
    );
    write_ish(
        &root.join("ish-nope--scrapped.md"),
        "ish-nope",
        "Nope",
        "scrapped",
        "task",
        Some("normal"),
        "Nope body.",
    );

    let mut store = Store::new(&root, Config::default()).expect("store should initialize");
    store.load().expect("store should load files");

    let archived_count = store
        .archive_all_completed()
        .expect("bulk archive should succeed");

    assert_eq!(archived_count, 2);
    assert!(root.join("ish-todo--active.md").exists());
    assert!(root.join("archive/ish-done--completed.md").exists());
    assert!(root.join("archive/ish-nope--scrapped.md").exists());
    assert_eq!(
        store.get("ish-done").expect("done ish should exist").path,
        "archive/ish-done--completed.md"
    );
    assert_eq!(
        store
            .get("ish-nope")
            .expect("scrapped ish should exist")
            .path,
        "archive/ish-nope--scrapped.md"
    );
}

#[test]
fn create_writes_new_ish_to_disk_and_store() {
    let temp = TestDir::new();
    let root = temp.path().join(".ish");
    fs::create_dir_all(&root).expect("root dir should exist");
    write_ish(
        &root.join("ish-parent--parent.md"),
        "ish-parent",
        "Parent",
        "todo",
        "feature",
        Some("normal"),
        "Parent body.",
    );
    let mut store = Store::new(&root, Config::default()).expect("store should initialize");
    store.load().expect("store should load files");

    let created = store
        .create(CreateIsh {
            title: "Create store record".to_string(),
            status: None,
            ish_type: Some("bug".to_string()),
            priority: Some("high".to_string()),
            body: "Created body.".to_string(),
            tags: vec!["Backend".to_string(), "backend".to_string()],
            parent: Some("parent".to_string()),
            blocking: vec!["dep1".to_string(), "dep1".to_string()],
            blocked_by: vec!["dep2".to_string()],
            id_prefix: None,
        })
        .expect("create should succeed");

    let file_path = root.join(&created.path);
    let file_contents = fs::read_to_string(&file_path).expect("created file should exist");

    assert!(created.id.starts_with("ish-"));
    assert_eq!(created.slug, "create-store-record");
    assert_eq!(created.status, "todo");
    assert_eq!(created.ish_type, "bug");
    assert_eq!(created.priority.as_deref(), Some("high"));
    assert_eq!(created.tags, vec!["backend"]);
    assert_eq!(created.parent.as_deref(), Some("ish-parent"));
    assert_eq!(created.blocking, vec!["ish-dep1"]);
    assert_eq!(created.blocked_by, vec!["ish-dep2"]);
    assert!(file_contents.contains("title: Create store record"));
    assert!(file_contents.contains("Created body."));
    assert_eq!(
        store.get(&created.id).expect("created ish should exist"),
        &created
    );
}

#[test]
fn update_applies_field_changes_and_renames_file() {
    let temp = TestDir::new();
    let root = temp.path().join(".ish");
    let original_path = root.join("ish-abcd--old-title.md");

    fs::create_dir_all(&root).expect("root dir should exist");
    write_ish(
        &root.join("ish-parent--parent.md"),
        "ish-parent",
        "Parent",
        "todo",
        "epic",
        Some("normal"),
        "Parent body.",
    );
    write_ish(
        &original_path,
        "ish-abcd",
        "Old title",
        "todo",
        "task",
        Some("normal"),
        "alpha target omega",
    );

    let mut store = Store::new(&root, Config::default()).expect("store should initialize");
    store.load().expect("store should load files");
    let etag = store.get("ish-abcd").expect("ish should exist").etag();

    let updated = store
        .update(
            "abcd",
            UpdateIsh {
                status: Some("in-progress".to_string()),
                ish_type: Some("feature".to_string()),
                priority: Some(Some("critical".to_string())),
                title: Some("New title".to_string()),
                body: None,
                body_replace: Some(("target".to_string(), "updated".to_string())),
                body_append: Some("appended text".to_string()),
                add_tags: vec!["cli".to_string()],
                remove_tags: Vec::new(),
                parent: Some(Some("parent".to_string())),
                add_blocking: vec!["child".to_string()],
                remove_blocking: Vec::new(),
                add_blocked_by: vec!["dep".to_string()],
                remove_blocked_by: Vec::new(),
                if_match: Some(etag),
            },
        )
        .expect("update should succeed");

    let renamed_path = root.join("ish-abcd--new-title.md");
    let file_contents = fs::read_to_string(&renamed_path).expect("renamed file should exist");

    assert!(!original_path.exists());
    assert!(renamed_path.exists());
    assert_eq!(updated.status, "in-progress");
    assert_eq!(updated.ish_type, "feature");
    assert_eq!(updated.priority.as_deref(), Some("critical"));
    assert_eq!(updated.slug, "new-title");
    assert_eq!(updated.path, "ish-abcd--new-title.md");
    assert_eq!(updated.tags, vec!["cli"]);
    assert_eq!(updated.parent.as_deref(), Some("ish-parent"));
    assert_eq!(updated.blocking, vec!["ish-child"]);
    assert_eq!(updated.blocked_by, vec!["ish-dep"]);
    assert_eq!(updated.body, "alpha updated omega\n\nappended text");
    assert!(file_contents.contains("alpha updated omega"));
    assert!(file_contents.contains("appended text"));
}

#[test]
fn update_rejects_etag_mismatch() {
    let temp = TestDir::new();
    let root = temp.path().join(".ish");
    fs::create_dir_all(&root).expect("root dir should exist");
    write_ish(
        &root.join("ish-abcd--etag.md"),
        "ish-abcd",
        "ETag",
        "todo",
        "task",
        Some("normal"),
        "Body.",
    );

    let mut store = Store::new(&root, Config::default()).expect("store should initialize");
    store.load().expect("store should load files");

    let error = store
        .update(
            "ish-abcd",
            UpdateIsh {
                title: Some("Other".to_string()),
                if_match: Some("deadbeefdeadbeef".to_string()),
                ..UpdateIsh::default()
            },
        )
        .expect_err("update should fail on mismatched etag");

    assert!(matches!(error, StoreError::ETagMismatch { .. }));
}

#[test]
fn valid_parent_types_match_expected_hierarchy() {
    assert_eq!(Store::valid_parent_types("milestone"), &[] as &[&str]);
    assert_eq!(Store::valid_parent_types("epic"), &["milestone"]);
    assert_eq!(Store::valid_parent_types("feature"), &["milestone", "epic"]);
    assert_eq!(
        Store::valid_parent_types("task"),
        &["milestone", "epic", "feature"]
    );
    assert_eq!(
        Store::valid_parent_types("bug"),
        &["milestone", "epic", "feature"]
    );
}

#[test]
fn create_rejects_invalid_parent_hierarchy() {
    let temp = TestDir::new();
    let root = temp.path().join(".ish");

    fs::create_dir_all(&root).expect("root dir should exist");
    write_ish(
        &root.join("ish-task-parent--task-parent.md"),
        "ish-task-parent",
        "Task parent",
        "todo",
        "task",
        Some("normal"),
        "Task parent body.",
    );

    let mut store = Store::new(&root, Config::default()).expect("store should initialize");
    store.load().expect("store should load files");

    let error = store
        .create(CreateIsh {
            title: "Feature child".to_string(),
            ish_type: Some("feature".to_string()),
            parent: Some("task-parent".to_string()),
            ..CreateIsh::default()
        })
        .expect_err("create should reject invalid parent hierarchy");

    assert!(matches!(
        error,
        StoreError::InvalidParentType {
            child_type,
            parent_id,
            parent_type,
            allowed_parent_types,
        } if child_type == "feature"
            && parent_id == "ish-task-parent"
            && parent_type == "task"
            && allowed_parent_types == vec!["milestone", "epic"]
    ));
}

#[test]
fn create_rejects_parent_for_milestone() {
    let temp = TestDir::new();
    let root = temp.path().join(".ish");

    fs::create_dir_all(&root).expect("root dir should exist");
    write_ish(
        &root.join("ish-parent--parent.md"),
        "ish-parent",
        "Parent",
        "todo",
        "milestone",
        Some("normal"),
        "Parent body.",
    );

    let mut store = Store::new(&root, Config::default()).expect("store should initialize");
    store.load().expect("store should load files");

    let error = store
        .create(CreateIsh {
            title: "Milestone child".to_string(),
            ish_type: Some("milestone".to_string()),
            parent: Some("parent".to_string()),
            ..CreateIsh::default()
        })
        .expect_err("milestone should reject parent assignments");

    assert!(matches!(error, StoreError::ParentNotAllowed(ref ish_type) if ish_type == "milestone"));
}

#[test]
fn update_rejects_invalid_parent_hierarchy_after_type_change() {
    let temp = TestDir::new();
    let root = temp.path().join(".ish");

    fs::create_dir_all(&root).expect("root dir should exist");
    write_ish(
        &root.join("ish-parent--parent.md"),
        "ish-parent",
        "Parent",
        "todo",
        "feature",
        Some("normal"),
        "Parent body.",
    );
    write_ish(
        &root.join("ish-child--child.md"),
        "ish-child",
        "Child",
        "todo",
        "task",
        Some("normal"),
        "Child body.",
    );

    let mut store = Store::new(&root, Config::default()).expect("store should initialize");
    store.load().expect("store should load files");

    let error = store
        .update(
            "child",
            UpdateIsh {
                ish_type: Some("epic".to_string()),
                parent: Some(Some("parent".to_string())),
                ..UpdateIsh::default()
            },
        )
        .expect_err("update should reject invalid parent hierarchy");

    assert!(matches!(
        error,
        StoreError::InvalidParentType {
            child_type,
            parent_id,
            parent_type,
            allowed_parent_types,
        } if child_type == "epic"
            && parent_id == "ish-parent"
            && parent_type == "feature"
            && allowed_parent_types == vec!["milestone"]
    ));
}

#[test]
fn delete_removes_file_and_cleans_incoming_references() {
    let temp = TestDir::new();
    let root = temp.path().join(".ish");
    fs::create_dir_all(&root).expect("root dir should exist");
    fs::write(
        root.join("ish-target--target.md"),
        "---\n# ish-target\ntitle: Target\nstatus: todo\ntype: task\ncreated_at: 2026-01-01T00:00:00Z\nupdated_at: 2026-01-01T00:00:00Z\n---\n\nTarget body.\n",
    )
    .expect("target file should be written");
    fs::write(
        root.join("ish-ref--ref.md"),
        "---\n# ish-ref\ntitle: Ref\nstatus: todo\ntype: task\ncreated_at: 2026-01-01T00:00:00Z\nupdated_at: 2026-01-01T00:00:00Z\nparent: ish-target\nblocking:\n  - ish-target\nblocked_by:\n  - ish-target\n---\n\nRef body.\n",
    )
    .expect("ref file should be written");

    let mut store = Store::new(&root, Config::default()).expect("store should initialize");
    store.load().expect("store should load files");

    let removed = store.delete("target").expect("delete should succeed");
    let ref_ish = store.get("ish-ref").expect("ref should remain");
    let ref_contents =
        fs::read_to_string(root.join("ish-ref--ref.md")).expect("ref file should still exist");

    assert_eq!(removed.id, "ish-target");
    assert!(!root.join("ish-target--target.md").exists());
    assert!(ref_ish.parent.is_none());
    assert!(ref_ish.blocking.is_empty());
    assert!(ref_ish.blocked_by.is_empty());
    assert!(!ref_contents.contains("parent: ish-target"));
    assert!(!ref_contents.contains("- ish-target"));
}

#[test]
fn detect_cycle_finds_cycles_per_link_type() {
    let temp = TestDir::new();
    let root = temp.path().join(".ish");
    fs::create_dir_all(&root).expect("root dir should exist");
    fs::write(
        root.join("ish-a--a.md"),
        "---\n# ish-a\ntitle: A\nstatus: todo\ntype: task\ncreated_at: 2026-01-01T00:00:00Z\nupdated_at: 2026-01-01T00:00:00Z\nblocking:\n  - ish-b\n---\n\nA body.\n",
    )
    .expect("a file should be written");
    fs::write(
        root.join("ish-b--b.md"),
        "---\n# ish-b\ntitle: B\nstatus: todo\ntype: task\ncreated_at: 2026-01-01T00:00:00Z\nupdated_at: 2026-01-01T00:00:00Z\nblocking:\n  - ish-c\n---\n\nB body.\n",
    )
    .expect("b file should be written");
    fs::write(
        root.join("ish-c--c.md"),
        "---\n# ish-c\ntitle: C\nstatus: todo\ntype: task\ncreated_at: 2026-01-01T00:00:00Z\nupdated_at: 2026-01-01T00:00:00Z\n---\n\nC body.\n",
    )
    .expect("c file should be written");

    let mut store = Store::new(&root, Config::default()).expect("store should initialize");
    store.load().expect("store should load files");

    assert!(store.detect_cycle("ish-c", LinkType::Blocking, "ish-a"));
    assert!(!store.detect_cycle("ish-c", LinkType::BlockedBy, "ish-a"));
}

#[test]
fn find_incoming_links_returns_all_matching_link_types() {
    let temp = TestDir::new();
    let root = temp.path().join(".ish");
    fs::create_dir_all(&root).expect("root dir should exist");
    fs::write(
        root.join("ish-target--target.md"),
        "---\n# ish-target\ntitle: Target\nstatus: todo\ntype: task\ncreated_at: 2026-01-01T00:00:00Z\nupdated_at: 2026-01-01T00:00:00Z\n---\n\nTarget body.\n",
    )
    .expect("target file should be written");
    fs::write(
        root.join("ish-parented--parented.md"),
        "---\n# ish-parented\ntitle: Parented\nstatus: todo\ntype: task\ncreated_at: 2026-01-01T00:00:00Z\nupdated_at: 2026-01-01T00:00:00Z\nparent: ish-target\n---\n\nParented body.\n",
    )
    .expect("parented file should be written");
    fs::write(
        root.join("ish-blocker--blocker.md"),
        "---\n# ish-blocker\ntitle: Blocker\nstatus: todo\ntype: task\ncreated_at: 2026-01-01T00:00:00Z\nupdated_at: 2026-01-01T00:00:00Z\nblocking:\n  - ish-target\n---\n\nBlocker body.\n",
    )
    .expect("blocker file should be written");
    fs::write(
        root.join("ish-blocked--blocked.md"),
        "---\n# ish-blocked\ntitle: Blocked\nstatus: todo\ntype: task\ncreated_at: 2026-01-01T00:00:00Z\nupdated_at: 2026-01-01T00:00:00Z\nblocked_by:\n  - ish-target\n---\n\nBlocked body.\n",
    )
    .expect("blocked file should be written");

    let mut store = Store::new(&root, Config::default()).expect("store should initialize");
    store.load().expect("store should load files");

    assert_eq!(
        store.find_incoming_links("target"),
        vec![
            LinkRef {
                source_id: "ish-blocked".to_string(),
                link_type: LinkType::BlockedBy,
                target_id: "ish-target".to_string(),
            },
            LinkRef {
                source_id: "ish-blocker".to_string(),
                link_type: LinkType::Blocking,
                target_id: "ish-target".to_string(),
            },
            LinkRef {
                source_id: "ish-parented".to_string(),
                link_type: LinkType::Parent,
                target_id: "ish-target".to_string(),
            },
        ]
    );
}

#[test]
fn check_all_links_reports_broken_self_and_cycle_links() {
    let temp = TestDir::new();
    let root = temp.path().join(".ish");
    fs::create_dir_all(&root).expect("root dir should exist");
    fs::write(
        root.join("ish-a--a.md"),
        "---\n# ish-a\ntitle: A\nstatus: todo\ntype: task\ncreated_at: 2026-01-01T00:00:00Z\nupdated_at: 2026-01-01T00:00:00Z\nblocking:\n  - ish-b\n---\n\nA body.\n",
    )
    .expect("a file should be written");
    fs::write(
        root.join("ish-b--b.md"),
        "---\n# ish-b\ntitle: B\nstatus: todo\ntype: task\ncreated_at: 2026-01-01T00:00:00Z\nupdated_at: 2026-01-01T00:00:00Z\nblocking:\n  - ish-c\n---\n\nB body.\n",
    )
    .expect("b file should be written");
    fs::write(
        root.join("ish-c--c.md"),
        "---\n# ish-c\ntitle: C\nstatus: todo\ntype: task\ncreated_at: 2026-01-01T00:00:00Z\nupdated_at: 2026-01-01T00:00:00Z\nblocking:\n  - ish-a\n---\n\nC body.\n",
    )
    .expect("c file should be written");
    fs::write(
        root.join("ish-bad--bad.md"),
        "---\n# ish-bad\ntitle: Bad\nstatus: todo\ntype: task\ncreated_at: 2026-01-01T00:00:00Z\nupdated_at: 2026-01-01T00:00:00Z\nparent: ish-missing\nblocking:\n  - ish-bad\nblocked_by:\n  - ish-missing-two\n---\n\nBad body.\n",
    )
    .expect("bad file should be written");

    let mut store = Store::new(&root, Config::default()).expect("store should initialize");
    store.load().expect("store should load files");

    assert_eq!(
        store.check_all_links(),
        LinkCheckResult {
            broken_links: vec![
                LinkRef {
                    source_id: "ish-bad".to_string(),
                    link_type: LinkType::Parent,
                    target_id: "ish-missing".to_string(),
                },
                LinkRef {
                    source_id: "ish-bad".to_string(),
                    link_type: LinkType::BlockedBy,
                    target_id: "ish-missing-two".to_string(),
                },
            ],
            self_links: vec![LinkRef {
                source_id: "ish-bad".to_string(),
                link_type: LinkType::Blocking,
                target_id: "ish-bad".to_string(),
            }],
            cycles: vec![LinkCycle {
                link_type: LinkType::Blocking,
                path: vec![
                    "ish-a".to_string(),
                    "ish-b".to_string(),
                    "ish-c".to_string(),
                    "ish-a".to_string(),
                ],
            }],
        }
    );
}

#[test]
fn fix_broken_links_removes_invalid_references_and_saves_files() {
    let temp = TestDir::new();
    let root = temp.path().join(".ish");
    fs::create_dir_all(&root).expect("root dir should exist");
    fs::write(
        root.join("ish-valid--valid.md"),
        "---\n# ish-valid\ntitle: Valid\nstatus: todo\ntype: task\ncreated_at: 2026-01-01T00:00:00Z\nupdated_at: 2026-01-01T00:00:00Z\n---\n\nValid body.\n",
    )
    .expect("valid file should be written");
    fs::write(
        root.join("ish-bad--bad.md"),
        "---\n# ish-bad\ntitle: Bad\nstatus: todo\ntype: task\ncreated_at: 2026-01-01T00:00:00Z\nupdated_at: 2026-01-01T00:00:00Z\nparent: ish-bad\nblocking:\n  - ish-valid\n  - ish-bad\n  - ish-missing\nblocked_by:\n  - ish-bad\n  - ish-missing-two\n---\n\nBad body.\n",
    )
    .expect("bad file should be written");

    let mut store = Store::new(&root, Config::default()).expect("store should initialize");
    store.load().expect("store should load files");

    let fixed = store
        .fix_broken_links()
        .expect("fixing broken links should succeed");
    let ish = store.get("ish-bad").expect("bad ish should exist");
    let contents =
        fs::read_to_string(root.join("ish-bad--bad.md")).expect("bad file should still exist");

    assert_eq!(fixed, 5);
    assert!(ish.parent.is_none());
    assert_eq!(ish.blocking, vec!["ish-valid"]);
    assert!(ish.blocked_by.is_empty());
    assert!(!contents.contains("parent: ish-bad"));
    assert!(contents.contains("- ish-valid"));
    assert!(!contents.contains("- ish-missing"));
    assert!(!contents.contains("- ish-bad"));
    assert!(!contents.contains("- ish-missing-two"));
}

#[test]
fn blocker_queries_include_direct_blockers_from_both_link_directions() {
    let temp = TestDir::new();
    let root = temp.path().join(".ish");
    fs::create_dir_all(&root).expect("root dir should exist");
    fs::write(
        root.join("ish-target--target.md"),
        "---\n# ish-target\ntitle: Target\nstatus: todo\ntype: task\ncreated_at: 2026-01-01T00:00:00Z\nupdated_at: 2026-01-01T00:00:00Z\nblocked_by:\n  - ish-listed\n---\n\nTarget body.\n",
    )
    .expect("target file should be written");
    fs::write(
        root.join("ish-listed--listed.md"),
        "---\n# ish-listed\ntitle: Listed\nstatus: todo\ntype: task\ncreated_at: 2026-01-01T00:00:00Z\nupdated_at: 2026-01-01T00:00:00Z\n---\n\nListed body.\n",
    )
    .expect("listed blocker file should be written");
    fs::write(
        root.join("ish-incoming--incoming.md"),
        "---\n# ish-incoming\ntitle: Incoming\nstatus: todo\ntype: task\ncreated_at: 2026-01-01T00:00:00Z\nupdated_at: 2026-01-01T00:00:00Z\nblocking:\n  - ish-target\n---\n\nIncoming body.\n",
    )
    .expect("incoming blocker file should be written");
    fs::write(
        root.join("ish-resolved--resolved.md"),
        "---\n# ish-resolved\ntitle: Resolved\nstatus: completed\ntype: task\ncreated_at: 2026-01-01T00:00:00Z\nupdated_at: 2026-01-01T00:00:00Z\nblocking:\n  - ish-target\n---\n\nResolved body.\n",
    )
    .expect("resolved blocker file should be written");

    let mut store = Store::new(&root, Config::default()).expect("store should initialize");
    store.load().expect("store should load files");

    assert_eq!(
        store.find_active_blockers("target"),
        vec!["ish-incoming".to_string(), "ish-listed".to_string()]
    );
    assert!(store.is_explicitly_blocked("target"));
}

#[test]
fn archived_blockers_do_not_count_as_active_blockers() {
    let temp = TestDir::new();
    let root = temp.path().join(".ish");
    let archive_dir = root.join("archive");
    fs::create_dir_all(&archive_dir).expect("archive dir should exist");
    fs::write(
        root.join("ish-target--target.md"),
        "---\n# ish-target\ntitle: Target\nstatus: todo\ntype: task\ncreated_at: 2026-01-01T00:00:00Z\nupdated_at: 2026-01-01T00:00:00Z\nblocked_by:\n  - ish-archived-listed\n---\n\nTarget body.\n",
    )
    .expect("target file should be written");
    fs::write(
        archive_dir.join("ish-archived-listed--listed.md"),
        "---\n# ish-archived-listed\ntitle: Archived Listed\nstatus: todo\ntype: task\ncreated_at: 2026-01-01T00:00:00Z\nupdated_at: 2026-01-01T00:00:00Z\n---\n\nArchived listed blocker body.\n",
    )
    .expect("archived listed blocker file should be written");
    fs::write(
        archive_dir.join("ish-archived-incoming--incoming.md"),
        "---\n# ish-archived-incoming\ntitle: Archived Incoming\nstatus: todo\ntype: task\ncreated_at: 2026-01-01T00:00:00Z\nupdated_at: 2026-01-01T00:00:00Z\nblocking:\n  - ish-target\n---\n\nArchived incoming blocker body.\n",
    )
    .expect("archived incoming blocker file should be written");
    fs::write(
        root.join("ish-active-blocker--active-blocker.md"),
        "---\n# ish-active-blocker\ntitle: Active Blocker\nstatus: todo\ntype: task\ncreated_at: 2026-01-01T00:00:00Z\nupdated_at: 2026-01-01T00:00:00Z\nblocking:\n  - ish-target\n---\n\nActive blocker body.\n",
    )
    .expect("active blocker file should be written");

    let mut store = Store::new(&root, Config::default()).expect("store should initialize");
    store.load().expect("store should load files");

    assert_eq!(
        store.find_active_blockers("target"),
        vec!["ish-active-blocker".to_string()]
    );
}

#[test]
fn blocker_queries_include_ancestor_blockers() {
    let temp = TestDir::new();
    let root = temp.path().join(".ish");
    fs::create_dir_all(&root).expect("root dir should exist");
    fs::write(
        root.join("ish-parent--parent.md"),
        "---\n# ish-parent\ntitle: Parent\nstatus: todo\ntype: task\ncreated_at: 2026-01-01T00:00:00Z\nupdated_at: 2026-01-01T00:00:00Z\nblocked_by:\n  - ish-parent-blocker\n---\n\nParent body.\n",
    )
    .expect("parent file should be written");
    fs::write(
        root.join("ish-child--child.md"),
        "---\n# ish-child\ntitle: Child\nstatus: todo\ntype: task\ncreated_at: 2026-01-01T00:00:00Z\nupdated_at: 2026-01-01T00:00:00Z\nparent: ish-parent\n---\n\nChild body.\n",
    )
    .expect("child file should be written");
    fs::write(
        root.join("ish-parent-blocker--parent-blocker.md"),
        "---\n# ish-parent-blocker\ntitle: Parent Blocker\nstatus: todo\ntype: task\ncreated_at: 2026-01-01T00:00:00Z\nupdated_at: 2026-01-01T00:00:00Z\n---\n\nParent blocker body.\n",
    )
    .expect("parent blocker file should be written");

    let mut store = Store::new(&root, Config::default()).expect("store should initialize");
    store.load().expect("store should load files");

    assert_eq!(
        store.find_all_blockers("child"),
        vec!["ish-parent-blocker".to_string()]
    );
    assert!(store.is_blocked("child"));
    assert!(!store.is_explicitly_blocked("child"));
    assert!(store.is_implicitly_blocked("child"));
}

#[test]
fn implicit_status_returns_first_terminal_ancestor() {
    let temp = TestDir::new();
    let root = temp.path().join(".ish");
    fs::create_dir_all(&root).expect("root dir should exist");
    fs::write(
        root.join("ish-grandparent--grandparent.md"),
        "---\n# ish-grandparent\ntitle: Grandparent\nstatus: completed\ntype: task\ncreated_at: 2026-01-01T00:00:00Z\nupdated_at: 2026-01-01T00:00:00Z\n---\n\nGrandparent body.\n",
    )
    .expect("grandparent file should be written");
    fs::write(
        root.join("ish-parent--parent.md"),
        "---\n# ish-parent\ntitle: Parent\nstatus: todo\ntype: task\ncreated_at: 2026-01-01T00:00:00Z\nupdated_at: 2026-01-01T00:00:00Z\nparent: ish-grandparent\n---\n\nParent body.\n",
    )
    .expect("parent file should be written");
    fs::write(
        root.join("ish-child--child.md"),
        "---\n# ish-child\ntitle: Child\nstatus: todo\ntype: task\ncreated_at: 2026-01-01T00:00:00Z\nupdated_at: 2026-01-01T00:00:00Z\nparent: ish-parent\n---\n\nChild body.\n",
    )
    .expect("child file should be written");

    let mut store = Store::new(&root, Config::default()).expect("store should initialize");
    store.load().expect("store should load files");

    assert_eq!(
        store.implicit_status("child"),
        Some(("completed".to_string(), "ish-grandparent".to_string()))
    );
}

#[test]
fn parent_chain_cycle_does_not_loop_when_finding_inherited_state() {
    let temp = TestDir::new();
    let root = temp.path().join(".ish");
    fs::create_dir_all(&root).expect("root dir should exist");
    fs::write(
        root.join("ish-a--a.md"),
        "---\n# ish-a\ntitle: A\nstatus: todo\ntype: task\ncreated_at: 2026-01-01T00:00:00Z\nupdated_at: 2026-01-01T00:00:00Z\nparent: ish-b\nblocked_by:\n  - ish-blocker\n---\n\nA body.\n",
    )
    .expect("a file should be written");
    fs::write(
        root.join("ish-b--b.md"),
        "---\n# ish-b\ntitle: B\nstatus: todo\ntype: task\ncreated_at: 2026-01-01T00:00:00Z\nupdated_at: 2026-01-01T00:00:00Z\nparent: ish-a\n---\n\nB body.\n",
    )
    .expect("b file should be written");
    fs::write(
        root.join("ish-blocker--blocker.md"),
        "---\n# ish-blocker\ntitle: Blocker\nstatus: todo\ntype: task\ncreated_at: 2026-01-01T00:00:00Z\nupdated_at: 2026-01-01T00:00:00Z\n---\n\nBlocker body.\n",
    )
    .expect("blocker file should be written");

    let mut store = Store::new(&root, Config::default()).expect("store should initialize");
    store.load().expect("store should load files");

    assert_eq!(
        store.find_all_blockers("b"),
        vec!["ish-blocker".to_string()]
    );
    assert!(store.is_implicitly_blocked("b"));
    assert_eq!(store.implicit_status("b"), None);
}

fn write_ish(
    path: &Path,
    id: &str,
    title: &str,
    status: &str,
    ish_type: &str,
    priority: Option<&str>,
    body: &str,
) {
    let priority_line = priority
        .map(|priority| format!("priority: {priority}\n"))
        .unwrap_or_default();
    let contents = format!(
        "---\n# {id}\ntitle: {title}\nstatus: {status}\ntype: {ish_type}\n{priority_line}created_at: 2026-01-01T00:00:00Z\nupdated_at: 2026-01-01T00:00:00Z\n---\n\n{body}\n"
    );
    fs::write(path, contents).expect("ish file should be written");
}

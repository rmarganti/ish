use crate::config::Config;
use crate::core::store::Store;
use crate::test_support::{TestDir, write_test_ish};
use crate::tui::effect::{Effect, IssueDraft, IssuePatch, execute};
use crate::tui::{IshType, Msg, Priority, SaveFailure, SaveSuccess, Status};
use std::fs;
use std::path::PathBuf;

fn empty_store() -> (TestDir, Store) {
    let temp = TestDir::new();
    let root = temp.path().join(".ish");
    fs::create_dir_all(&root).expect("store root should exist");
    let store = Store::new(&root, Config::default()).expect("store should initialize");
    (temp, store)
}

fn store_with_issue() -> (TestDir, Store, PathBuf) {
    let temp = TestDir::new();
    let root = temp.path().join(".ish");
    fs::create_dir_all(&root).expect("store root should exist");
    write_test_ish(
        &root,
        "ish-abcd",
        "Ship kanban",
        "todo",
        "task",
        Some("normal"),
        "Initial body.",
        None,
        &[],
        &[],
        &["tui"],
    );

    let mut store = Store::new(&root, Config::default()).expect("store should initialize");
    store.load().expect("store should load");
    let path = root.join("ish-abcd--ship-kanban.md");
    (temp, store, path)
}

#[test]
fn create_issue_round_trips_through_store_and_disk() {
    let (_temp, mut store) = empty_store();

    let create_msgs = execute(
        Effect::CreateIssue {
            draft: IssueDraft {
                title: "Create form draft".to_string(),
                status: Status::Todo,
                ish_type: IshType::Feature,
                priority: Priority::High,
                tags: vec!["tui".to_string(), "kanban".to_string()],
                body: "Body from create form".to_string(),
                parent: None,
                blocking: Vec::new(),
                blocked_by: Vec::new(),
            },
            open_in_editor: false,
        },
        &mut store,
    );

    let created_id = match create_msgs.as_slice() {
        [
            Msg::SaveCompleted(SaveSuccess { id }),
            Msg::IssuesLoaded(Ok(issues)),
        ] => {
            let issue = issues
                .iter()
                .find(|issue| issue.id == *id)
                .expect("created issue should be present in reload");
            assert_eq!(issue.title, "Create form draft");
            assert_eq!(issue.status, "todo");
            assert_eq!(issue.ish_type, "feature");
            assert_eq!(issue.priority.as_deref(), Some("high"));
            assert_eq!(issue.tags, vec!["tui", "kanban"]);
            assert_eq!(issue.body, "Body from create form");
            id.clone()
        }
        other => panic!("unexpected msgs: {other:?}"),
    };

    let load_msgs = execute(Effect::LoadIssues, &mut store);
    let loaded_issue = match load_msgs.as_slice() {
        [Msg::IssuesLoaded(Ok(issues))] => issues
            .iter()
            .find(|issue| issue.id == created_id)
            .expect("created issue should load from disk"),
        other => panic!("unexpected msgs: {other:?}"),
    };

    let disk_contents = fs::read_to_string(store.root().join(&loaded_issue.path))
        .expect("created issue file should exist on disk");
    assert!(disk_contents.contains("title: Create form draft"));
    assert!(disk_contents.contains("status: todo"));
    assert!(disk_contents.contains("type: feature"));
    assert!(disk_contents.contains("priority: high"));
    assert!(disk_contents.contains("- tui"));
    assert!(disk_contents.contains("- kanban"));
    assert!(disk_contents.contains("Body from create form"));
}

#[test]
fn save_issue_updates_status_on_disk_and_changes_etag() {
    let (_temp, mut store, path) = store_with_issue();
    let previous = store.get("ish-abcd").expect("issue should exist").clone();
    let previous_etag = previous.etag();

    let msgs = execute(
        Effect::SaveIssue {
            patch: IssuePatch {
                id: "ish-abcd".to_string(),
                status: Some(Status::InProgress),
                priority: None,
            },
            etag: previous_etag.clone(),
        },
        &mut store,
    );

    match msgs.as_slice() {
        [
            Msg::SaveCompleted(SaveSuccess { id }),
            Msg::IssuesLoaded(Ok(issues)),
        ] => {
            assert_eq!(id, "ish-abcd");
            let issue = issues
                .iter()
                .find(|issue| issue.id == "ish-abcd")
                .expect("updated issue should be present after reload");
            assert_eq!(issue.status, "in-progress");
            assert_ne!(issue.etag(), previous_etag);
        }
        other => panic!("unexpected msgs: {other:?}"),
    }

    let disk_contents = fs::read_to_string(path).expect("updated issue file should exist");
    assert!(disk_contents.contains("status: in-progress"));

    let reloaded = store
        .load_one("ish-abcd")
        .expect("issue should reload from disk");
    assert_eq!(reloaded.status, "in-progress");
    assert_ne!(reloaded.etag(), previous_etag);
}

#[test]
fn stale_etag_conflict_is_reported_after_external_write_and_reload() {
    let (_temp, mut store, path) = store_with_issue();
    let stale_etag = store.get("ish-abcd").expect("issue should exist").etag();

    write_test_ish(
        path.parent().expect("issue path should have parent"),
        "ish-abcd",
        "Ship kanban",
        "in-progress",
        "task",
        Some("normal"),
        "External change.",
        None,
        &[],
        &[],
        &["tui"],
    );
    store.load().expect("store should reload external change");

    let msgs = execute(
        Effect::SaveIssue {
            patch: IssuePatch {
                id: "ish-abcd".to_string(),
                status: Some(Status::Completed),
                priority: None,
            },
            etag: stale_etag,
        },
        &mut store,
    );

    assert_eq!(
        msgs,
        vec![Msg::SaveFailed(SaveFailure::Conflict {
            id: "ish-abcd".to_string(),
        })]
    );

    let disk_contents =
        fs::read_to_string(path).expect("externally updated issue file should exist");
    assert!(disk_contents.contains("status: in-progress"));
    assert!(disk_contents.contains("External change."));
}

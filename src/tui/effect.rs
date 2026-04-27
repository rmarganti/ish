#![allow(dead_code)]

//! TUI effect execution over `core::store`.
//!
//! This module intentionally stays independent of `crossterm` so tests can
//! drive store-backed TUI workflows without a live terminal. The runtime owns
//! terminal-specific behavior such as raw mode and editor suspension.
//!
//! `Effect::OpenEditorForIssue` is therefore translated into
//! `Msg::EditorRequested(...)` rather than launching an editor here.

use crate::core::store::{CreateIsh, Store, StoreError, UpdateIsh};
use crate::model::ish::Ish;
use crate::tui::model::{IshType, Priority, Status};
use crate::tui::msg::{EditorRequest, Msg, SaveFailure, SaveSuccess};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IssuePatch {
    pub id: String,
    pub status: Option<Status>,
    pub priority: Option<Priority>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IssueDraft {
    pub title: String,
    pub status: Status,
    pub ish_type: IshType,
    pub priority: Priority,
    pub tags: Vec<String>,
    pub body: String,
    pub parent: Option<String>,
    pub blocking: Vec<String>,
    pub blocked_by: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Effect {
    LoadIssues,
    SaveIssue {
        patch: IssuePatch,
        etag: String,
    },
    CreateIssue {
        draft: IssueDraft,
        open_in_editor: bool,
    },
    OpenEditorForIssue {
        id: String,
    },
    Quit,
}

pub fn execute(effect: Effect, store: &mut Store) -> Vec<Msg> {
    match effect {
        Effect::LoadIssues => vec![load_issues(store)],
        Effect::SaveIssue { patch, etag } => save_issue(store, patch, etag),
        Effect::CreateIssue {
            draft,
            open_in_editor,
        } => create_issue(store, draft, open_in_editor),
        Effect::OpenEditorForIssue { id } => vec![Msg::EditorRequested(EditorRequest { id })],
        Effect::Quit => vec![Msg::Quit],
    }
}

fn load_issues(store: &mut Store) -> Msg {
    match store.load() {
        Ok(()) => Msg::IssuesLoaded(Ok(cloned_issues(store))),
        Err(error) => Msg::IssuesLoaded(Err(error.to_string())),
    }
}

fn save_issue(store: &mut Store, patch: IssuePatch, etag: String) -> Vec<Msg> {
    match store.update(
        &patch.id,
        UpdateIsh {
            status: patch.status.map(status_to_store_value),
            priority: patch.priority.map(priority_to_store_update_value),
            if_match: Some(etag),
            ..UpdateIsh::default()
        },
    ) {
        Ok(updated) => {
            let mut msgs = vec![Msg::SaveCompleted(SaveSuccess {
                id: updated.id.clone(),
            })];
            msgs.push(load_issues(store));
            msgs
        }
        Err(StoreError::ETagMismatch { .. }) => {
            vec![Msg::SaveFailed(SaveFailure::Conflict { id: patch.id })]
        }
        Err(error) => vec![Msg::SaveFailed(SaveFailure::Message(error.to_string()))],
    }
}

fn create_issue(store: &mut Store, draft: IssueDraft, open_in_editor: bool) -> Vec<Msg> {
    match store.create(CreateIsh {
        title: draft.title,
        status: Some(status_to_store_value(draft.status)),
        ish_type: Some(ish_type_to_store_value(draft.ish_type)),
        priority: Some(priority_to_store_value(draft.priority)),
        body: draft.body,
        tags: draft.tags,
        parent: draft.parent,
        blocking: draft.blocking,
        blocked_by: draft.blocked_by,
        id_prefix: None,
    }) {
        Ok(created) => {
            let mut msgs = vec![Msg::SaveCompleted(SaveSuccess {
                id: created.id.clone(),
            })];
            msgs.push(load_issues(store));
            if open_in_editor {
                msgs.push(Msg::EditorRequested(EditorRequest { id: created.id }));
            }
            msgs
        }
        Err(error) => vec![Msg::SaveFailed(SaveFailure::Message(error.to_string()))],
    }
}

fn cloned_issues(store: &Store) -> Vec<Ish> {
    let mut issues = store.all().into_iter().cloned().collect::<Vec<_>>();
    issues.sort_by(|left, right| left.id.cmp(&right.id));
    issues
}

fn status_to_store_value(status: Status) -> String {
    status.as_str().to_string()
}

fn ish_type_to_store_value(ish_type: IshType) -> String {
    ish_type.as_str().to_string()
}

fn priority_to_store_value(priority: Priority) -> String {
    priority.as_str().to_string()
}

fn priority_to_store_update_value(priority: Priority) -> Option<String> {
    Some(priority.as_str().to_string())
}

#[cfg(test)]
mod tests {
    use super::{Effect, IssueDraft, IssuePatch, execute};
    use crate::config::Config;
    use crate::core::store::Store;
    use crate::test_support::{TestDir, write_test_ish};
    use crate::tui::{EditorRequest, IshType, Msg, Priority, SaveFailure, SaveSuccess, Status};
    use std::fs;

    fn store_with_issue() -> (TestDir, Store) {
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
            "Body.",
            None,
            &[],
            &[],
            &["tui"],
        );

        let mut store = Store::new(&root, Config::default()).expect("store should initialize");
        store.load().expect("store should load");
        (temp, store)
    }

    #[test]
    fn execute_load_issues_returns_loaded_cache() {
        let (_temp, mut store) = store_with_issue();

        let msgs = execute(Effect::LoadIssues, &mut store);

        match msgs.as_slice() {
            [Msg::IssuesLoaded(Ok(issues))] => {
                assert_eq!(issues.len(), 1);
                assert_eq!(issues[0].id, "ish-abcd");
            }
            other => panic!("unexpected msgs: {other:?}"),
        }
    }

    #[test]
    fn execute_save_issue_updates_store_and_reloads() {
        let (_temp, mut store) = store_with_issue();
        let etag = store.get("ish-abcd").expect("issue should exist").etag();

        let msgs = execute(
            Effect::SaveIssue {
                patch: IssuePatch {
                    id: "ish-abcd".to_string(),
                    status: Some(Status::InProgress),
                    priority: None,
                },
                etag,
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
                    .expect("updated issue should be present");
                assert_eq!(issue.status, "in-progress");
            }
            other => panic!("unexpected msgs: {other:?}"),
        }
    }

    #[test]
    fn execute_save_issue_can_update_priority() {
        let (_temp, mut store) = store_with_issue();
        let etag = store.get("ish-abcd").expect("issue should exist").etag();

        let msgs = execute(
            Effect::SaveIssue {
                patch: IssuePatch {
                    id: "ish-abcd".to_string(),
                    status: None,
                    priority: Some(Priority::Critical),
                },
                etag,
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
                    .expect("updated issue should be present");
                assert_eq!(issue.priority.as_deref(), Some("critical"));
            }
            other => panic!("unexpected msgs: {other:?}"),
        }
    }

    #[test]
    fn execute_save_issue_surfaces_conflicts() {
        let (_temp, mut store) = store_with_issue();

        let msgs = execute(
            Effect::SaveIssue {
                patch: IssuePatch {
                    id: "ish-abcd".to_string(),
                    status: Some(Status::Completed),
                    priority: None,
                },
                etag: "stale-etag".to_string(),
            },
            &mut store,
        );

        assert_eq!(
            msgs,
            vec![Msg::SaveFailed(SaveFailure::Conflict {
                id: "ish-abcd".to_string(),
            })]
        );
    }

    #[test]
    fn execute_create_issue_returns_reload_and_optional_editor_request() {
        let temp = TestDir::new();
        let root = temp.path().join(".ish");
        fs::create_dir_all(&root).expect("store root should exist");
        let mut store = Store::new(&root, Config::default()).expect("store should initialize");

        let msgs = execute(
            Effect::CreateIssue {
                draft: IssueDraft {
                    title: "Create form draft".to_string(),
                    status: Status::Todo,
                    ish_type: IshType::Task,
                    priority: Priority::High,
                    tags: vec!["tui".to_string(), "draft".to_string()],
                    body: "Body from form".to_string(),
                    parent: None,
                    blocking: Vec::new(),
                    blocked_by: Vec::new(),
                },
                open_in_editor: true,
            },
            &mut store,
        );

        match msgs.as_slice() {
            [
                Msg::SaveCompleted(SaveSuccess { id }),
                Msg::IssuesLoaded(Ok(issues)),
                Msg::EditorRequested(EditorRequest { id: editor_id }),
            ] => {
                assert_eq!(id, editor_id);
                let issue = issues
                    .iter()
                    .find(|issue| issue.id == *id)
                    .expect("created issue should be present");
                assert_eq!(issue.title, "Create form draft");
                assert_eq!(issue.priority.as_deref(), Some("high"));
            }
            other => panic!("unexpected msgs: {other:?}"),
        }
    }

    #[test]
    fn execute_open_editor_and_quit_emit_runtime_markers() {
        let temp = TestDir::new();
        let root = temp.path().join(".ish");
        fs::create_dir_all(&root).expect("store root should exist");
        let mut store = Store::new(&root, Config::default()).expect("store should initialize");

        assert_eq!(
            execute(
                Effect::OpenEditorForIssue {
                    id: "ish-abcd".to_string(),
                },
                &mut store,
            ),
            vec![Msg::EditorRequested(EditorRequest {
                id: "ish-abcd".to_string(),
            })]
        );
        assert_eq!(execute(Effect::Quit, &mut store), vec![Msg::Quit]);
    }
}

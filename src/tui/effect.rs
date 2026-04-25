#![allow(dead_code)]

use crate::tui::model::{IshType, Priority, Status};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IssuePatch {
    pub id: String,
    pub status: Option<Status>,
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

#![allow(dead_code)]

use crate::config::Config;
use crate::model::ish::Ish;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::time::Instant;

pub type ConfigHandle = Config;

pub const BOARD_COLUMNS: [Status; 4] = [
    Status::Draft,
    Status::Todo,
    Status::InProgress,
    Status::Completed,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Status {
    Draft,
    Todo,
    InProgress,
    Completed,
    Scrapped,
}

impl Status {
    pub const ALL: [Self; 5] = [
        Self::Draft,
        Self::Todo,
        Self::InProgress,
        Self::Completed,
        Self::Scrapped,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Todo => "todo",
            Self::InProgress => "in-progress",
            Self::Completed => "completed",
            Self::Scrapped => "scrapped",
        }
    }

    pub fn from_str(value: &str) -> Option<Self> {
        match value {
            "draft" => Some(Self::Draft),
            "todo" => Some(Self::Todo),
            "in-progress" => Some(Self::InProgress),
            "completed" => Some(Self::Completed),
            "scrapped" => Some(Self::Scrapped),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IshType {
    Milestone,
    Epic,
    Bug,
    Feature,
    Task,
}

impl IshType {
    pub const ALL: [Self; 5] = [
        Self::Milestone,
        Self::Epic,
        Self::Bug,
        Self::Feature,
        Self::Task,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Milestone => "milestone",
            Self::Epic => "epic",
            Self::Bug => "bug",
            Self::Feature => "feature",
            Self::Task => "task",
        }
    }

    pub fn from_str(value: &str) -> Option<Self> {
        match value {
            "milestone" => Some(Self::Milestone),
            "epic" => Some(Self::Epic),
            "bug" => Some(Self::Bug),
            "feature" => Some(Self::Feature),
            "task" => Some(Self::Task),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Priority {
    Critical,
    High,
    Normal,
    Low,
    Deferred,
}

impl Priority {
    pub const ALL: [Self; 5] = [
        Self::Critical,
        Self::High,
        Self::Normal,
        Self::Low,
        Self::Deferred,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Critical => "critical",
            Self::High => "high",
            Self::Normal => "normal",
            Self::Low => "low",
            Self::Deferred => "deferred",
        }
    }

    pub fn from_str(value: &str) -> Option<Self> {
        match value {
            "critical" => Some(Self::Critical),
            "high" => Some(Self::High),
            "normal" => Some(Self::Normal),
            "low" => Some(Self::Low),
            "deferred" => Some(Self::Deferred),
            _ => None,
        }
    }

    fn rank(self) -> usize {
        match self {
            Self::Critical => 0,
            Self::High => 1,
            Self::Normal => 2,
            Self::Low => 3,
            Self::Deferred => 4,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Info,
    Success,
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatusLine {
    pub text: String,
    pub severity: Severity,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct BoardState {
    pub selected_column: usize,
    pub column_cursors: [Option<usize>; 4],
    pub column_offsets: [usize; 4],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DetailState {
    pub id: String,
    pub scroll: u16,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PickerState {
    pub issue_id: String,
    pub options: Vec<Status>,
    pub selected: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreateFormState {
    pub title: String,
    pub ish_type: IshType,
    pub priority: Priority,
    pub tags: String,
    pub focused_field: usize,
    pub pending_cancel: bool,
}

impl CreateFormState {
    pub fn new(config: &ConfigHandle) -> Self {
        Self {
            title: String::new(),
            ish_type: IshType::from_str(&config.ish.default_type).unwrap_or(IshType::Task),
            priority: Priority::Normal,
            tags: String::new(),
            focused_field: 0,
            pending_cancel: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct HelpState;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Screen {
    Board(BoardState),
    IssueDetail(DetailState),
    StatusPicker(PickerState),
    CreateForm(CreateFormState),
    Help(HelpState),
}

#[derive(Debug, Clone)]
pub struct Model {
    pub issues: Vec<Ish>,
    pub etags: HashMap<String, String>,
    pub config: ConfigHandle,
    pub screens: Vec<Screen>,
    pub status_line: Option<StatusLine>,
    pub status_line_set_at: Option<Instant>,
    pub quit: bool,
    pub term_too_small: bool,
}

impl Model {
    pub fn new(config: ConfigHandle) -> Self {
        Self {
            issues: Vec::new(),
            etags: HashMap::new(),
            config,
            screens: vec![Screen::Board(BoardState::default())],
            status_line: None,
            status_line_set_at: None,
            quit: false,
            term_too_small: false,
        }
    }

    pub fn bucket_for_status(&self, status: Status) -> Vec<&Ish> {
        let mut bucket = self
            .issues
            .iter()
            .filter(|ish| !is_archived(ish))
            .filter(|ish| ish.status != Status::Scrapped.as_str())
            .filter(|ish| ish.status == status.as_str())
            .collect::<Vec<_>>();

        bucket.sort_by(|left, right| compare_ish(left, right));
        bucket
    }
}

fn compare_ish(left: &Ish, right: &Ish) -> Ordering {
    priority_for_ish(left)
        .rank()
        .cmp(&priority_for_ish(right).rank())
        .then_with(|| right.updated_at.cmp(&left.updated_at))
        .then_with(|| compare_case_insensitive(&left.title, &right.title))
        .then_with(|| left.id.cmp(&right.id))
}

fn compare_case_insensitive(left: &str, right: &str) -> Ordering {
    left.to_ascii_lowercase()
        .cmp(&right.to_ascii_lowercase())
        .then_with(|| left.cmp(right))
}

fn is_archived(ish: &Ish) -> bool {
    ish.path.starts_with("archive/")
}

fn priority_for_ish(ish: &Ish) -> Priority {
    ish.priority
        .as_deref()
        .and_then(Priority::from_str)
        .unwrap_or(Priority::Normal)
}

#[cfg(test)]
mod tests {
    use super::{Model, Priority, Status};
    use crate::config::Config;
    use crate::model::ish::Ish;
    use chrono::{TimeZone, Utc};

    fn sample_ish(
        id: &str,
        title: &str,
        status: &str,
        priority: Option<&str>,
        updated_at: (i32, u32, u32, u32, u32, u32),
        path: &str,
    ) -> Ish {
        Ish {
            id: id.to_string(),
            slug: title.to_ascii_lowercase().replace(' ', "-"),
            path: path.to_string(),
            title: title.to_string(),
            status: status.to_string(),
            ish_type: "task".to_string(),
            priority: priority.map(str::to_string),
            tags: Vec::new(),
            created_at: Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap(),
            updated_at: Utc
                .with_ymd_and_hms(
                    updated_at.0,
                    updated_at.1,
                    updated_at.2,
                    updated_at.3,
                    updated_at.4,
                    updated_at.5,
                )
                .unwrap(),
            order: None,
            body: String::new(),
            parent: None,
            blocking: Vec::new(),
            blocked_by: Vec::new(),
        }
    }

    #[test]
    fn bucket_for_status_excludes_archived_and_scrapped_and_sorts_by_priority_then_updated_at() {
        let mut model = Model::new(Config::default());
        model.issues = vec![
            sample_ish(
                "ish-archived",
                "Archived completed",
                Status::Completed.as_str(),
                Some(Priority::Critical.as_str()),
                (2026, 1, 5, 0, 0, 0),
                "archive/ish-archived--archived-completed.md",
            ),
            sample_ish(
                "ish-scrapped",
                "Scrapped todo",
                Status::Scrapped.as_str(),
                Some(Priority::Critical.as_str()),
                (2026, 1, 6, 0, 0, 0),
                "ish-scrapped--scrapped-todo.md",
            ),
            sample_ish(
                "ish-high-new",
                "High new",
                Status::Todo.as_str(),
                Some(Priority::High.as_str()),
                (2026, 1, 4, 0, 0, 0),
                "ish-high-new--high-new.md",
            ),
            sample_ish(
                "ish-high-old",
                "High old",
                Status::Todo.as_str(),
                Some(Priority::High.as_str()),
                (2026, 1, 3, 0, 0, 0),
                "ish-high-old--high-old.md",
            ),
            sample_ish(
                "ish-normal",
                "Normal",
                Status::Todo.as_str(),
                None,
                (2026, 1, 7, 0, 0, 0),
                "ish-normal--normal.md",
            ),
            sample_ish(
                "ish-draft",
                "Draft only",
                Status::Draft.as_str(),
                Some(Priority::Critical.as_str()),
                (2026, 1, 8, 0, 0, 0),
                "ish-draft--draft-only.md",
            ),
        ];

        let todo_bucket = model.bucket_for_status(Status::Todo);
        let ids = todo_bucket
            .iter()
            .map(|ish| ish.id.as_str())
            .collect::<Vec<_>>();

        assert_eq!(ids, vec!["ish-high-new", "ish-high-old", "ish-normal"]);
        assert!(model.bucket_for_status(Status::Completed).is_empty());
    }
}

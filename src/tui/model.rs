#![allow(dead_code)]

use crate::config::Config;
use crate::model::ish::Ish;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct KeyPattern {
    pub code: KeyCode,
    pub modifiers: KeyModifiers,
}

impl KeyPattern {
    pub fn new(code: KeyCode, modifiers: KeyModifiers) -> Self {
        Self { code, modifiers }
    }
}

impl From<KeyEvent> for KeyPattern {
    fn from(value: KeyEvent) -> Self {
        Self {
            code: value.code,
            modifiers: value.modifiers,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct InputState {
    pub pending_keys: Vec<KeyPattern>,
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
pub struct PriorityPickerState {
    pub issue_id: String,
    pub options: Vec<Priority>,
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
    PriorityPicker(PriorityPickerState),
    CreateForm(CreateFormState),
    Help(HelpState),
}

#[derive(Debug, Clone)]
pub struct Model {
    pub issues: Vec<Ish>,
    pub etags: HashMap<String, String>,
    pub config: ConfigHandle,
    pub screens: Vec<Screen>,
    pub input: InputState,
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
            input: InputState::default(),
            status_line: None,
            status_line_set_at: None,
            quit: false,
            term_too_small: false,
        }
    }

    pub fn bucket_for_status(&self, status: Status) -> Vec<BoardRow<'_>> {
        let candidates = self
            .issues
            .iter()
            .filter(|ish| !is_archived(ish))
            .filter(|ish| ish.status != Status::Scrapped.as_str())
            .filter(|ish| ish.status == status.as_str())
            .collect::<Vec<_>>();

        build_tree_rows(&candidates)
    }
}

/// A single row in a board column, carrying the ish along with its position
/// in the parent–child tree formed within that column.
#[derive(Debug, Clone)]
pub struct BoardRow<'a> {
    pub ish: &'a Ish,
    /// For each ancestor (from root to this row's parent), `true` if that
    /// ancestor has more siblings after it. Length equals this row's depth
    /// in the column's tree (0 for top-level rows).
    pub ancestors_have_more: Vec<bool>,
    /// Whether this row is the last among its siblings.
    pub is_last: bool,
}

impl BoardRow<'_> {
    pub fn depth(&self) -> usize {
        self.ancestors_have_more.len()
    }
}

fn build_tree_rows<'a>(candidates: &[&'a Ish]) -> Vec<BoardRow<'a>> {
    if candidates.is_empty() {
        return Vec::new();
    }

    let candidate_ids: HashSet<&str> = candidates.iter().map(|ish| ish.id.as_str()).collect();

    // Group candidates by their effective parent within the column. An ish
    // whose parent is filtered out of this column is treated as a root.
    let mut children_by_parent: HashMap<Option<&'a str>, Vec<&'a Ish>> = HashMap::new();
    for ish in candidates {
        let parent_key = ish
            .parent
            .as_deref()
            .filter(|parent| candidate_ids.contains(parent));
        children_by_parent.entry(parent_key).or_default().push(*ish);
    }

    for siblings in children_by_parent.values_mut() {
        siblings.sort_by(|left, right| compare_ish(left, right));
    }

    let mut rows = Vec::with_capacity(candidates.len());
    let mut visited: HashSet<&str> = HashSet::new();

    append_subtree(
        None,
        &children_by_parent,
        &mut Vec::new(),
        &mut rows,
        &mut visited,
    );

    // Defensive: any ish that wasn't reached (e.g., a parent cycle leaves the
    // node detached from a real root) is appended as a top-level row, sorted
    // among themselves with the same comparator. Without this, those ishes
    // would silently disappear from the column.
    let mut orphans: Vec<&'a Ish> = candidates
        .iter()
        .filter(|ish| !visited.contains(ish.id.as_str()))
        .copied()
        .collect();
    if !orphans.is_empty() {
        orphans.sort_by(|left, right| compare_ish(left, right));
        let last = orphans.len() - 1;
        for (index, ish) in orphans.iter().enumerate() {
            rows.push(BoardRow {
                ish,
                ancestors_have_more: Vec::new(),
                is_last: index == last,
            });
            visited.insert(ish.id.as_str());
        }
    }

    rows
}

fn append_subtree<'a>(
    parent_id: Option<&'a str>,
    children_by_parent: &HashMap<Option<&'a str>, Vec<&'a Ish>>,
    ancestors_have_more: &mut Vec<bool>,
    rows: &mut Vec<BoardRow<'a>>,
    visited: &mut HashSet<&'a str>,
) {
    let Some(siblings) = children_by_parent.get(&parent_id) else {
        return;
    };

    let len = siblings.len();
    for (index, ish) in siblings.iter().enumerate() {
        if !visited.insert(ish.id.as_str()) {
            // Cycle protection: skip nodes already placed.
            continue;
        }

        let is_last = index + 1 == len;
        rows.push(BoardRow {
            ish,
            ancestors_have_more: ancestors_have_more.clone(),
            is_last,
        });

        ancestors_have_more.push(!is_last);
        append_subtree(
            Some(ish.id.as_str()),
            children_by_parent,
            ancestors_have_more,
            rows,
            visited,
        );
        ancestors_have_more.pop();
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
            .map(|row| row.ish.id.as_str())
            .collect::<Vec<_>>();

        assert_eq!(ids, vec!["ish-high-new", "ish-high-old", "ish-normal"]);
        assert!(model.bucket_for_status(Status::Completed).is_empty());
    }

    fn child_ish(
        id: &str,
        title: &str,
        status: &str,
        priority: Option<&str>,
        updated_at: (i32, u32, u32, u32, u32, u32),
        parent: Option<&str>,
    ) -> Ish {
        let mut ish = sample_ish(
            id,
            title,
            status,
            priority,
            updated_at,
            &format!("{id}--{}.md", title.to_ascii_lowercase().replace(' ', "-")),
        );
        ish.parent = parent.map(str::to_string);
        ish
    }

    #[test]
    fn bucket_for_status_groups_children_directly_below_parents() {
        let mut model = Model::new(Config::default());
        // Two top-level ishes: parent-a (high), parent-b (normal). Children of
        // parent-a are mixed priorities; one orphan whose parent is in another
        // status column should appear as a root.
        model.issues = vec![
            child_ish(
                "parent-a",
                "Parent A",
                Status::Todo.as_str(),
                Some(Priority::High.as_str()),
                (2026, 1, 1, 0, 0, 0),
                None,
            ),
            child_ish(
                "parent-b",
                "Parent B",
                Status::Todo.as_str(),
                None,
                (2026, 1, 5, 0, 0, 0),
                None,
            ),
            child_ish(
                "child-a-low",
                "Child A Low",
                Status::Todo.as_str(),
                Some(Priority::Low.as_str()),
                (2026, 1, 4, 0, 0, 0),
                Some("parent-a"),
            ),
            child_ish(
                "child-a-critical",
                "Child A Critical",
                Status::Todo.as_str(),
                Some(Priority::Critical.as_str()),
                (2026, 1, 2, 0, 0, 0),
                Some("parent-a"),
            ),
            child_ish(
                "grandchild",
                "Grandchild",
                Status::Todo.as_str(),
                Some(Priority::Normal.as_str()),
                (2026, 1, 3, 0, 0, 0),
                Some("child-a-critical"),
            ),
            child_ish(
                "orphan-todo",
                "Orphan",
                Status::Todo.as_str(),
                Some(Priority::High.as_str()),
                (2026, 1, 6, 0, 0, 0),
                Some("parent-elsewhere"),
            ),
            child_ish(
                "parent-elsewhere",
                "Parent Elsewhere",
                Status::InProgress.as_str(),
                Some(Priority::High.as_str()),
                (2026, 1, 1, 0, 0, 0),
                None,
            ),
        ];

        let bucket = model.bucket_for_status(Status::Todo);
        let summary = bucket
            .iter()
            .map(|row| (row.ish.id.as_str(), row.depth(), row.is_last))
            .collect::<Vec<_>>();

        // Top-level rows (parent-a, parent-b, orphan-todo treated as root)
        // are sorted by the existing comparator: priority asc → updated_at
        // desc. Children appear immediately after their parent, also sorted
        // with the same comparator at their level.
        assert_eq!(
            summary,
            vec![
                ("orphan-todo", 0, false),
                ("parent-a", 0, false),
                ("child-a-critical", 1, false),
                ("grandchild", 2, true),
                ("child-a-low", 1, true),
                ("parent-b", 0, true),
            ]
        );

        // ancestors_have_more is wired up so the connector renderer can draw
        // continuing vertical bars when an ancestor still has more siblings.
        // For the grandchild, both parent-a (followed by parent-b) and
        // child-a-critical (followed by child-a-low) still have siblings
        // pending, so both ancestor slots want a continuing bar.
        let grandchild = bucket
            .iter()
            .find(|row| row.ish.id == "grandchild")
            .expect("grandchild row");
        assert_eq!(grandchild.ancestors_have_more, vec![true, true]);
    }

    #[test]
    fn bucket_for_status_handles_parent_cycles_without_dropping_rows() {
        let mut model = Model::new(Config::default());
        model.issues = vec![
            child_ish(
                "loop-a",
                "Loop A",
                Status::Todo.as_str(),
                None,
                (2026, 1, 1, 0, 0, 0),
                Some("loop-b"),
            ),
            child_ish(
                "loop-b",
                "Loop B",
                Status::Todo.as_str(),
                None,
                (2026, 1, 2, 0, 0, 0),
                Some("loop-a"),
            ),
        ];

        let bucket = model.bucket_for_status(Status::Todo);
        let ids = bucket
            .iter()
            .map(|row| row.ish.id.as_str())
            .collect::<Vec<_>>();

        // Both members of the cycle still surface; depth falls back to 0.
        assert_eq!(ids.len(), 2);
        assert!(ids.contains(&"loop-a"));
        assert!(ids.contains(&"loop-b"));
    }
}

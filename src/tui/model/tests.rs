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

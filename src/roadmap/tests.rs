use super::{RoadmapOptions, build_roadmap, render_markdown};
use crate::config::Config;
use crate::model::ish::Ish;
use chrono::{TimeZone, Utc};

fn issue(id: &str, title: &str, status: &str, ish_type: &str, parent: Option<&str>) -> Ish {
    Ish {
        id: id.to_string(),
        slug: title.to_ascii_lowercase().replace(' ', "-"),
        path: format!("{id}.md"),
        title: title.to_string(),
        status: status.to_string(),
        ish_type: ish_type.to_string(),
        priority: Some("normal".to_string()),
        tags: Vec::new(),
        created_at: Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap(),
        updated_at: Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap(),
        order: None,
        body: format!("{title} body"),
        parent: parent.map(str::to_string),
        blocking: Vec::new(),
        blocked_by: Vec::new(),
    }
}

#[test]
fn build_roadmap_groups_items_under_milestones_epics_and_unscheduled() {
    let config = Config::default();
    let roadmap = build_roadmap(
        &config,
        &[
            issue("ish-m1", "Milestone", "todo", "milestone", None),
            issue("ish-e1", "Epic", "todo", "epic", Some("ish-m1")),
            issue("ish-f1", "Feature", "todo", "feature", Some("ish-e1")),
            issue("ish-t1", "Task", "todo", "task", Some("ish-m1")),
            issue("ish-e2", "Loose Epic", "todo", "epic", None),
            issue("ish-b1", "Loose Bug", "todo", "bug", Some("ish-e2")),
            issue("ish-t2", "Loose Task", "todo", "task", None),
        ],
        &RoadmapOptions::default(),
    );

    assert_eq!(roadmap.milestones.len(), 1);
    assert_eq!(roadmap.milestones[0].epics.len(), 1);
    assert_eq!(roadmap.milestones[0].epics[0].items[0].id, "ish-f1");
    assert_eq!(roadmap.milestones[0].items[0].id, "ish-t1");
    assert_eq!(roadmap.unscheduled.epics.len(), 1);
    assert_eq!(roadmap.unscheduled.epics[0].items[0].id, "ish-b1");
    assert_eq!(roadmap.unscheduled.items[0].id, "ish-t2");
}

#[test]
fn build_roadmap_excludes_done_items_by_default() {
    let config = Config::default();
    let roadmap = build_roadmap(
        &config,
        &[
            issue("ish-m1", "Milestone", "todo", "milestone", None),
            issue("ish-e1", "Epic", "completed", "epic", Some("ish-m1")),
            issue("ish-f1", "Feature", "completed", "feature", Some("ish-e1")),
        ],
        &RoadmapOptions::default(),
    );

    assert!(roadmap.milestones[0].epics.is_empty());
}

#[test]
fn build_roadmap_honors_include_done_and_status_filters() {
    let config = Config::default();
    let roadmap = build_roadmap(
        &config,
        &[
            issue("ish-m1", "Todo milestone", "todo", "milestone", None),
            issue("ish-m2", "Done milestone", "completed", "milestone", None),
            issue("ish-e1", "Done epic", "completed", "epic", Some("ish-m2")),
        ],
        &RoadmapOptions {
            include_done: true,
            status: vec!["completed".to_string()],
            ..RoadmapOptions::default()
        },
    );

    assert_eq!(roadmap.milestones.len(), 1);
    assert_eq!(roadmap.milestones[0].milestone.id, "ish-m2");
    assert_eq!(roadmap.milestones[0].epics[0].epic.id, "ish-e1");
}

#[test]
fn render_markdown_supports_link_options() {
    let config = Config::default();
    let roadmap = build_roadmap(
        &config,
        &[
            issue("ish-m1", "Milestone", "todo", "milestone", None),
            issue("ish-e1", "Epic", "todo", "epic", Some("ish-m1")),
            issue("ish-f1", "Feature", "todo", "feature", Some("ish-e1")),
        ],
        &RoadmapOptions::default(),
    );

    let linked = render_markdown(
        &config,
        &roadmap,
        &RoadmapOptions {
            link_prefix: Some("https://example.test/issues".to_string()),
            ..RoadmapOptions::default()
        },
    );
    assert!(linked.contains("[ish-m1](https://example.test/issues/ish-m1.md)"));

    let plain = render_markdown(
        &config,
        &roadmap,
        &RoadmapOptions {
            no_links: true,
            ..RoadmapOptions::default()
        },
    );
    assert!(plain.contains("Milestone: Milestone (ish-m1)"));
    assert!(!plain.contains("CLI"));
}

#[test]
fn roadmap_json_uses_nested_structure() {
    let config = Config::default();
    let roadmap = build_roadmap(
        &config,
        &[
            issue("ish-m1", "Milestone", "todo", "milestone", None),
            issue("ish-e1", "Epic", "todo", "epic", Some("ish-m1")),
            issue("ish-f1", "Feature", "todo", "feature", Some("ish-e1")),
        ],
        &RoadmapOptions::default(),
    );
    let json = roadmap.to_json();

    assert_eq!(json.milestones.len(), 1);
    assert_eq!(json.milestones[0].milestone.id, "ish-m1");
    assert_eq!(json.milestones[0].epics[0].epic.id, "ish-e1");
    assert_eq!(json.milestones[0].epics[0].items[0].id, "ish-f1");
}

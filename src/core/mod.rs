#[allow(dead_code)]
pub mod store;

use crate::model::ish::Ish;

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortMode {
    Default,
    Created,
    Updated,
    Status,
    Priority,
    Id,
}

#[allow(dead_code)]
pub fn sort_by_status_priority_and_type<'a>(
    ishes: &'a [Ish],
    status_names: &[&str],
    priority_names: &[&str],
    type_names: &[&str],
) -> Vec<&'a Ish> {
    let mut sorted = ishes.iter().collect::<Vec<_>>();
    sorted.sort_by(|left, right| {
        status_rank(status_names, &left.status)
            .cmp(&status_rank(status_names, &right.status))
            .then_with(|| compare_manual_order(left.order.as_deref(), right.order.as_deref()))
            .then_with(|| {
                priority_rank(priority_names, left.priority.as_deref())
                    .cmp(&priority_rank(priority_names, right.priority.as_deref()))
            })
            .then_with(|| {
                type_rank(type_names, &left.ish_type).cmp(&type_rank(type_names, &right.ish_type))
            })
            .then_with(|| {
                left.title
                    .to_ascii_lowercase()
                    .cmp(&right.title.to_ascii_lowercase())
            })
            .then_with(|| left.id.cmp(&right.id))
    });
    sorted
}

#[allow(dead_code)]
pub fn sort_ishes<'a>(
    ishes: &'a [Ish],
    sort_mode: SortMode,
    status_names: &[&str],
    priority_names: &[&str],
    type_names: &[&str],
) -> Vec<&'a Ish> {
    match sort_mode {
        SortMode::Default => {
            sort_by_status_priority_and_type(ishes, status_names, priority_names, type_names)
        }
        SortMode::Created => {
            let mut sorted = ishes.iter().collect::<Vec<_>>();
            sorted.sort_by(|left, right| {
                left.created_at
                    .cmp(&right.created_at)
                    .then_with(|| left.id.cmp(&right.id))
            });
            sorted
        }
        SortMode::Updated => {
            let mut sorted = ishes.iter().collect::<Vec<_>>();
            sorted.sort_by(|left, right| {
                left.updated_at
                    .cmp(&right.updated_at)
                    .then_with(|| left.id.cmp(&right.id))
            });
            sorted
        }
        SortMode::Status => {
            let mut sorted = ishes.iter().collect::<Vec<_>>();
            sorted.sort_by(|left, right| {
                status_rank(status_names, &left.status)
                    .cmp(&status_rank(status_names, &right.status))
                    .then_with(|| {
                        left.title
                            .to_ascii_lowercase()
                            .cmp(&right.title.to_ascii_lowercase())
                    })
                    .then_with(|| left.id.cmp(&right.id))
            });
            sorted
        }
        SortMode::Priority => {
            let mut sorted = ishes.iter().collect::<Vec<_>>();
            sorted.sort_by(|left, right| {
                priority_rank(priority_names, left.priority.as_deref())
                    .cmp(&priority_rank(priority_names, right.priority.as_deref()))
                    .then_with(|| {
                        left.title
                            .to_ascii_lowercase()
                            .cmp(&right.title.to_ascii_lowercase())
                    })
                    .then_with(|| left.id.cmp(&right.id))
            });
            sorted
        }
        SortMode::Id => {
            let mut sorted = ishes.iter().collect::<Vec<_>>();
            sorted.sort_by(|left, right| left.id.cmp(&right.id));
            sorted
        }
    }
}

/// Return ishes whose title, slug, or body contains the query.
#[allow(dead_code)]
pub fn search<'a>(ishes: &'a [Ish], query: &str) -> Vec<&'a Ish> {
    let needle = query.to_lowercase();

    ishes
        .iter()
        .filter(|ish| {
            ish.title.to_lowercase().contains(&needle)
                || ish.slug.to_lowercase().contains(&needle)
                || ish.body.to_lowercase().contains(&needle)
        })
        .collect()
}

#[allow(dead_code)]
fn compare_manual_order(left: Option<&str>, right: Option<&str>) -> std::cmp::Ordering {
    match (normalize_order(left), normalize_order(right)) {
        (Some(left), Some(right)) => left.cmp(right),
        (Some(_), None) => std::cmp::Ordering::Less,
        (None, Some(_)) => std::cmp::Ordering::Greater,
        (None, None) => std::cmp::Ordering::Equal,
    }
}

#[allow(dead_code)]
fn normalize_order(order: Option<&str>) -> Option<&str> {
    match order {
        Some(order) if !order.trim().is_empty() => Some(order),
        _ => None,
    }
}

#[allow(dead_code)]
fn status_rank(status_names: &[&str], status: &str) -> usize {
    ordered_rank(status_names, status)
}

#[allow(dead_code)]
fn type_rank(type_names: &[&str], ish_type: &str) -> usize {
    ordered_rank(type_names, ish_type)
}

#[allow(dead_code)]
fn priority_rank(priority_names: &[&str], priority: Option<&str>) -> usize {
    ordered_rank(priority_names, priority.unwrap_or("normal"))
}

#[allow(dead_code)]
fn ordered_rank(names: &[&str], value: &str) -> usize {
    names
        .iter()
        .position(|candidate| *candidate == value)
        .unwrap_or(names.len())
}

#[cfg(test)]
mod tests {
    use super::{SortMode, search, sort_by_status_priority_and_type, sort_ishes};
    use crate::model::ish::Ish;
    use chrono::{TimeZone, Utc};

    fn sample_ish(id: &str, title: &str, slug: &str, body: &str) -> Ish {
        Ish {
            id: id.to_string(),
            slug: slug.to_string(),
            path: format!("{id}--{slug}.md"),
            title: title.to_string(),
            status: "todo".to_string(),
            ish_type: "task".to_string(),
            priority: None,
            tags: Vec::new(),
            created_at: Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap(),
            updated_at: Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap(),
            order: None,
            body: body.to_string(),
            parent: None,
            blocking: Vec::new(),
            blocked_by: Vec::new(),
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn sortable_ish(
        id: &str,
        title: &str,
        status: &str,
        priority: Option<&str>,
        ish_type: &str,
        order: Option<&str>,
        created_at: (i32, u32, u32, u32, u32, u32),
        updated_at: (i32, u32, u32, u32, u32, u32),
    ) -> Ish {
        Ish {
            id: id.to_string(),
            slug: title.to_ascii_lowercase().replace(' ', "-"),
            path: format!("{id}.md"),
            title: title.to_string(),
            status: status.to_string(),
            ish_type: ish_type.to_string(),
            priority: priority.map(str::to_string),
            tags: Vec::new(),
            created_at: Utc
                .with_ymd_and_hms(
                    created_at.0,
                    created_at.1,
                    created_at.2,
                    created_at.3,
                    created_at.4,
                    created_at.5,
                )
                .unwrap(),
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
            order: order.map(str::to_string),
            body: String::new(),
            parent: None,
            blocking: Vec::new(),
            blocked_by: Vec::new(),
        }
    }

    fn sample_ishes() -> Vec<Ish> {
        vec![
            sample_ish(
                "ish-title",
                "Fix Widget Search",
                "fix-widget-search",
                "Track search regressions here.",
            ),
            sample_ish(
                "ish-body",
                "Add filter command",
                "add-filter-command",
                "Need a substring query for list output.",
            ),
            sample_ish(
                "ish-slug",
                "Polish terminal rendering",
                "search-rendering-tweaks",
                "No text match in the body.",
            ),
        ]
    }

    #[test]
    fn search_matches_title() {
        let ishes = sample_ishes();

        let matches = search(&ishes, "widget");

        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].id, "ish-title");
    }

    #[test]
    fn search_matches_body() {
        let ishes = sample_ishes();

        let matches = search(&ishes, "substring query");

        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].id, "ish-body");
    }

    #[test]
    fn search_matches_slug() {
        let ishes = sample_ishes();

        let matches = search(&ishes, "rendering-tweaks");

        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].id, "ish-slug");
    }

    #[test]
    fn search_is_case_insensitive() {
        let ishes = sample_ishes();

        let matches = search(&ishes, "SeArCh");

        assert_eq!(matches.len(), 2);
        assert_eq!(matches[0].id, "ish-title");
        assert_eq!(matches[1].id, "ish-slug");
    }

    #[test]
    fn search_returns_empty_when_nothing_matches() {
        let ishes = sample_ishes();

        let matches = search(&ishes, "milestone");

        assert!(matches.is_empty());
    }

    #[test]
    fn default_sort_orders_by_status_manual_order_priority_type_and_title() {
        let ishes = vec![
            sortable_ish(
                "ish-c",
                "gamma",
                "todo",
                Some("high"),
                "bug",
                Some("b"),
                (2026, 1, 3, 0, 0, 0),
                (2026, 1, 3, 0, 0, 0),
            ),
            sortable_ish(
                "ish-a",
                "Alpha",
                "in-progress",
                None,
                "task",
                None,
                (2026, 1, 1, 0, 0, 0),
                (2026, 1, 4, 0, 0, 0),
            ),
            sortable_ish(
                "ish-b",
                "beta",
                "todo",
                None,
                "task",
                Some("a"),
                (2026, 1, 2, 0, 0, 0),
                (2026, 1, 2, 0, 0, 0),
            ),
            sortable_ish(
                "ish-d",
                "delta",
                "todo",
                Some("low"),
                "feature",
                None,
                (2026, 1, 4, 0, 0, 0),
                (2026, 1, 1, 0, 0, 0),
            ),
        ];

        let sorted = sort_by_status_priority_and_type(
            &ishes,
            &["in-progress", "todo", "completed"],
            &["critical", "high", "normal", "low"],
            &["bug", "feature", "task"],
        );

        assert_eq!(ids(sorted), vec!["ish-a", "ish-b", "ish-c", "ish-d"]);
    }

    #[test]
    fn default_sort_places_unrecognized_values_last() {
        let ishes = vec![
            sortable_ish(
                "ish-known",
                "Known",
                "todo",
                Some("normal"),
                "task",
                None,
                (2026, 1, 1, 0, 0, 0),
                (2026, 1, 1, 0, 0, 0),
            ),
            sortable_ish(
                "ish-unknown-status",
                "Unknown status",
                "waiting",
                Some("normal"),
                "task",
                None,
                (2026, 1, 1, 0, 0, 0),
                (2026, 1, 1, 0, 0, 0),
            ),
            sortable_ish(
                "ish-unknown-priority",
                "Unknown priority",
                "todo",
                Some("urgent"),
                "task",
                None,
                (2026, 1, 1, 0, 0, 0),
                (2026, 1, 1, 0, 0, 0),
            ),
            sortable_ish(
                "ish-unknown-type",
                "Unknown type",
                "todo",
                Some("normal"),
                "chore",
                None,
                (2026, 1, 1, 0, 0, 0),
                (2026, 1, 1, 0, 0, 0),
            ),
        ];

        let sorted = sort_by_status_priority_and_type(&ishes, &["todo"], &["normal"], &["task"]);

        assert_eq!(
            ids(sorted),
            vec![
                "ish-known",
                "ish-unknown-type",
                "ish-unknown-priority",
                "ish-unknown-status"
            ]
        );
    }

    #[test]
    fn sort_modes_cover_created_updated_priority_status_and_id() {
        let ishes = vec![
            sortable_ish(
                "ish-c",
                "Charlie",
                "todo",
                Some("low"),
                "task",
                None,
                (2026, 1, 3, 0, 0, 0),
                (2026, 1, 1, 0, 0, 0),
            ),
            sortable_ish(
                "ish-a",
                "Alpha",
                "in-progress",
                Some("high"),
                "task",
                None,
                (2026, 1, 1, 0, 0, 0),
                (2026, 1, 3, 0, 0, 0),
            ),
            sortable_ish(
                "ish-b",
                "Bravo",
                "waiting",
                None,
                "task",
                None,
                (2026, 1, 2, 0, 0, 0),
                (2026, 1, 2, 0, 0, 0),
            ),
        ];

        assert_eq!(
            ids(sort_ishes(
                &ishes,
                SortMode::Created,
                &["in-progress", "todo"],
                &["critical", "high", "normal", "low"],
                &["task"],
            )),
            vec!["ish-a", "ish-b", "ish-c"]
        );
        assert_eq!(
            ids(sort_ishes(
                &ishes,
                SortMode::Updated,
                &["in-progress", "todo"],
                &["critical", "high", "normal", "low"],
                &["task"],
            )),
            vec!["ish-c", "ish-b", "ish-a"]
        );
        assert_eq!(
            ids(sort_ishes(
                &ishes,
                SortMode::Status,
                &["in-progress", "todo"],
                &["critical", "high", "normal", "low"],
                &["task"],
            )),
            vec!["ish-a", "ish-c", "ish-b"]
        );
        assert_eq!(
            ids(sort_ishes(
                &ishes,
                SortMode::Priority,
                &["in-progress", "todo"],
                &["critical", "high", "normal", "low"],
                &["task"],
            )),
            vec!["ish-a", "ish-b", "ish-c"]
        );
        assert_eq!(
            ids(sort_ishes(
                &ishes,
                SortMode::Id,
                &["in-progress", "todo"],
                &["critical", "high", "normal", "low"],
                &["task"],
            )),
            vec!["ish-a", "ish-b", "ish-c"]
        );
    }

    fn ids(ishes: Vec<&Ish>) -> Vec<&str> {
        ishes.into_iter().map(|ish| ish.id.as_str()).collect()
    }
}

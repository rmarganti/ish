use crate::config::Config;
use crate::core::store::Store;
use crate::model::ish::{Ish, IshJson};
use serde::Serialize;
use std::collections::{BTreeMap, HashMap};
use std::fmt::Write;
use std::path::Path;

const EXCERPT_LIMIT: usize = 160;

#[derive(Debug, Clone, Default)]
pub struct RoadmapOptions {
    pub include_done: bool,
    pub status: Vec<String>,
    pub no_status: Vec<String>,
    pub no_links: bool,
    pub link_prefix: Option<String>,
    pub json: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Roadmap {
    pub milestones: Vec<RoadmapMilestone>,
    pub unscheduled: RoadmapUnscheduled,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoadmapMilestone {
    pub milestone: Ish,
    pub epics: Vec<RoadmapEpic>,
    pub items: Vec<Ish>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoadmapEpic {
    pub epic: Ish,
    pub items: Vec<Ish>,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct RoadmapUnscheduled {
    pub epics: Vec<RoadmapEpic>,
    pub items: Vec<Ish>,
}

#[derive(Debug, Serialize)]
pub struct RoadmapJson {
    pub milestones: Vec<RoadmapMilestoneJson>,
    #[serde(skip_serializing_if = "RoadmapUnscheduledJson::is_empty")]
    pub unscheduled: RoadmapUnscheduledJson,
}

#[derive(Debug, Serialize)]
pub struct RoadmapMilestoneJson {
    pub milestone: IshJson,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub epics: Vec<RoadmapEpicJson>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub items: Vec<IshJson>,
}

#[derive(Debug, Serialize)]
pub struct RoadmapEpicJson {
    pub epic: IshJson,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub items: Vec<IshJson>,
}

#[derive(Debug, Serialize, Default)]
pub struct RoadmapUnscheduledJson {
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub epics: Vec<RoadmapEpicJson>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub items: Vec<IshJson>,
}

impl RoadmapUnscheduledJson {
    fn is_empty(&self) -> bool {
        self.epics.is_empty() && self.items.is_empty()
    }
}

pub fn roadmap_output(
    current_dir: &Path,
    options: &RoadmapOptions,
) -> Result<Option<String>, String> {
    let Some(config_path) = crate::config::find_config(current_dir) else {
        return Err("no `.ish.yml` found in the current directory or its parents".to_string());
    };

    let config = Config::load(&config_path)
        .map_err(|error| format!("failed to load `{}`: {error}", config_path.display()))?;
    let store_root = config_path
        .parent()
        .ok_or_else(|| format!("invalid config path: {}", config_path.display()))?
        .join(&config.ish.path);
    let mut store = Store::new(&store_root, config.clone())
        .map_err(|error| format!("failed to open store `{}`: {error}", store_root.display()))?;
    store
        .load()
        .map_err(|error| format!("failed to load store `{}`: {error}", store_root.display()))?;

    let ishes = store.all().into_iter().cloned().collect::<Vec<_>>();
    let roadmap = build_roadmap(&config, &ishes, options);

    if options.json {
        let json = serde_json::to_string_pretty(&roadmap.to_json())
            .map_err(|error| format!("failed to serialize roadmap JSON: {error}"))?;
        Ok(Some(json))
    } else {
        Ok(Some(render_markdown(&config, &roadmap, options)))
    }
}

pub fn build_roadmap(config: &Config, ishes: &[Ish], options: &RoadmapOptions) -> Roadmap {
    let visible = ishes
        .iter()
        .filter(|ish| options.include_done || !config.is_archive_status(&ish.status))
        .cloned()
        .collect::<Vec<_>>();
    let by_id = visible
        .iter()
        .cloned()
        .map(|ish| (ish.id.clone(), ish))
        .collect::<HashMap<_, _>>();

    let visible_milestones = visible
        .iter()
        .filter(|ish| ish.ish_type == "milestone")
        .filter(|ish| matches_status_filter(ish, options))
        .cloned()
        .collect::<Vec<_>>();
    let visible_milestone_ids = visible_milestones
        .iter()
        .map(|ish| ish.id.clone())
        .collect::<std::collections::HashSet<_>>();
    let visible_epics = visible
        .iter()
        .filter(|ish| ish.ish_type == "epic")
        .cloned()
        .collect::<Vec<_>>();
    let visible_epic_ids = visible_epics
        .iter()
        .map(|ish| ish.id.clone())
        .collect::<std::collections::HashSet<_>>();

    let mut epics_by_milestone: BTreeMap<String, Vec<Ish>> = BTreeMap::new();
    let mut direct_items_by_milestone: BTreeMap<String, Vec<Ish>> = BTreeMap::new();
    let mut items_by_epic: BTreeMap<String, Vec<Ish>> = BTreeMap::new();
    let mut unscheduled_epics = Vec::new();
    let mut unscheduled_items = Vec::new();

    for epic in &visible_epics {
        match nearest_milestone(epic, &by_id) {
            Some(milestone_id) if visible_milestone_ids.contains(&milestone_id) => {
                epics_by_milestone
                    .entry(milestone_id)
                    .or_default()
                    .push(epic.clone());
            }
            Some(_) => {}
            None => unscheduled_epics.push(epic.clone()),
        }
    }

    for item in visible
        .iter()
        .filter(|ish| ish.ish_type != "milestone" && ish.ish_type != "epic")
    {
        let milestone_id = nearest_milestone(item, &by_id);
        let epic_id = nearest_epic(item, &by_id);

        match milestone_id {
            Some(ref id) if visible_milestone_ids.contains(id) => {
                if let Some(epic_id) = epic_id {
                    if visible_epic_ids.contains(&epic_id) {
                        items_by_epic.entry(epic_id).or_default().push(item.clone());
                    }
                } else {
                    direct_items_by_milestone
                        .entry(id.clone())
                        .or_default()
                        .push(item.clone());
                }
            }
            Some(_) => {}
            None => {
                if let Some(epic_id) = epic_id {
                    if visible_epic_ids.contains(&epic_id) {
                        items_by_epic.entry(epic_id).or_default().push(item.clone());
                    }
                } else {
                    unscheduled_items.push(item.clone());
                }
            }
        }
    }

    let mut milestones = visible_milestones
        .into_iter()
        .map(|milestone| {
            let mut epics = epics_by_milestone.remove(&milestone.id).unwrap_or_default();
            sort_issues(config, &mut epics);

            let epics = epics
                .into_iter()
                .map(|epic| {
                    let mut items = items_by_epic.remove(&epic.id).unwrap_or_default();
                    sort_issues(config, &mut items);
                    RoadmapEpic { epic, items }
                })
                .collect::<Vec<_>>();

            let mut items = direct_items_by_milestone
                .remove(&milestone.id)
                .unwrap_or_default();
            sort_issues(config, &mut items);

            RoadmapMilestone {
                milestone,
                epics,
                items,
            }
        })
        .collect::<Vec<_>>();

    milestones.sort_by(|left, right| compare_issues(config, &left.milestone, &right.milestone));

    sort_issues(config, &mut unscheduled_epics);
    let unscheduled_epics = unscheduled_epics
        .into_iter()
        .map(|epic| {
            let mut items = items_by_epic.remove(&epic.id).unwrap_or_default();
            sort_issues(config, &mut items);
            RoadmapEpic { epic, items }
        })
        .collect::<Vec<_>>();
    sort_issues(config, &mut unscheduled_items);

    Roadmap {
        milestones,
        unscheduled: RoadmapUnscheduled {
            epics: unscheduled_epics,
            items: unscheduled_items,
        },
    }
}

pub fn render_markdown(config: &Config, roadmap: &Roadmap, options: &RoadmapOptions) -> String {
    let mut out = String::from("# Roadmap\n");

    for milestone in &roadmap.milestones {
        out.push('\n');
        let _ = writeln!(
            out,
            "## Milestone: {} ({})",
            milestone.milestone.title,
            issue_ref(config, &milestone.milestone, options)
        );
        out.push('\n');
        push_excerpt(&mut out, &milestone.milestone.body);
        push_items(config, &mut out, &milestone.items, options);

        for epic in &milestone.epics {
            out.push('\n');
            let _ = writeln!(
                out,
                "### Epic: {} ({})",
                epic.epic.title,
                issue_ref(config, &epic.epic, options)
            );
            out.push('\n');
            push_excerpt(&mut out, &epic.epic.body);
            push_items(config, &mut out, &epic.items, options);
        }
    }

    if !roadmap.unscheduled.epics.is_empty() || !roadmap.unscheduled.items.is_empty() {
        out.push_str("\n## Unscheduled\n\n");
        push_items(config, &mut out, &roadmap.unscheduled.items, options);

        for epic in &roadmap.unscheduled.epics {
            let _ = writeln!(
                out,
                "### Epic: {} ({})",
                epic.epic.title,
                issue_ref(config, &epic.epic, options)
            );
            out.push('\n');
            push_excerpt(&mut out, &epic.epic.body);
            push_items(config, &mut out, &epic.items, options);
        }
    }

    out.trim_end().to_string()
}

impl Roadmap {
    pub fn to_json(&self) -> RoadmapJson {
        RoadmapJson {
            milestones: self
                .milestones
                .iter()
                .map(|milestone| RoadmapMilestoneJson {
                    milestone: to_json_issue(&milestone.milestone),
                    epics: milestone
                        .epics
                        .iter()
                        .map(|epic| RoadmapEpicJson {
                            epic: to_json_issue(&epic.epic),
                            items: epic.items.iter().map(to_json_issue).collect(),
                        })
                        .collect(),
                    items: milestone.items.iter().map(to_json_issue).collect(),
                })
                .collect(),
            unscheduled: RoadmapUnscheduledJson {
                epics: self
                    .unscheduled
                    .epics
                    .iter()
                    .map(|epic| RoadmapEpicJson {
                        epic: to_json_issue(&epic.epic),
                        items: epic.items.iter().map(to_json_issue).collect(),
                    })
                    .collect(),
                items: self.unscheduled.items.iter().map(to_json_issue).collect(),
            },
        }
    }
}

fn to_json_issue(ish: &Ish) -> IshJson {
    ish.to_json(&ish.etag())
}

fn matches_status_filter(ish: &Ish, options: &RoadmapOptions) -> bool {
    let allowed =
        options.status.is_empty() || options.status.iter().any(|status| status == &ish.status);
    let excluded = options.no_status.iter().any(|status| status == &ish.status);
    allowed && !excluded
}

fn nearest_milestone(ish: &Ish, by_id: &HashMap<String, Ish>) -> Option<String> {
    walk_parents(ish, by_id)
        .find(|candidate| candidate.ish_type == "milestone")
        .map(|candidate| candidate.id.clone())
}

fn nearest_epic(ish: &Ish, by_id: &HashMap<String, Ish>) -> Option<String> {
    walk_parents(ish, by_id)
        .find(|candidate| candidate.ish_type == "epic")
        .map(|candidate| candidate.id.clone())
}

fn walk_parents<'a>(
    ish: &'a Ish,
    by_id: &'a HashMap<String, Ish>,
) -> impl Iterator<Item = &'a Ish> {
    let mut next_id = ish.parent.as_deref();
    std::iter::from_fn(move || {
        let current_id = next_id?;
        let parent = by_id.get(current_id)?;
        next_id = parent.parent.as_deref();
        Some(parent)
    })
}

fn compare_issues(config: &Config, left: &Ish, right: &Ish) -> std::cmp::Ordering {
    type_rank(config, &left.ish_type)
        .cmp(&type_rank(config, &right.ish_type))
        .then_with(|| status_rank(config, &left.status).cmp(&status_rank(config, &right.status)))
        .then_with(|| {
            left.title
                .to_ascii_lowercase()
                .cmp(&right.title.to_ascii_lowercase())
        })
        .then_with(|| left.id.cmp(&right.id))
}

fn sort_issues(config: &Config, issues: &mut [Ish]) {
    issues.sort_by(|left, right| compare_issues(config, left, right));
}

fn status_rank(config: &Config, status: &str) -> usize {
    config
        .status_names()
        .iter()
        .position(|candidate| *candidate == status)
        .unwrap_or(usize::MAX)
}

fn type_rank(config: &Config, ish_type: &str) -> usize {
    config
        .type_names()
        .iter()
        .position(|candidate| *candidate == ish_type)
        .unwrap_or(usize::MAX)
}

fn push_excerpt(out: &mut String, body: &str) {
    let excerpt = excerpt(body);
    if !excerpt.is_empty() {
        let _ = writeln!(out, "> {excerpt}");
        out.push('\n');
    }
}

fn push_items(config: &Config, out: &mut String, items: &[Ish], options: &RoadmapOptions) {
    if items.is_empty() {
        return;
    }

    for item in items {
        let _ = writeln!(
            out,
            "- {} {} ({})",
            type_badge(config, &item.ish_type),
            item.title,
            issue_ref(config, item, options)
        );
    }
}

fn excerpt(body: &str) -> String {
    let collapsed = body.split_whitespace().collect::<Vec<_>>().join(" ");
    if collapsed.len() <= EXCERPT_LIMIT {
        collapsed
    } else {
        format!("{}...", &collapsed[..EXCERPT_LIMIT].trim_end())
    }
}

fn type_badge(config: &Config, ish_type: &str) -> String {
    let color_name = config
        .get_type(ish_type)
        .map(|ish_type| ish_type.color)
        .unwrap_or("gray");
    let color = shield_color(color_name);
    format!("![{ish_type}](https://img.shields.io/badge/{ish_type}-{color}?style=flat-square)")
}

fn shield_color(color: &str) -> &'static str {
    match color {
        "red" => "d73a4a",
        "yellow" => "bf8700",
        "green" => "2da44e",
        "blue" => "1d76db",
        "purple" => "8250df",
        "cyan" => "1b7f83",
        "gray" => "6e7781",
        "white" => "57606a",
        _ => "6e7781",
    }
}

fn issue_ref(config: &Config, ish: &Ish, options: &RoadmapOptions) -> String {
    if options.no_links {
        return ish.id.clone();
    }

    format!("[{}]({})", ish.id, issue_link(config, ish, options))
}

fn issue_link(config: &Config, ish: &Ish, options: &RoadmapOptions) -> String {
    let base = options
        .link_prefix
        .as_deref()
        .unwrap_or(&config.ish.path)
        .trim_end_matches('/');

    if base.is_empty() {
        ish.path.clone()
    } else {
        format!("{base}/{}", ish.path)
    }
}

#[cfg(test)]
mod tests {
    use super::{build_roadmap, render_markdown, RoadmapOptions};
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
}

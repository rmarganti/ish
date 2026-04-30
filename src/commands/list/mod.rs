use crate::app::{AppContext, AppError, json_output_error};
use crate::cli::{ListArgs, ListSortArg};
use crate::core::SortMode;
use crate::model::ish::Ish;
use crate::output::{output_success_multiple, warning};

mod filters;
mod render;

use filters::{
    ArchiveVisibility, archive_visibility, filter_ishes, sort_ish_refs, validate_list_args,
};
use render::{list_json_value, render_tree_output};

pub(crate) fn list_command(args: ListArgs, json: bool) -> Result<Option<String>, AppError> {
    let context = AppContext::load()?;
    let config = context.config;
    let store = context.store;
    validate_list_args(&args, &config)?;

    let all_ishes = store.all().into_iter().cloned().collect::<Vec<_>>();
    let filtered = filter_ishes(&all_ishes, &store, &config, &args);
    let sort_mode = args
        .sort
        .map(ListSortArg::into_sort_mode)
        .unwrap_or(SortMode::Default);
    let sorted = sort_ish_refs(&filtered, sort_mode, &config);

    if json {
        let ishes = sorted
            .into_iter()
            .map(|ish| list_json_value(ish, args.full))
            .collect::<Result<Vec<_>, _>>()?;
        return Ok(Some(
            output_success_multiple(ishes).map_err(json_output_error)?,
        ));
    }

    if args.quiet {
        let output = sorted
            .iter()
            .map(|ish| ish.id.as_str())
            .collect::<Vec<_>>()
            .join("\n");
        return Ok((!output.is_empty()).then_some(output));
    }

    if sorted.is_empty() {
        return Ok(Some(warning("No ishes found")));
    }

    let tree_universe = tree_universe(&all_ishes, archive_visibility(&args));

    Ok(Some(render_tree_output(
        &sorted,
        &tree_universe,
        &store,
        &config,
        sort_mode,
    )))
}

fn tree_universe(all_ishes: &[Ish], visibility: ArchiveVisibility) -> Vec<&Ish> {
    all_ishes
        .iter()
        .filter(|ish| match visibility {
            ArchiveVisibility::ActiveOnly => !ish.is_archived(),
            ArchiveVisibility::ArchivedOnly => ish.is_archived(),
            ArchiveVisibility::All => true,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::list_command;
    use crate::app::run;
    use crate::cli::{Cli, Commands, ListArgs, ListSortArg};
    use crate::config::Config;
    use crate::test_support::{TestDir, WorkingDirGuard, write_test_ish};
    use serde_json::Value;
    use std::fs;

    fn base_list_args() -> ListArgs {
        ListArgs {
            status: Vec::new(),
            no_status: Vec::new(),
            ish_type: Vec::new(),
            no_type: Vec::new(),
            priority: Vec::new(),
            no_priority: Vec::new(),
            tag: Vec::new(),
            no_tag: Vec::new(),
            has_parent: false,
            no_parent: false,
            parent: None,
            has_blocking: false,
            no_blocking: false,
            is_blocked: false,
            ready: false,
            archived: false,
            all: false,
            search: None,
            sort: None,
            quiet: false,
            full: false,
        }
    }

    #[test]
    fn list_command_json_filters_and_omits_body_by_default() {
        let temp = TestDir::new();
        let config = Config::default();
        config.save(temp.path()).expect("config should save");
        let store_root = temp.path().join(".ish");
        fs::create_dir_all(&store_root).expect("store root should exist");
        write_test_ish(
            &store_root,
            "ish-alpha",
            "Alpha task",
            "todo",
            "task",
            Some("high"),
            "Matches search body.",
            None,
            &[],
            &[],
            &["cli"],
        );
        write_test_ish(
            &store_root,
            "ish-beta",
            "Beta bug",
            "todo",
            "bug",
            Some("normal"),
            "Other body.",
            None,
            &[],
            &[],
            &["backend"],
        );
        let _guard = WorkingDirGuard::change_to(temp.path());

        let output = list_command(
            ListArgs {
                status: vec!["todo".to_string()],
                ish_type: vec!["task".to_string()],
                priority: vec!["high".to_string()],
                tag: vec!["CLI".to_string()],
                search: Some("matches".to_string()),
                sort: Some(ListSortArg::Id),
                ..base_list_args()
            },
            true,
        )
        .expect("list command should succeed")
        .expect("list command should print output");

        let parsed: Value = serde_json::from_str(&output).expect("json should parse");
        assert_eq!(parsed[0]["id"], "ish-alpha");
        assert!(parsed[0].get("body").is_none());
    }

    #[test]
    fn list_command_full_json_includes_body() {
        let temp = TestDir::new();
        let config = Config::default();
        config.save(temp.path()).expect("config should save");
        let store_root = temp.path().join(".ish");
        fs::create_dir_all(&store_root).expect("store root should exist");
        write_test_ish(
            &store_root,
            "ish-alpha",
            "Alpha task",
            "todo",
            "task",
            Some("normal"),
            "Detailed body.",
            None,
            &[],
            &[],
            &[],
        );
        let _guard = WorkingDirGuard::change_to(temp.path());

        let output = list_command(
            ListArgs {
                sort: Some(ListSortArg::Id),
                full: true,
                ..base_list_args()
            },
            true,
        )
        .expect("list command should succeed")
        .expect("list command should print output");

        let parsed: Value = serde_json::from_str(&output).expect("json should parse");
        assert_eq!(parsed[0]["body"], "Detailed body.");
    }

    #[test]
    fn list_command_ready_excludes_blocked_in_progress_archived_and_implicitly_completed() {
        let temp = TestDir::new();
        let config = Config::default();
        config.save(temp.path()).expect("config should save");
        let store_root = temp.path().join(".ish");
        fs::create_dir_all(&store_root).expect("store root should exist");
        write_test_ish(
            &store_root,
            "ish-ready",
            "Ready item",
            "todo",
            "task",
            Some("normal"),
            "Body.",
            None,
            &[],
            &[],
            &[],
        );
        write_test_ish(
            &store_root,
            "ish-blocker",
            "Blocker",
            "in-progress",
            "task",
            Some("normal"),
            "Body.",
            None,
            &[],
            &[],
            &[],
        );
        write_test_ish(
            &store_root,
            "ish-blocked",
            "Blocked item",
            "todo",
            "task",
            Some("normal"),
            "Body.",
            None,
            &[],
            &["ish-blocker"],
            &[],
        );
        write_test_ish(
            &store_root,
            "ish-active",
            "Active item",
            "in-progress",
            "task",
            Some("normal"),
            "Body.",
            None,
            &[],
            &[],
            &[],
        );
        write_test_ish(
            &store_root,
            "ish-parent",
            "Completed parent",
            "completed",
            "feature",
            Some("normal"),
            "Body.",
            None,
            &[],
            &[],
            &[],
        );
        write_test_ish(
            &store_root,
            "ish-child",
            "Child item",
            "todo",
            "task",
            Some("normal"),
            "Body.",
            Some("ish-parent"),
            &[],
            &[],
            &[],
        );
        let _guard = WorkingDirGuard::change_to(temp.path());

        let output = list_command(
            ListArgs {
                ready: true,
                sort: Some(ListSortArg::Id),
                quiet: true,
                ..base_list_args()
            },
            false,
        )
        .expect("list command should succeed")
        .expect("list command should print output");

        assert_eq!(output.trim(), "ish-ready");
    }

    #[test]
    fn list_command_human_output_renders_tree_with_context_parent() {
        let temp = TestDir::new();
        let config = Config::default();
        config.save(temp.path()).expect("config should save");
        let store_root = temp.path().join(".ish");
        fs::create_dir_all(&store_root).expect("store root should exist");
        write_test_ish(
            &store_root,
            "ish-parent",
            "Parent",
            "todo",
            "feature",
            Some("normal"),
            "Parent body.",
            None,
            &[],
            &[],
            &[],
        );
        write_test_ish(
            &store_root,
            "ish-child",
            "Child",
            "todo",
            "task",
            Some("high"),
            "Child body.",
            Some("ish-parent"),
            &[],
            &[],
            &["cli"],
        );
        let _guard = WorkingDirGuard::change_to(temp.path());

        let output = list_command(
            ListArgs {
                tag: vec!["cli".to_string()],
                ..base_list_args()
            },
            false,
        )
        .expect("list command should succeed")
        .expect("list command should print output");

        assert!(output.contains("ish-parent"));
        assert!(output.contains("ish-child"));
        assert!(output.contains("└──"));
    }

    #[test]
    fn run_dispatches_list_command_through_app_layer() {
        let temp = TestDir::new();
        let config = Config::default();
        config.save(temp.path()).expect("config should save");
        fs::create_dir_all(temp.path().join(".ish")).expect("store dir should be created");
        write_test_ish(
            &temp.path().join(".ish"),
            "ish-abcd",
            "From run",
            "todo",
            "task",
            Some("normal"),
            "Body.",
            None,
            &[],
            &[],
            &[],
        );
        let _guard = WorkingDirGuard::change_to(temp.path());

        let output = run(Cli {
            json: true,
            command: Commands::List(base_list_args()),
        })
        .expect("run should succeed")
        .output
        .expect("list output should be present");

        let parsed: Value = serde_json::from_str(&output).expect("json should parse");
        assert_eq!(parsed[0]["id"], "ish-abcd");
    }

    #[test]
    fn list_command_archive_visibility_filters_and_tree_contexts() {
        let temp = TestDir::new();
        let config = Config::default();
        config.save(temp.path()).expect("config should save");
        let store_root = temp.path().join(".ish");
        let archive_root = store_root.join("archive");
        fs::create_dir_all(&archive_root).expect("archive root should exist");
        write_test_ish(
            &archive_root,
            "ish-parent",
            "Archived Parent",
            "completed",
            "feature",
            Some("normal"),
            "Parent body.",
            None,
            &[],
            &[],
            &[],
        );
        write_test_ish(
            &store_root,
            "ish-child",
            "Active Child",
            "todo",
            "task",
            Some("normal"),
            "Child body.",
            Some("ish-parent"),
            &[],
            &[],
            &["cli"],
        );
        write_test_ish(
            &store_root,
            "ish-active",
            "Active Root",
            "todo",
            "feature",
            Some("normal"),
            "Active body.",
            None,
            &[],
            &[],
            &[],
        );
        write_test_ish(
            &archive_root,
            "ish-archived-child",
            "Archived Child",
            "completed",
            "task",
            Some("normal"),
            "Archived child body.",
            Some("ish-active"),
            &[],
            &[],
            &["cli"],
        );
        let _guard = WorkingDirGuard::change_to(temp.path());

        let default_output = list_command(
            ListArgs {
                tag: vec!["cli".to_string()],
                quiet: false,
                ..base_list_args()
            },
            false,
        )
        .expect("default list should succeed")
        .expect("default list should print output");
        assert!(default_output.contains("ish-child"));
        assert!(!default_output.contains("ish-parent"));

        let archived_output = list_command(
            ListArgs {
                archived: true,
                ..base_list_args()
            },
            false,
        )
        .expect("archived list should succeed")
        .expect("archived list should print output");
        assert!(archived_output.contains("ish-parent"));
        assert!(archived_output.contains("ish-archived-child"));
        assert!(!archived_output.contains("ish-active"));

        let all_output = list_command(
            ListArgs {
                tag: vec!["cli".to_string()],
                all: true,
                ..base_list_args()
            },
            false,
        )
        .expect("all list should succeed")
        .expect("all list should print output");
        assert!(all_output.contains("ish-parent"));
        assert!(all_output.contains("ish-child"));
        assert!(all_output.contains("ish-active"));
        assert!(all_output.contains("ish-archived-child"));
        assert!(all_output.contains("└──"));

        let filtered_archived = list_command(
            ListArgs {
                status: vec!["completed".to_string()],
                ..base_list_args()
            },
            true,
        )
        .expect("default filtered list should succeed")
        .expect("default filtered list should print output");
        let parsed: Value = serde_json::from_str(&filtered_archived).expect("json should parse");
        assert_eq!(parsed, Value::Array(Vec::new()));
    }

    #[test]
    fn list_command_ready_excludes_archived_even_with_all() {
        let temp = TestDir::new();
        let config = Config::default();
        config.save(temp.path()).expect("config should save");
        let store_root = temp.path().join(".ish");
        let archive_root = store_root.join("archive");
        fs::create_dir_all(&archive_root).expect("archive root should exist");
        write_test_ish(
            &store_root,
            "ish-ready",
            "Ready item",
            "todo",
            "task",
            Some("normal"),
            "Body.",
            None,
            &[],
            &[],
            &[],
        );
        write_test_ish(
            &archive_root,
            "ish-archived-ready",
            "Archived ready item",
            "todo",
            "task",
            Some("normal"),
            "Body.",
            None,
            &[],
            &[],
            &[],
        );
        let _guard = WorkingDirGuard::change_to(temp.path());

        let output = list_command(
            ListArgs {
                ready: true,
                all: true,
                quiet: true,
                ..base_list_args()
            },
            false,
        )
        .expect("ready list should succeed")
        .expect("ready list should print output");

        assert_eq!(output.trim(), "ish-ready");
    }
}

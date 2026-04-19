use crate::app::{AppContext, AppError, json_output_error};
use crate::cli::{ListArgs, ListSortArg};
use crate::core::SortMode;
use crate::output::{output_success_multiple, warning};

mod filters;
mod render;

use filters::{filter_ishes, sort_ish_refs, validate_list_args};
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

    Ok(Some(render_tree_output(
        &sorted, &all_ishes, &store, &config, sort_mode,
    )))
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
                no_status: Vec::new(),
                ish_type: vec!["task".to_string()],
                no_type: Vec::new(),
                priority: vec!["high".to_string()],
                no_priority: Vec::new(),
                tag: vec!["CLI".to_string()],
                no_tag: Vec::new(),
                has_parent: false,
                no_parent: false,
                parent: None,
                has_blocking: false,
                no_blocking: false,
                is_blocked: false,
                ready: false,
                search: Some("matches".to_string()),
                sort: Some(ListSortArg::Id),
                quiet: false,
                full: false,
            },
            true,
        )
        .expect("list command should succeed")
        .expect("list command should print output");

        let parsed: Value = serde_json::from_str(&output).expect("json should parse");
        assert_eq!(parsed["success"], Value::Bool(true));
        assert_eq!(parsed["count"], Value::from(1));
        assert_eq!(parsed["ishes"][0]["id"], "ish-alpha");
        assert!(parsed["ishes"][0].get("body").is_none());
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
                search: None,
                sort: Some(ListSortArg::Id),
                quiet: false,
                full: true,
            },
            true,
        )
        .expect("list command should succeed")
        .expect("list command should print output");

        let parsed: Value = serde_json::from_str(&output).expect("json should parse");
        assert_eq!(parsed["ishes"][0]["body"], "Detailed body.");
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
                ready: true,
                search: None,
                sort: Some(ListSortArg::Id),
                quiet: true,
                full: false,
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
                status: Vec::new(),
                no_status: Vec::new(),
                ish_type: Vec::new(),
                no_type: Vec::new(),
                priority: Vec::new(),
                no_priority: Vec::new(),
                tag: vec!["cli".to_string()],
                no_tag: Vec::new(),
                has_parent: false,
                no_parent: false,
                parent: None,
                has_blocking: false,
                no_blocking: false,
                is_blocked: false,
                ready: false,
                search: None,
                sort: None,
                quiet: false,
                full: false,
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
            command: Commands::List(ListArgs {
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
                search: None,
                sort: None,
                quiet: false,
                full: false,
            }),
        })
        .expect("run should succeed")
        .output
        .expect("list output should be present");

        let parsed: Value = serde_json::from_str(&output).expect("json should parse");
        assert_eq!(parsed["success"], Value::Bool(true));
        assert_eq!(parsed["count"], Value::from(1));
        assert_eq!(parsed["ishes"][0]["id"], "ish-abcd");
    }
}

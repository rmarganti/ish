use crate::app::{AppContext, AppError, RunOutcome, json_output_error};
use crate::cli::CheckArgs;
use std::process::ExitCode;

mod config;
mod render;

use config::{link_issue_count, validate_config};
use render::{render_check_human, render_check_json};

pub(crate) fn check_command(args: CheckArgs, json: bool) -> Result<RunOutcome, AppError> {
    let context = AppContext::load()?;
    let config = context.config;
    let mut store = context.store;
    let config_checks = validate_config(&config);
    let initial_links = store.check_all_links();
    let issues_found = config_checks.issue_count() + link_issue_count(&initial_links);

    let fixed_links = if args.fix {
        Some(store.fix_broken_links().map_err(|error| {
            AppError::new(
                crate::output::ErrorCode::FileError,
                format!("failed to fix broken links: {error}"),
            )
        })?)
    } else {
        None
    };
    let final_links = if args.fix {
        store.check_all_links()
    } else {
        initial_links.clone()
    };

    let output = if json {
        render_check_json(&config_checks, &initial_links, &final_links, fixed_links)
            .map_err(json_output_error)?
    } else {
        render_check_human(&config_checks, &initial_links, &final_links, fixed_links)
    };

    Ok(RunOutcome {
        output: Some(output),
        exit_code: if issues_found == 0 {
            ExitCode::SUCCESS
        } else {
            ExitCode::FAILURE
        },
    })
}

#[cfg(test)]
mod tests {
    use super::check_command;
    use crate::app::run;
    use crate::cli::{CheckArgs, Cli, Commands};
    use crate::config::Config;
    use crate::test_support::{TestDir, WorkingDirGuard};
    use serde_json::Value;
    use std::fs;
    use std::process::ExitCode;

    #[test]
    fn check_command_reports_link_issues_and_returns_failure_exit_code() {
        let temp = TestDir::new();
        let mut config = Config::default();
        config.project.name = "Check Test".to_string();
        config.save(temp.path()).expect("config should save");
        let store_root = temp.path().join(".ish");
        fs::create_dir_all(&store_root).expect("store root should exist");
        fs::write(
            store_root.join("ish-a--a.md"),
            "---\n# ish-a\ntitle: A\nstatus: todo\ntype: task\ncreated_at: 2026-01-01T00:00:00Z\nupdated_at: 2026-01-01T00:00:00Z\nblocking:\n  - ish-missing\n---\n\nA body.\n",
        )
        .expect("issue file should exist");
        let _guard = WorkingDirGuard::change_to(temp.path());

        let outcome =
            check_command(CheckArgs { fix: false }, false).expect("check command should succeed");
        let output = outcome.output.expect("check command should print output");

        assert_eq!(outcome.exit_code, ExitCode::FAILURE);
        assert!(output.contains("✗ broken links"));
        assert!(output.contains("ish-a blocking ish-missing"));
        assert!(output.contains("Summary: 1 issue(s) found"));
    }

    #[test]
    fn check_command_fix_removes_broken_links_but_still_reports_failure() {
        let temp = TestDir::new();
        let mut config = Config::default();
        config.project.name = "Check Fix Test".to_string();
        config.save(temp.path()).expect("config should save");
        let store_root = temp.path().join(".ish");
        fs::create_dir_all(&store_root).expect("store root should exist");
        fs::write(
            store_root.join("ish-a--a.md"),
            "---\n# ish-a\ntitle: A\nstatus: todo\ntype: task\ncreated_at: 2026-01-01T00:00:00Z\nupdated_at: 2026-01-01T00:00:00Z\nblocking:\n  - ish-a\n  - ish-missing\n---\n\nA body.\n",
        )
        .expect("issue file should exist");
        let _guard = WorkingDirGuard::change_to(temp.path());

        let outcome =
            check_command(CheckArgs { fix: true }, false).expect("check command should succeed");
        let output = outcome.output.expect("check command should print output");
        let contents = fs::read_to_string(store_root.join("ish-a--a.md"))
            .expect("updated ish should be readable");

        assert_eq!(outcome.exit_code, ExitCode::FAILURE);
        assert!(output.contains("Applied --fix to 2 link(s)"));
        assert!(output.contains("✓ remaining broken links: none"));
        assert!(!contents.contains("ish-missing"));
        assert!(!contents.contains("- ish-a"));
    }

    #[test]
    fn check_command_wraps_results_in_json_mode() {
        let temp = TestDir::new();
        let mut config = Config::default();
        config.project.name = "Check JSON Test".to_string();
        config.save(temp.path()).expect("config should save");
        let store_root = temp.path().join(".ish");
        fs::create_dir_all(&store_root).expect("store root should exist");
        fs::write(
            store_root.join("ish-a--a.md"),
            "---\n# ish-a\ntitle: A\nstatus: todo\ntype: task\ncreated_at: 2026-01-01T00:00:00Z\nupdated_at: 2026-01-01T00:00:00Z\nblocked_by:\n  - ish-missing\n---\n\nA body.\n",
        )
        .expect("issue file should exist");
        let _guard = WorkingDirGuard::change_to(temp.path());

        let outcome = run(Cli {
            json: true,
            command: Some(Commands::Check(CheckArgs { fix: false })),
        })
        .expect("check command should succeed");
        let output = outcome.output.expect("check command should print output");
        let parsed: Value = serde_json::from_str(&output).expect("json should parse");

        assert_eq!(outcome.exit_code, ExitCode::FAILURE);
        assert_eq!(parsed["success"], Value::Bool(true));
        assert_eq!(parsed["data"]["summary"]["issues_found"], Value::from(1));
        assert_eq!(
            parsed["data"]["checks"]["links"]["initial"]["broken_links"][0]["link_type"],
            Value::String("blocked_by".to_string())
        );
    }

    #[test]
    fn check_command_returns_success_when_workspace_is_clean() {
        let temp = TestDir::new();
        let mut config = Config::default();
        config.project.name = "Check Clean Test".to_string();
        config.save(temp.path()).expect("config should save");
        let store_root = temp.path().join(".ish");
        fs::create_dir_all(&store_root).expect("store root should exist");
        fs::write(
            store_root.join("ish-a--a.md"),
            "---\n# ish-a\ntitle: A\nstatus: todo\ntype: task\ncreated_at: 2026-01-01T00:00:00Z\nupdated_at: 2026-01-01T00:00:00Z\n---\n\nA body.\n",
        )
        .expect("clean file should exist");
        let _guard = WorkingDirGuard::change_to(temp.path());

        let outcome =
            check_command(CheckArgs { fix: false }, false).expect("check command should succeed");
        let output = outcome.output.expect("check command should print output");

        assert_eq!(outcome.exit_code, ExitCode::SUCCESS);
        assert!(output.contains("Summary: no issues found"));
    }
}

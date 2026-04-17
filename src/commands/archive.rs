use crate::app::{AppContext, AppError, json_output_error};
use crate::output::{ErrorCode, output_success, success, warning};
use serde_json::json;

pub(crate) fn archive_command(json: bool) -> Result<Option<String>, AppError> {
    let mut store = AppContext::load()?.store;
    let archived = store.archive_all_completed().map_err(|error| {
        AppError::new(
            ErrorCode::FileError,
            format!("failed to archive completed ishoos: {error}"),
        )
    })?;

    if json {
        return Ok(Some(
            output_success(json!({ "archived": archived })).map_err(json_output_error)?,
        ));
    }

    let message = if archived == 0 {
        "no completed or scrapped ishoos to archive".to_string()
    } else if archived == 1 {
        "archived 1 ishoo".to_string()
    } else {
        format!("archived {archived} ishoos")
    };

    if archived == 0 {
        Ok(Some(warning(&message)))
    } else {
        Ok(Some(success(&message)))
    }
}

#[cfg(test)]
mod tests {
    use super::archive_command;
    use crate::app::run;
    use crate::cli::{Cli, Commands};
    use crate::config::Config;
    use crate::test_support::{TestDir, WorkingDirGuard};
    use serde_json::Value;
    use std::fs;

    #[test]
    fn run_archive_moves_completed_ishoos() {
        let temp = TestDir::new();
        let mut config = Config::default();
        config.project.name = "Archive Test".to_string();
        config.save(temp.path()).expect("config should save");
        let store_root = temp.path().join(".ish");
        fs::create_dir_all(&store_root).expect("store root should exist");
        fs::write(
            store_root.join("ish-done--completed.md"),
            "---\n# ish-done\ntitle: Done\nstatus: completed\ntype: task\ncreated_at: 2026-01-01T00:00:00Z\nupdated_at: 2026-01-01T00:00:00Z\n---\n\nDone body.\n",
        )
        .expect("completed file should exist");
        fs::write(
            store_root.join("ish-todo--todo.md"),
            "---\n# ish-todo\ntitle: Todo\nstatus: todo\ntype: task\ncreated_at: 2026-01-01T00:00:00Z\nupdated_at: 2026-01-01T00:00:00Z\n---\n\nTodo body.\n",
        )
        .expect("todo file should exist");
        let _guard = WorkingDirGuard::change_to(temp.path());

        let output = run(Cli {
            json: false,
            command: Some(Commands::Archive),
        })
        .expect("archive command should succeed")
        .output
        .expect("archive command should print output");

        assert!(output.contains("archived 1 ishoo"));
        assert!(store_root.join("archive/ish-done--completed.md").exists());
        assert!(!store_root.join("ish-done--completed.md").exists());
        assert!(store_root.join("ish-todo--todo.md").exists());
    }

    #[test]
    fn archive_command_returns_noop_message_when_nothing_matches() {
        let temp = TestDir::new();
        let mut config = Config::default();
        config.project.name = "Archive Empty Test".to_string();
        config.save(temp.path()).expect("config should save");
        let store_root = temp.path().join(".ish");
        fs::create_dir_all(&store_root).expect("store root should exist");
        fs::write(
            store_root.join("ish-todo--todo.md"),
            "---\n# ish-todo\ntitle: Todo\nstatus: todo\ntype: task\ncreated_at: 2026-01-01T00:00:00Z\nupdated_at: 2026-01-01T00:00:00Z\n---\n\nTodo body.\n",
        )
        .expect("todo file should exist");
        let _guard = WorkingDirGuard::change_to(temp.path());

        let output = archive_command(false)
            .expect("archive command should succeed")
            .expect("archive command should print output");

        assert!(output.contains("no completed or scrapped ishoos to archive"));
        assert!(!store_root.join("archive").exists());
    }

    #[test]
    fn archive_command_wraps_archived_count_in_json_mode() {
        let temp = TestDir::new();
        let mut config = Config::default();
        config.project.name = "Archive JSON Test".to_string();
        config.save(temp.path()).expect("config should save");
        let store_root = temp.path().join(".ish");
        fs::create_dir_all(&store_root).expect("store root should exist");
        fs::write(
            store_root.join("ish-nope--scrapped.md"),
            "---\n# ish-nope\ntitle: Nope\nstatus: scrapped\ntype: task\ncreated_at: 2026-01-01T00:00:00Z\nupdated_at: 2026-01-01T00:00:00Z\n---\n\nNope body.\n",
        )
        .expect("scrapped file should exist");
        let _guard = WorkingDirGuard::change_to(temp.path());

        let output = archive_command(true)
            .expect("archive command should succeed")
            .expect("archive command should print output");

        let parsed: Value = serde_json::from_str(&output).expect("json should parse");
        assert_eq!(parsed["success"], Value::Bool(true));
        assert_eq!(parsed["data"]["archived"], Value::from(1));
    }
}

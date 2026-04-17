use crate::app::{AppContext, AppError, json_output_error, store_app_error};
use crate::cli::UpdateArgs;
use crate::core::store::UpdateIshoo;
use crate::output::{ErrorCode, muted, output_success, render_id, success};
use std::fs;
use std::io::{self, Read};

pub(crate) fn update_command(args: UpdateArgs, json: bool) -> Result<Option<String>, AppError> {
    let changes = resolve_update_changes(args)?;
    let mut store = AppContext::load()?.store;
    let should_unarchive = store.is_archived(&changes.0).map_err(store_app_error)?;

    if should_unarchive {
        store
            .load_and_unarchive(&changes.0)
            .map_err(store_app_error)?;
    }

    let updated = store
        .update(&changes.0, changes.1)
        .map_err(store_app_error)?;

    if json {
        return Ok(Some(
            output_success(updated.to_json(&updated.etag())).map_err(json_output_error)?,
        ));
    }

    Ok(Some(success(&format!(
        "Updated {} {}",
        render_id(&updated.id),
        muted(&updated.path)
    ))))
}

pub(crate) fn resolve_update_changes(args: UpdateArgs) -> Result<(String, UpdateIshoo), AppError> {
    let body = resolve_optional_body(args.body, args.body_file)?;
    let body_append = resolve_optional_stdin_text(args.body_append, "body append")?;
    let body_replace = args.body_replace_old.zip(args.body_replace_new);
    let parent = if args.remove_parent {
        Some(None)
    } else {
        args.parent.map(Some)
    };
    let priority = args.priority.map(|priority| {
        if priority.eq_ignore_ascii_case("none") {
            None
        } else {
            Some(priority)
        }
    });

    let has_changes = args.status.is_some()
        || args.ishoo_type.is_some()
        || priority.is_some()
        || args.title.is_some()
        || body.is_some()
        || body_replace.is_some()
        || body_append.is_some()
        || !args.tags.is_empty()
        || !args.remove_tags.is_empty()
        || parent.is_some()
        || !args.blocking.is_empty()
        || !args.remove_blocking.is_empty()
        || !args.blocked_by.is_empty()
        || !args.remove_blocked_by.is_empty();

    if !has_changes {
        return Err(AppError::new(ErrorCode::Validation, "no changes specified"));
    }

    Ok((
        args.id,
        UpdateIshoo {
            status: args.status,
            ishoo_type: args.ishoo_type,
            priority,
            title: args.title,
            body,
            body_replace,
            body_append,
            add_tags: args.tags,
            remove_tags: args.remove_tags,
            parent,
            add_blocking: args.blocking,
            remove_blocking: args.remove_blocking,
            add_blocked_by: args.blocked_by,
            remove_blocked_by: args.remove_blocked_by,
            if_match: args.if_match,
        },
    ))
}

fn resolve_optional_body(
    body: Option<String>,
    body_file: Option<String>,
) -> Result<Option<String>, AppError> {
    match (body, body_file) {
        (Some(body), None) => Ok(Some(read_stdin_or_literal(body, "body")?)),
        (None, Some(path)) => Ok(Some(fs::read_to_string(&path).map_err(|error| {
            AppError::new(
                ErrorCode::FileError,
                format!("failed to read body file `{path}`: {error}"),
            )
        })?)),
        (None, None) => Ok(None),
        (Some(_), Some(_)) => Err(AppError::new(
            ErrorCode::Validation,
            "`--body` and `--body-file` cannot be used together",
        )),
    }
}

fn resolve_optional_stdin_text(
    value: Option<String>,
    label: &str,
) -> Result<Option<String>, AppError> {
    value
        .map(|text| read_stdin_or_literal(text, label))
        .transpose()
}

fn read_stdin_or_literal(value: String, label: &str) -> Result<String, AppError> {
    if value != "-" {
        return Ok(value);
    }

    read_text_input(io::stdin(), label)
}

pub(crate) fn read_text_input<R: Read>(mut reader: R, label: &str) -> Result<String, AppError> {
    let mut stdin = String::new();
    reader.read_to_string(&mut stdin).map_err(|error| {
        AppError::new(
            ErrorCode::FileError,
            format!("failed to read {label} from stdin: {error}"),
        )
    })?;
    Ok(stdin)
}

#[cfg(test)]
mod tests {
    use super::{read_text_input, resolve_update_changes, update_command};
    use crate::app::{AppContext, AppError};
    use crate::cli::UpdateArgs;
    use crate::config::Config;
    use crate::output::ErrorCode;
    use crate::test_support::{TestDir, WorkingDirGuard, write_test_ishoo};
    use serde_json::Value;
    use std::fs;
    use std::io::Cursor;
    use std::path::PathBuf;

    fn load_store_from_current_dir()
    -> Result<(PathBuf, crate::config::Config, crate::core::store::Store), AppError> {
        let context = AppContext::load()?;
        Ok((context.current_dir, context.config, context.store))
    }

    #[test]
    fn update_command_applies_field_changes_and_returns_json() {
        let temp = TestDir::new();
        let config = Config::default();
        config.save(temp.path()).expect("config should save");
        let store_root = temp.path().join(".ish");
        fs::create_dir_all(&store_root).expect("store root should exist");
        write_test_ishoo(
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
        write_test_ishoo(
            &store_root,
            "ish-blocker",
            "Blocker",
            "todo",
            "task",
            Some("normal"),
            "Blocker body.",
            None,
            &[],
            &[],
            &[],
        );
        write_test_ishoo(
            &store_root,
            "ish-dependency",
            "Dependency",
            "todo",
            "task",
            Some("normal"),
            "Dependency body.",
            None,
            &[],
            &[],
            &[],
        );
        write_test_ishoo(
            &store_root,
            "ish-target",
            "Original title",
            "todo",
            "task",
            Some("normal"),
            "Alpha target",
            None,
            &[],
            &[],
            &["old-tag"],
        );
        let _guard = WorkingDirGuard::change_to(temp.path());

        let output = update_command(
            UpdateArgs {
                id: "target".to_string(),
                status: Some("in-progress".to_string()),
                ishoo_type: Some("bug".to_string()),
                priority: Some("high".to_string()),
                title: Some("Updated title".to_string()),
                body: None,
                body_file: None,
                body_replace_old: Some("target".to_string()),
                body_replace_new: Some("replacement".to_string()),
                body_append: Some("Appended text".to_string()),
                parent: Some("parent".to_string()),
                remove_parent: false,
                blocking: vec!["blocker".to_string()],
                remove_blocking: Vec::new(),
                blocked_by: vec!["dependency".to_string()],
                remove_blocked_by: Vec::new(),
                tags: vec!["new-tag".to_string()],
                remove_tags: vec!["old-tag".to_string()],
                if_match: None,
            },
            true,
        )
        .expect("update command should succeed")
        .expect("update command should print output");

        let parsed: Value = serde_json::from_str(&output).expect("json should parse");
        assert_eq!(parsed["success"], Value::Bool(true));
        assert_eq!(
            parsed["data"]["id"],
            Value::String("ish-target".to_string())
        );
        assert_eq!(
            parsed["data"]["status"],
            Value::String("in-progress".to_string())
        );
        assert_eq!(parsed["data"]["type"], Value::String("bug".to_string()));
        assert_eq!(
            parsed["data"]["priority"],
            Value::String("high".to_string())
        );
        assert_eq!(
            parsed["data"]["title"],
            Value::String("Updated title".to_string())
        );
        assert_eq!(
            parsed["data"]["body"],
            Value::String("Alpha replacement\n\nAppended text".to_string())
        );
        assert_eq!(
            parsed["data"]["parent"],
            Value::String("ish-parent".to_string())
        );
        assert_eq!(
            parsed["data"]["blocking"][0],
            Value::String("ish-blocker".to_string())
        );
        assert_eq!(
            parsed["data"]["blocked_by"][0],
            Value::String("ish-dependency".to_string())
        );
        assert_eq!(
            parsed["data"]["tags"][0],
            Value::String("new-tag".to_string())
        );
        assert!(store_root.join("ish-target--updated-title.md").exists());
        assert!(!store_root.join("ish-target--original-title.md").exists());
    }

    #[test]
    fn update_command_supports_body_append_priority_none_and_relation_removals() {
        let temp = TestDir::new();
        let config = Config::default();
        config.save(temp.path()).expect("config should save");
        let store_root = temp.path().join(".ish");
        fs::create_dir_all(&store_root).expect("store root should exist");
        write_test_ishoo(
            &store_root,
            "ish-target",
            "Target",
            "todo",
            "task",
            Some("normal"),
            "Body.",
            Some("ish-parent"),
            &["ish-blocker"],
            &["ish-dependency"],
            &["cli"],
        );
        write_test_ishoo(
            &store_root,
            "ish-parent",
            "Parent",
            "todo",
            "feature",
            Some("normal"),
            "Parent.",
            None,
            &[],
            &[],
            &[],
        );
        let _guard = WorkingDirGuard::change_to(temp.path());

        let changes = resolve_update_changes(UpdateArgs {
            id: "target".to_string(),
            status: None,
            ishoo_type: None,
            priority: Some("none".to_string()),
            title: None,
            body: None,
            body_file: None,
            body_replace_old: None,
            body_replace_new: None,
            body_append: Some("From stdin".to_string()),
            parent: None,
            remove_parent: true,
            blocking: Vec::new(),
            remove_blocking: vec!["blocker".to_string()],
            blocked_by: Vec::new(),
            remove_blocked_by: vec!["dependency".to_string()],
            tags: Vec::new(),
            remove_tags: vec!["cli".to_string()],
            if_match: None,
        })
        .expect("changes should resolve");

        let (_, _, mut store) = load_store_from_current_dir().expect("store should load");
        let updated = store
            .update(&changes.0, changes.1)
            .expect("store update should succeed");

        assert_eq!(updated.priority, None);
        assert_eq!(updated.parent, None);
        assert!(updated.blocking.is_empty());
        assert!(updated.blocked_by.is_empty());
        assert!(updated.tags.is_empty());
        assert_eq!(updated.body, "Body.\n\nFrom stdin");
    }

    #[test]
    fn read_text_input_reads_body_append_from_stdin() {
        let value = read_text_input(Cursor::new(b"stdin body\n".to_vec()), "body append")
            .expect("stdin text should be read");

        assert_eq!(value, "stdin body\n");
    }

    #[test]
    fn update_command_rejects_when_no_changes_specified() {
        let error = resolve_update_changes(UpdateArgs {
            id: "target".to_string(),
            status: None,
            ishoo_type: None,
            priority: None,
            title: None,
            body: None,
            body_file: None,
            body_replace_old: None,
            body_replace_new: None,
            body_append: None,
            parent: None,
            remove_parent: false,
            blocking: Vec::new(),
            remove_blocking: Vec::new(),
            blocked_by: Vec::new(),
            remove_blocked_by: Vec::new(),
            tags: Vec::new(),
            remove_tags: Vec::new(),
            if_match: None,
        })
        .expect_err("missing updates should fail");

        assert_eq!(error.code, ErrorCode::Validation);
        assert_eq!(error.message, "no changes specified");
    }

    #[test]
    fn update_command_reports_etag_conflict() {
        let temp = TestDir::new();
        let config = Config::default();
        config.save(temp.path()).expect("config should save");
        let store_root = temp.path().join(".ish");
        fs::create_dir_all(&store_root).expect("store root should exist");
        write_test_ishoo(
            &store_root,
            "ish-target",
            "Target",
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

        let error = update_command(
            UpdateArgs {
                id: "target".to_string(),
                status: Some("in-progress".to_string()),
                ishoo_type: None,
                priority: None,
                title: None,
                body: None,
                body_file: None,
                body_replace_old: None,
                body_replace_new: None,
                body_append: None,
                parent: None,
                remove_parent: false,
                blocking: Vec::new(),
                remove_blocking: Vec::new(),
                blocked_by: Vec::new(),
                remove_blocked_by: Vec::new(),
                tags: Vec::new(),
                remove_tags: Vec::new(),
                if_match: Some("deadbeefdeadbeef".to_string()),
            },
            false,
        )
        .expect_err("etag mismatch should fail");

        assert_eq!(error.code, ErrorCode::Conflict);
        assert!(error.message.contains("etag mismatch"));
    }

    #[test]
    fn update_command_auto_unarchives_before_updating() {
        let temp = TestDir::new();
        let config = Config::default();
        config.save(temp.path()).expect("config should save");
        let archive_root = temp.path().join(".ish").join("archive");
        fs::create_dir_all(&archive_root).expect("archive root should exist");
        write_test_ishoo(
            &archive_root,
            "ish-target",
            "Archived title",
            "completed",
            "task",
            Some("normal"),
            "Archived body.",
            None,
            &[],
            &[],
            &[],
        );
        let _guard = WorkingDirGuard::change_to(temp.path());

        let output = update_command(
            UpdateArgs {
                id: "target".to_string(),
                status: Some("todo".to_string()),
                ishoo_type: None,
                priority: None,
                title: Some("Restored title".to_string()),
                body: None,
                body_file: None,
                body_replace_old: None,
                body_replace_new: None,
                body_append: None,
                parent: None,
                remove_parent: false,
                blocking: Vec::new(),
                remove_blocking: Vec::new(),
                blocked_by: Vec::new(),
                remove_blocked_by: Vec::new(),
                tags: Vec::new(),
                remove_tags: Vec::new(),
                if_match: None,
            },
            false,
        )
        .expect("update command should succeed")
        .expect("update command should print output");

        assert!(output.contains("Updated"));
        assert!(
            temp.path()
                .join(".ish/ish-target--restored-title.md")
                .exists()
        );
        assert!(
            !temp
                .path()
                .join(".ish/archive/ish-target--archived-title.md")
                .exists()
        );
    }
}

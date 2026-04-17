use crate::app::{AppContext, AppError, json_output_error, store_app_error};
use crate::cli::CreateArgs;
use crate::core::store::CreateIshoo;
use crate::output::{ErrorCode, muted, output_success, render_id, success};
use std::fs;
use std::io::{self, Read};

pub(crate) fn create_command(args: CreateArgs, json: bool) -> Result<Option<String>, AppError> {
    let context = AppContext::load()?;
    let mut store = context.store;

    let ishoo = store
        .create(CreateIshoo {
            title: args.title.unwrap_or_else(|| "Untitled".to_string()),
            status: args.status,
            ishoo_type: args.ishoo_type,
            priority: args.priority,
            body: resolve_create_body(args.body, args.body_file)?,
            tags: args.tags,
            parent: args.parent,
            blocking: args.blocking,
            blocked_by: args.blocked_by,
            id_prefix: args.prefix,
        })
        .map_err(store_app_error)?;

    if json {
        return Ok(Some(
            output_success(ishoo.to_json(&ishoo.etag())).map_err(json_output_error)?,
        ));
    }

    Ok(Some(success(&format!(
        "Created {} {}",
        render_id(&ishoo.id),
        muted(&ishoo.path)
    ))))
}

fn resolve_create_body(
    body: Option<String>,
    body_file: Option<String>,
) -> Result<String, AppError> {
    match (body, body_file) {
        (Some(body), None) if body == "-" => read_from_stdin("body"),
        (Some(body), None) => Ok(body),
        (None, Some(path)) => fs::read_to_string(&path).map_err(|error| {
            AppError::new(
                ErrorCode::FileError,
                format!("failed to read body file `{path}`: {error}"),
            )
        }),
        (None, None) => Ok(String::new()),
        (Some(_), Some(_)) => Err(AppError::new(
            ErrorCode::Validation,
            "`--body` and `--body-file` cannot be used together",
        )),
    }
}

fn read_from_stdin(label: &str) -> Result<String, AppError> {
    let mut stdin = String::new();
    io::stdin().read_to_string(&mut stdin).map_err(|error| {
        AppError::new(
            ErrorCode::FileError,
            format!("failed to read {label} from stdin: {error}"),
        )
    })?;
    Ok(stdin)
}

#[cfg(test)]
mod tests {
    use super::create_command;
    use crate::cli::CreateArgs;
    use crate::config::Config;
    use crate::output::ErrorCode;
    use crate::test_support::{TestDir, WorkingDirGuard};
    use serde_json::Value;
    use std::fs;

    #[test]
    fn create_command_uses_defaults_and_writes_file() {
        let temp = TestDir::new();
        let mut config = Config::default_with_prefix("demo");
        config.project.name = "Create Test".to_string();
        config.save(temp.path()).expect("config should save");
        let _guard = WorkingDirGuard::change_to(temp.path());

        let output = create_command(
            CreateArgs {
                title: Some("Ship feature".to_string()),
                status: None,
                ishoo_type: None,
                priority: None,
                body: None,
                body_file: None,
                tags: Vec::new(),
                parent: None,
                blocking: Vec::new(),
                blocked_by: Vec::new(),
                prefix: None,
            },
            false,
        )
        .expect("create command should succeed")
        .expect("create command should print output");

        assert!(output.contains("Created"));

        let files = fs::read_dir(temp.path().join(".ish"))
            .expect("store root should exist")
            .map(|entry| entry.expect("entry should read").path())
            .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("md"))
            .collect::<Vec<_>>();
        assert_eq!(files.len(), 1);

        let contents = fs::read_to_string(&files[0]).expect("created file should be readable");
        assert!(contents.contains("# demo-"));
        assert!(contents.contains("title: Ship feature"));
        assert!(contents.contains("status: todo"));
        assert!(contents.contains("type: task"));
        assert!(contents.contains("priority: normal"));
    }

    #[test]
    fn create_command_supports_body_file_tags_and_relations_in_json_mode() {
        let temp = TestDir::new();
        let mut config = Config::default_with_prefix("ish");
        config.project.name = "Create JSON Test".to_string();
        config.save(temp.path()).expect("config should save");
        let store_root = temp.path().join(".ish");
        fs::create_dir_all(&store_root).expect("store root should exist");
        fs::write(
            store_root.join("ish-parent--parent.md"),
            "---\n# ish-parent\ntitle: Parent\nstatus: todo\ntype: feature\ncreated_at: 2026-01-01T00:00:00Z\nupdated_at: 2026-01-01T00:00:00Z\n---\n\nParent body.\n",
        )
        .expect("parent file should exist");
        fs::write(
            store_root.join("ish-blocker--blocker.md"),
            "---\n# ish-blocker\ntitle: Blocker\nstatus: todo\ntype: task\ncreated_at: 2026-01-01T00:00:00Z\nupdated_at: 2026-01-01T00:00:00Z\n---\n\nBlocker body.\n",
        )
        .expect("blocker file should exist");
        fs::write(
            store_root.join("ish-dependency--dependency.md"),
            "---\n# ish-dependency\ntitle: Dependency\nstatus: todo\ntype: task\ncreated_at: 2026-01-01T00:00:00Z\nupdated_at: 2026-01-01T00:00:00Z\n---\n\nDependency body.\n",
        )
        .expect("dependency file should exist");
        let body_path = temp.path().join("body.md");
        fs::write(&body_path, "Body from file\n").expect("body file should be written");
        let _guard = WorkingDirGuard::change_to(temp.path());

        let output = create_command(
            CreateArgs {
                title: Some("Wire command".to_string()),
                status: Some("in-progress".to_string()),
                ishoo_type: Some("task".to_string()),
                priority: Some("high".to_string()),
                body: None,
                body_file: Some(body_path.display().to_string()),
                tags: vec!["cli".to_string(), "json".to_string()],
                parent: Some("parent".to_string()),
                blocking: vec!["blocker".to_string()],
                blocked_by: vec!["dependency".to_string()],
                prefix: Some("feat".to_string()),
            },
            true,
        )
        .expect("create command should succeed")
        .expect("create command should print output");

        let parsed: Value = serde_json::from_str(&output).expect("json should parse");
        assert_eq!(parsed["success"], Value::Bool(true));
        assert_eq!(
            parsed["data"]["title"],
            Value::String("Wire command".to_string())
        );
        assert_eq!(
            parsed["data"]["status"],
            Value::String("in-progress".to_string())
        );
        assert_eq!(parsed["data"]["type"], Value::String("task".to_string()));
        assert_eq!(
            parsed["data"]["priority"],
            Value::String("high".to_string())
        );
        assert_eq!(
            parsed["data"]["body"],
            Value::String("Body from file\n".to_string())
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
        assert_eq!(parsed["data"]["tags"][0], Value::String("cli".to_string()));
        assert!(
            parsed["data"]["id"]
                .as_str()
                .expect("id should be present")
                .starts_with("feat-")
        );
    }

    #[test]
    fn create_command_defaults_title_to_untitled() {
        let temp = TestDir::new();
        let config = Config::default();
        config.save(temp.path()).expect("config should save");
        let _guard = WorkingDirGuard::change_to(temp.path());

        create_command(
            CreateArgs {
                title: None,
                status: None,
                ishoo_type: None,
                priority: None,
                body: None,
                body_file: None,
                tags: Vec::new(),
                parent: None,
                blocking: Vec::new(),
                blocked_by: Vec::new(),
                prefix: None,
            },
            false,
        )
        .expect("create command should succeed");

        let files = fs::read_dir(temp.path().join(".ish"))
            .expect("store root should exist")
            .map(|entry| entry.expect("entry should read").path())
            .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("md"))
            .collect::<Vec<_>>();
        assert_eq!(files.len(), 1);
        let contents = fs::read_to_string(&files[0]).expect("created file should be readable");
        assert!(contents.contains("title: Untitled"));
    }

    #[test]
    fn create_command_rejects_invalid_status() {
        let temp = TestDir::new();
        let config = Config::default();
        config.save(temp.path()).expect("config should save");
        let _guard = WorkingDirGuard::change_to(temp.path());

        let error = create_command(
            CreateArgs {
                title: Some("Broken".to_string()),
                status: Some("invalid".to_string()),
                ishoo_type: None,
                priority: None,
                body: None,
                body_file: None,
                tags: Vec::new(),
                parent: None,
                blocking: Vec::new(),
                blocked_by: Vec::new(),
                prefix: None,
            },
            false,
        )
        .expect_err("create command should fail");

        assert_eq!(error.code, ErrorCode::Validation);
        assert!(error.message.contains("invalid status"));
    }
}

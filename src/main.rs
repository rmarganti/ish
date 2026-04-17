mod app;
mod cli;
mod commands;
mod config;
mod core;
mod model;
mod output;
mod roadmap;

use clap::Parser;
use std::process::ExitCode;

use crate::app::run;
use crate::cli::Cli;
#[cfg(test)]
use crate::cli::{
    CheckArgs, Commands, CreateArgs, DeleteArgs, ListArgs, ListSortArg, RoadmapArgs, ShowArgs,
    UpdateArgs,
};
#[cfg(test)]
pub(crate) use crate::commands::update::{read_text_input, resolve_update_changes};
#[cfg(test)]
use crate::commands::{
    DeleteTarget, STORE_GITIGNORE_CONTENT, archive_command, check_command, confirm_delete,
    create_command, delete_command_with_io, init_command, list_command, prime_command,
    roadmap_command, show_command, update_command, version_output,
};
use crate::output::{danger, output_error};

#[cfg(test)]
fn load_store_from_current_dir() -> Result<
    (
        std::path::PathBuf,
        crate::config::Config,
        crate::core::store::Store,
    ),
    crate::app::AppError,
> {
    let context = crate::app::AppContext::load()?;
    Ok((context.current_dir, context.config, context.store))
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    let json = cli.json;

    match run(cli) {
        Ok(outcome) => {
            if let Some(output) = outcome.output {
                println!("{output}");
            }
            outcome.exit_code
        }
        Err(error) => {
            if json {
                println!("{}", output_error(error.code, error.message));
            } else {
                eprintln!("{}", danger(&format!("ish: {}", error.message)));
            }
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        CheckArgs, Cli, Commands, CreateArgs, DeleteArgs, DeleteTarget, ListArgs, RoadmapArgs,
        STORE_GITIGNORE_CONTENT, ShowArgs, UpdateArgs, archive_command, check_command,
        confirm_delete, create_command, delete_command_with_io, init_command, list_command,
        prime_command, roadmap_command, run, show_command, update_command, version_output,
    };
    use crate::config::{CONFIG_FILE_NAME, Config};
    use crate::core::store::{LinkRef, LinkType};
    use crate::model::ishoo::Ishoo;
    use chrono::Utc;
    use clap::Parser;
    use serde_json::Value;
    use std::fs;
    use std::io::Cursor;
    use std::path::{Path, PathBuf};
    use std::process::ExitCode;
    use std::sync::{Mutex, MutexGuard, OnceLock};
    use std::time::{SystemTime, UNIX_EPOCH};

    struct TestDir {
        path: PathBuf,
    }

    impl TestDir {
        fn new() -> Self {
            let unique = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("system time should be after unix epoch")
                .as_nanos();
            let path = std::env::temp_dir().join(format!("ish-main-test-{unique}"));
            fs::create_dir_all(&path).expect("temp dir should be created");

            Self { path }
        }

        fn path(&self) -> &Path {
            &self.path
        }
    }

    impl Drop for TestDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.path);
        }
    }

    struct WorkingDirGuard {
        _lock: MutexGuard<'static, ()>,
        original: PathBuf,
    }

    impl WorkingDirGuard {
        fn change_to(path: &Path) -> Self {
            let lock = cwd_lock()
                .lock()
                .expect("working directory test lock should not be poisoned");
            let original = std::env::current_dir().expect("current directory should be readable");
            std::env::set_current_dir(path).expect("current directory should be changed");
            Self {
                _lock: lock,
                original,
            }
        }
    }

    impl Drop for WorkingDirGuard {
        fn drop(&mut self) {
            let _ = std::env::set_current_dir(&self.original);
        }
    }

    fn cwd_lock() -> &'static Mutex<()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
    }

    fn sample_delete_target(incoming_links: usize) -> DeleteTarget {
        DeleteTarget {
            ishoo: Ishoo {
                id: "ish-target".to_string(),
                slug: "target".to_string(),
                path: "ish-target--target.md".to_string(),
                title: "Target".to_string(),
                status: "todo".to_string(),
                ishoo_type: "task".to_string(),
                priority: Some("normal".to_string()),
                tags: Vec::new(),
                created_at: Utc::now(),
                updated_at: Utc::now(),
                order: None,
                body: "Target body.".to_string(),
                parent: None,
                blocking: Vec::new(),
                blocked_by: Vec::new(),
            },
            incoming_links: (0..incoming_links)
                .map(|index| LinkRef {
                    source_id: format!("ish-ref-{index}"),
                    link_type: LinkType::Blocking,
                    target_id: "ish-target".to_string(),
                })
                .collect(),
        }
    }

    fn write_test_ishoo(
        root: &Path,
        id: &str,
        title: &str,
        status: &str,
        ishoo_type: &str,
        priority: Option<&str>,
        body: &str,
        parent: Option<&str>,
        blocking: &[&str],
        blocked_by: &[&str],
        tags: &[&str],
    ) {
        let mut content = format!(
            "---\n# {id}\ntitle: {title}\nstatus: {status}\ntype: {ishoo_type}\ncreated_at: 2026-01-01T00:00:00Z\nupdated_at: 2026-01-01T00:00:00Z\n"
        );

        if let Some(priority) = priority {
            content.push_str(&format!("priority: {priority}\n"));
        }
        if !tags.is_empty() {
            content.push_str("tags:\n");
            for tag in tags {
                content.push_str(&format!("  - {tag}\n"));
            }
        }
        if let Some(parent) = parent {
            content.push_str(&format!("parent: {parent}\n"));
        }
        if !blocking.is_empty() {
            content.push_str("blocking:\n");
            for blocked in blocking {
                content.push_str(&format!("  - {blocked}\n"));
            }
        }
        if !blocked_by.is_empty() {
            content.push_str("blocked_by:\n");
            for blocker in blocked_by {
                content.push_str(&format!("  - {blocker}\n"));
            }
        }

        content.push_str("---\n\n");
        content.push_str(body);
        content.push('\n');

        fs::write(
            root.join(format!(
                "{id}--{}.md",
                title.to_ascii_lowercase().replace(' ', "-")
            )),
            content,
        )
        .expect("ishoo file should be written");
    }

    #[test]
    fn version_output_uses_package_version() {
        assert_eq!(
            version_output(),
            format!("ish {}", env!("CARGO_PKG_VERSION"))
        );
    }

    #[test]
    fn run_prime_returns_rendered_guide_when_config_exists() {
        let temp = TestDir::new();
        let mut config = Config::default();
        config.project.name = "Prime Test".to_string();
        config.save(temp.path()).expect("config should save");
        let _guard = WorkingDirGuard::change_to(temp.path());

        let output = run(Cli {
            json: false,
            command: Some(Commands::Prime),
        })
        .expect("prime command should succeed")
        .output
        .expect("prime command should print output");

        assert!(output.contains("# ish Agent Guide"));
        assert!(output.contains("Prime Test"));
    }

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
    fn list_command_json_filters_and_omits_body_by_default() {
        let temp = TestDir::new();
        let config = Config::default();
        config.save(temp.path()).expect("config should save");
        let store_root = temp.path().join(".ish");
        fs::create_dir_all(&store_root).expect("store root should exist");
        write_test_ishoo(
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
        write_test_ishoo(
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
                ishoo_type: vec!["task".to_string()],
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
                sort: Some(super::ListSortArg::Id),
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
        assert_eq!(parsed["ishoos"][0]["id"], "ish-alpha");
        assert!(parsed["ishoos"][0].get("body").is_none());
    }

    #[test]
    fn list_command_full_json_includes_body() {
        let temp = TestDir::new();
        let config = Config::default();
        config.save(temp.path()).expect("config should save");
        let store_root = temp.path().join(".ish");
        fs::create_dir_all(&store_root).expect("store root should exist");
        write_test_ishoo(
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
                ishoo_type: Vec::new(),
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
                sort: Some(super::ListSortArg::Id),
                quiet: false,
                full: true,
            },
            true,
        )
        .expect("list command should succeed")
        .expect("list command should print output");

        let parsed: Value = serde_json::from_str(&output).expect("json should parse");
        assert_eq!(parsed["ishoos"][0]["body"], "Detailed body.");
    }

    #[test]
    fn list_command_ready_excludes_blocked_in_progress_archived_and_implicitly_completed() {
        let temp = TestDir::new();
        let config = Config::default();
        config.save(temp.path()).expect("config should save");
        let store_root = temp.path().join(".ish");
        fs::create_dir_all(&store_root).expect("store root should exist");
        write_test_ishoo(
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
        write_test_ishoo(
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
        write_test_ishoo(
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
        write_test_ishoo(
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
        write_test_ishoo(
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
        write_test_ishoo(
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
                ishoo_type: Vec::new(),
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
                sort: Some(super::ListSortArg::Id),
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
                ishoo_type: Vec::new(),
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
    fn show_command_supports_json_raw_body_and_etag_output() {
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
            &["context"],
        );
        write_test_ishoo(
            &store_root,
            "ish-blocker",
            "Blocker",
            "todo",
            "task",
            Some("high"),
            "Blocker body.",
            None,
            &[],
            &[],
            &[],
        );
        write_test_ishoo(
            &store_root,
            "ish-target",
            "Target",
            "todo",
            "task",
            Some("normal"),
            "# Heading\n\nBody text.",
            Some("ish-parent"),
            &["ish-blocker"],
            &[],
            &["cli", "show"],
        );
        let _guard = WorkingDirGuard::change_to(temp.path());

        let json_output = show_command(
            ShowArgs {
                ids: vec!["target".to_string()],
                raw: false,
                body_only: false,
                etag_only: false,
            },
            true,
        )
        .expect("show command should succeed")
        .expect("show command should print output");
        let parsed: Value = serde_json::from_str(&json_output).expect("json should parse");
        assert_eq!(parsed["success"], Value::Bool(true));
        assert_eq!(parsed["count"], Value::from(1));
        assert_eq!(parsed["ishoos"][0]["id"], "ish-target");
        assert_eq!(parsed["ishoos"][0]["parent"], "ish-parent");
        assert_eq!(parsed["ishoos"][0]["blocking"][0], "ish-blocker");
        assert!(parsed["ishoos"][0].get("etag").is_some());

        let raw_output = show_command(
            ShowArgs {
                ids: vec!["target".to_string()],
                raw: true,
                body_only: false,
                etag_only: false,
            },
            false,
        )
        .expect("show command should succeed")
        .expect("show command should print output");
        assert!(raw_output.starts_with("---\n# ish-target"));
        assert!(raw_output.contains("title: Target"));

        let body_output = show_command(
            ShowArgs {
                ids: vec!["target".to_string()],
                raw: false,
                body_only: true,
                etag_only: false,
            },
            false,
        )
        .expect("show command should succeed")
        .expect("show command should print output");
        assert_eq!(body_output, "# Heading\n\nBody text.");

        let etag_output = show_command(
            ShowArgs {
                ids: vec!["target".to_string()],
                raw: false,
                body_only: false,
                etag_only: true,
            },
            false,
        )
        .expect("show command should succeed")
        .expect("show command should print output");
        assert_eq!(etag_output.len(), 16);
        assert!(etag_output.chars().all(|ch| ch.is_ascii_hexdigit()));
    }

    #[test]
    fn show_command_human_output_renders_header_relationships_and_separator() {
        let temp = TestDir::new();
        let config = Config::default();
        config.save(temp.path()).expect("config should save");
        let store_root = temp.path().join(".ish");
        fs::create_dir_all(&store_root).expect("store root should exist");
        write_test_ishoo(
            &store_root,
            "ish-parent",
            "Parent",
            "completed",
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
            Some("high"),
            "Blocker body.",
            None,
            &[],
            &[],
            &[],
        );
        write_test_ishoo(
            &store_root,
            "ish-child",
            "Child",
            "todo",
            "task",
            Some("normal"),
            "Child body.",
            Some("ish-parent"),
            &["ish-blocker"],
            &[],
            &["demo"],
        );
        let _guard = WorkingDirGuard::change_to(temp.path());

        let output = show_command(
            ShowArgs {
                ids: vec!["child".to_string(), "parent".to_string()],
                raw: false,
                body_only: false,
                etag_only: false,
            },
            false,
        )
        .expect("show command should succeed")
        .expect("show command should print output");

        assert!(output.contains("ish-child"));
        assert!(output.contains("Path: ish-child--child.md"));
        assert!(output.contains("Parent: ish-parent"));
        assert!(output.contains("Blocking: ish-blocker"));
        assert!(output.contains("Inherited status: completed from ish-parent"));
        assert!(output.contains("#demo"));
        assert!(output.contains("Child body."));
        assert!(output.contains("════════"));
    }

    #[test]
    fn confirm_delete_prints_title_path_and_incoming_link_count() {
        let mut input = Cursor::new(b"yes\n".to_vec());
        let mut output = Vec::new();

        let confirmed = confirm_delete(&[sample_delete_target(2)], &mut input, &mut output)
            .expect("confirmation prompt should succeed");
        let rendered = String::from_utf8(output).expect("prompt should be utf8");

        assert!(confirmed);
        assert!(rendered.contains("Delete 1 ishoo?"));
        assert!(rendered.contains("title: Target"));
        assert!(rendered.contains("path: ish-target--target.md"));
        assert!(rendered.contains("incoming links: 2"));
        assert!(rendered.contains("remove 2 incoming link(s)"));
        assert!(rendered.contains("Proceed? [y/N]:"));
    }

    #[test]
    fn delete_command_force_removes_file_and_cleans_references() {
        let temp = TestDir::new();
        let config = Config::default();
        config.save(temp.path()).expect("config should save");
        let store_root = temp.path().join(".ish");
        fs::create_dir_all(&store_root).expect("store root should exist");
        fs::write(
            store_root.join("ish-target--target.md"),
            "---\n# ish-target\ntitle: Target\nstatus: todo\ntype: task\ncreated_at: 2026-01-01T00:00:00Z\nupdated_at: 2026-01-01T00:00:00Z\n---\n\nTarget body.\n",
        )
        .expect("target file should exist");
        fs::write(
            store_root.join("ish-ref--ref.md"),
            "---\n# ish-ref\ntitle: Ref\nstatus: todo\ntype: task\ncreated_at: 2026-01-01T00:00:00Z\nupdated_at: 2026-01-01T00:00:00Z\nparent: ish-target\nblocking:\n  - ish-target\nblocked_by:\n  - ish-target\n---\n\nRef body.\n",
        )
        .expect("ref file should exist");
        let _guard = WorkingDirGuard::change_to(temp.path());

        let output = delete_command_with_io(
            DeleteArgs {
                ids: vec!["target".to_string()],
                force: true,
            },
            false,
            &mut Cursor::new(Vec::new()),
            &mut Vec::new(),
        )
        .expect("delete command should succeed")
        .expect("delete command should print output");

        assert!(output.contains("Deleted"));
        assert!(output.contains("cleaned 3 incoming link(s)"));
        assert!(!store_root.join("ish-target--target.md").exists());

        let ref_contents =
            fs::read_to_string(store_root.join("ish-ref--ref.md")).expect("ref file should exist");
        assert!(!ref_contents.contains("parent: ish-target"));
        assert!(!ref_contents.contains("- ish-target"));
    }

    #[test]
    fn delete_command_returns_cancelled_message_without_deleting() {
        let temp = TestDir::new();
        let config = Config::default();
        config.save(temp.path()).expect("config should save");
        let store_root = temp.path().join(".ish");
        fs::create_dir_all(&store_root).expect("store root should exist");
        fs::write(
            store_root.join("ish-target--target.md"),
            "---\n# ish-target\ntitle: Target\nstatus: todo\ntype: task\ncreated_at: 2026-01-01T00:00:00Z\nupdated_at: 2026-01-01T00:00:00Z\n---\n\nTarget body.\n",
        )
        .expect("target file should exist");
        let _guard = WorkingDirGuard::change_to(temp.path());
        let mut prompt_output = Vec::new();

        let output = delete_command_with_io(
            DeleteArgs {
                ids: vec!["target".to_string()],
                force: false,
            },
            false,
            &mut Cursor::new(b"n\n".to_vec()),
            &mut prompt_output,
        )
        .expect("delete command should succeed")
        .expect("delete command should print output");

        assert!(output.contains("Delete cancelled"));
        assert!(store_root.join("ish-target--target.md").exists());
        let prompt_output = String::from_utf8(prompt_output).expect("prompt should be utf8");
        assert!(prompt_output.contains("title: Target"));
    }

    #[test]
    fn delete_command_json_implies_force_and_returns_deleted_items() {
        let temp = TestDir::new();
        let config = Config::default();
        config.save(temp.path()).expect("config should save");
        let store_root = temp.path().join(".ish");
        fs::create_dir_all(&store_root).expect("store root should exist");
        fs::write(
            store_root.join("ish-target--target.md"),
            "---\n# ish-target\ntitle: Target\nstatus: todo\ntype: task\ncreated_at: 2026-01-01T00:00:00Z\nupdated_at: 2026-01-01T00:00:00Z\n---\n\nTarget body.\n",
        )
        .expect("target file should exist");
        let _guard = WorkingDirGuard::change_to(temp.path());
        let mut prompt_output = Vec::new();

        let output = delete_command_with_io(
            DeleteArgs {
                ids: vec!["target".to_string()],
                force: false,
            },
            true,
            &mut Cursor::new(Vec::new()),
            &mut prompt_output,
        )
        .expect("delete command should succeed")
        .expect("delete command should print output");

        let parsed: Value = serde_json::from_str(&output).expect("json should parse");
        assert_eq!(parsed["success"], Value::Bool(true));
        assert_eq!(parsed["data"]["count"], Value::from(1));
        assert_eq!(parsed["data"]["cleaned_links"], Value::from(0));
        assert_eq!(parsed["data"]["deleted"][0]["id"], "ish-target");
        assert!(prompt_output.is_empty());
        assert!(!store_root.join("ish-target--target.md").exists());
    }

    #[test]
    fn cli_parses_delete_alias() {
        let cli = Cli::try_parse_from(["ish", "rm", "target"])
            .expect("delete alias should parse successfully");

        match cli.command {
            Some(Commands::Delete(args)) => {
                assert_eq!(args.ids, vec!["target".to_string()]);
                assert!(!args.force);
            }
            _ => panic!("expected delete command"),
        }
    }

    #[test]
    fn cli_parses_list_alias() {
        let cli = Cli::try_parse_from(["ish", "ls", "--ready"])
            .expect("list alias should parse successfully");

        match cli.command {
            Some(Commands::List(args)) => {
                assert!(args.ready);
                assert!(!args.quiet);
            }
            _ => panic!("expected list command"),
        }
    }

    #[test]
    fn cli_parses_show_flags() {
        let cli = Cli::try_parse_from(["ish", "show", "abcd", "efgh", "--body-only"])
            .expect("show command should parse successfully");

        match cli.command {
            Some(Commands::Show(args)) => {
                assert_eq!(args.ids, vec!["abcd".to_string(), "efgh".to_string()]);
                assert!(args.body_only);
                assert!(!args.raw);
                assert!(!args.etag_only);
            }
            _ => panic!("expected show command"),
        }
    }

    #[test]
    fn run_dispatches_list_command_through_app_layer() {
        let temp = TestDir::new();
        let config = Config::default();
        config.save(temp.path()).expect("config should save");
        fs::create_dir_all(temp.path().join(".ish")).expect("store dir should be created");
        write_test_ishoo(
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
            command: Some(Commands::List(ListArgs {
                status: Vec::new(),
                no_status: Vec::new(),
                ishoo_type: Vec::new(),
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
            })),
        })
        .expect("run should succeed")
        .output
        .expect("list output should be present");

        let parsed: Value = serde_json::from_str(&output).expect("json should parse");
        assert_eq!(parsed["success"], Value::Bool(true));
        assert_eq!(parsed["count"], Value::from(1));
        assert_eq!(parsed["ishoos"][0]["id"], "ish-abcd");
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

        assert_eq!(error.code, crate::output::ErrorCode::Validation);
        assert!(error.message.contains("invalid status"));
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

        let changes = super::resolve_update_changes(UpdateArgs {
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

        let (_, _, mut store) = super::load_store_from_current_dir().expect("store should load");
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
        let value = super::read_text_input(Cursor::new(b"stdin body\n".to_vec()), "body append")
            .expect("stdin text should be read");

        assert_eq!(value, "stdin body\n");
    }

    #[test]
    fn update_command_rejects_when_no_changes_specified() {
        let error = super::resolve_update_changes(UpdateArgs {
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

        assert_eq!(error.code, crate::output::ErrorCode::Validation);
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

        assert_eq!(error.code, crate::output::ErrorCode::Conflict);
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

    #[test]
    fn run_init_creates_project_files_with_defaults() {
        let temp = TestDir::new();
        let project_dir = temp.path().join("demo-project");
        fs::create_dir_all(&project_dir).expect("project dir should exist");
        let _guard = WorkingDirGuard::change_to(&project_dir);

        let output = run(Cli {
            json: false,
            command: Some(Commands::Init),
        })
        .expect("init command should succeed")
        .output
        .expect("init command should print output");

        assert!(output.contains("initialized ish project"));
        assert_eq!(
            fs::read_to_string(project_dir.join(".ish").join(".gitignore"))
                .expect("gitignore should be written"),
            STORE_GITIGNORE_CONTENT
        );

        let config = Config::load(project_dir.join(CONFIG_FILE_NAME)).expect("config should load");
        assert_eq!(config.ish.path, ".ish");
        assert_eq!(config.ish.prefix, "demo-project-");
        assert_eq!(config.project.name, "demo-project");
    }

    #[test]
    fn prime_command_silently_exits_without_config() {
        let temp = TestDir::new();
        let _guard = WorkingDirGuard::change_to(temp.path());

        assert!(
            prime_command(false)
                .expect("prime command should succeed")
                .is_none()
        );
    }

    #[test]
    fn init_command_is_idempotent_and_preserves_existing_config() {
        let temp = TestDir::new();
        let project_dir = temp.path().join("custom-project");
        fs::create_dir_all(&project_dir).expect("project dir should exist");
        let _guard = WorkingDirGuard::change_to(&project_dir);

        let mut config = Config::default_with_prefix("custom");
        config.project.name = "Custom Name".to_string();
        config.save(&project_dir).expect("config should save");

        let output = init_command(false)
            .expect("init command should succeed")
            .expect("init command should print output");

        assert!(output.contains("already initialized"));
        let loaded = Config::load(project_dir.join(CONFIG_FILE_NAME)).expect("config should load");
        assert_eq!(loaded.ish.prefix, "custom");
        assert_eq!(loaded.project.name, "Custom Name");
        assert_eq!(
            fs::read_to_string(project_dir.join(".ish").join(".gitignore"))
                .expect("gitignore should be written"),
            STORE_GITIGNORE_CONTENT
        );
    }

    #[test]
    fn run_roadmap_returns_rendered_output_when_config_exists() {
        let temp = TestDir::new();
        let mut config = Config::default();
        config.project.name = "Roadmap Test".to_string();
        config.save(temp.path()).expect("config should save");
        let store_root = temp.path().join(".ish");
        fs::create_dir_all(&store_root).expect("store root should exist");
        fs::write(
            store_root.join("ish-m1--milestone.md"),
            "---\n# ish-m1\ntitle: Milestone\nstatus: todo\ntype: milestone\ncreated_at: 2026-01-01T00:00:00Z\nupdated_at: 2026-01-01T00:00:00Z\n---\n\nMilestone body.\n",
        )
        .expect("milestone file should exist");
        let _guard = WorkingDirGuard::change_to(temp.path());

        let output = run(Cli {
            command: Some(Commands::Roadmap(RoadmapArgs {
                include_done: false,
                status: Vec::new(),
                no_status: Vec::new(),
                no_links: true,
                link_prefix: None,
            })),
            json: false,
        })
        .expect("roadmap command should succeed")
        .output
        .expect("roadmap command should print output");

        assert!(output.contains("# Roadmap"));
        assert!(output.contains("Milestone: Milestone (ish-m1)"));
    }

    #[test]
    fn roadmap_command_errors_without_config() {
        let temp = TestDir::new();
        let _guard = WorkingDirGuard::change_to(temp.path());

        let error = roadmap_command(
            RoadmapArgs {
                include_done: false,
                status: Vec::new(),
                no_status: Vec::new(),
                no_links: false,
                link_prefix: None,
            },
            false,
        )
        .expect_err("roadmap command should fail without config");

        assert!(error.message.contains("no `.ish.yml` found"));
    }

    #[test]
    fn run_version_wraps_output_in_json_mode() {
        let output = run(Cli {
            json: true,
            command: Some(Commands::Version),
        })
        .expect("version command should succeed")
        .output
        .expect("version command should print output");

        let parsed: Value = serde_json::from_str(&output).expect("json should parse");
        assert_eq!(parsed["success"], Value::Bool(true));
        assert_eq!(parsed["message"], Value::String(version_output()));
    }

    #[test]
    fn init_command_wraps_message_in_json_mode() {
        let temp = TestDir::new();
        let project_dir = temp.path().join("json-project");
        fs::create_dir_all(&project_dir).expect("project dir should exist");
        let _guard = WorkingDirGuard::change_to(&project_dir);

        let output = init_command(true)
            .expect("init command should succeed")
            .expect("init command should print output");

        let parsed: Value = serde_json::from_str(&output).expect("json should parse");
        assert_eq!(parsed["success"], Value::Bool(true));
        assert!(
            parsed["message"]
                .as_str()
                .expect("message should be present")
                .contains("initialized ish project")
        );
    }

    #[test]
    fn run_roadmap_wraps_nested_json_in_response() {
        let temp = TestDir::new();
        let mut config = Config::default();
        config.project.name = "Roadmap JSON Test".to_string();
        config.save(temp.path()).expect("config should save");
        let store_root = temp.path().join(".ish");
        fs::create_dir_all(&store_root).expect("store root should exist");
        fs::write(
            store_root.join("ish-m1--milestone.md"),
            "---\n# ish-m1\ntitle: Milestone\nstatus: todo\ntype: milestone\ncreated_at: 2026-01-01T00:00:00Z\nupdated_at: 2026-01-01T00:00:00Z\n---\n\nMilestone body.\n",
        )
        .expect("milestone file should exist");
        let _guard = WorkingDirGuard::change_to(temp.path());

        let output = run(Cli {
            json: true,
            command: Some(Commands::Roadmap(RoadmapArgs {
                include_done: false,
                status: Vec::new(),
                no_status: Vec::new(),
                no_links: true,
                link_prefix: None,
            })),
        })
        .expect("roadmap command should succeed")
        .output
        .expect("roadmap command should print output");

        let parsed: Value = serde_json::from_str(&output).expect("json should parse");
        assert_eq!(parsed["success"], Value::Bool(true));
        assert_eq!(parsed["data"]["milestones"][0]["milestone"]["id"], "ish-m1");
    }

    #[test]
    fn run_without_command_returns_validation_error_in_json_mode() {
        let output = run(Cli {
            json: true,
            command: None,
        })
        .expect("run should succeed")
        .output
        .expect("run should print output");

        let parsed: Value = serde_json::from_str(&output).expect("json should parse");
        assert_eq!(parsed["success"], Value::Bool(false));
        assert_eq!(parsed["code"], Value::String("validation".to_string()));
    }

    #[test]
    fn prime_command_wraps_markdown_in_json_mode() {
        let temp = TestDir::new();
        let mut config = Config::default();
        config.project.name = "Prime JSON Test".to_string();
        config.save(temp.path()).expect("config should save");
        let _guard = WorkingDirGuard::change_to(temp.path());

        let output = prime_command(true)
            .expect("prime command should succeed")
            .expect("prime command should print output");

        let parsed: Value = serde_json::from_str(&output).expect("json should parse");
        assert_eq!(parsed["success"], Value::Bool(true));
        assert!(
            parsed["message"]
                .as_str()
                .expect("message should be present")
                .contains("# ish Agent Guide")
        );
    }

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
            .expect("updated ishoo should be readable");

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

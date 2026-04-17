mod cli;
mod config;
mod core;
mod model;
mod output;
mod roadmap;

use clap::Parser;
use serde_json::Value;
use std::fs;
use std::path::Path;
use std::process::ExitCode;

use crate::cli::prime_output;
use crate::config::{CONFIG_FILE_NAME, Config, find_config};
use crate::output::{ErrorCode, output_error, output_message, output_success};
use crate::roadmap::{RoadmapOptions, roadmap_output};

const STORE_DIRECTORY: &str = ".ish";
const STORE_GITIGNORE_NAME: &str = ".gitignore";
const STORE_GITIGNORE_CONTENT: &str = ".conversations/\n";

/// A terminal-based issue tracker.
#[derive(Parser)]
#[command(name = "ish", version, about)]
struct Cli {
    /// Output structured JSON.
    #[arg(long, global = true)]
    json: bool,
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(clap::Subcommand)]
enum Commands {
    /// Initialize a new ish project in the current directory.
    Init,
    /// Print AI-agent guidance for the current ish project.
    Prime,
    /// Generate a roadmap from milestone and epic hierarchy.
    Roadmap(RoadmapArgs),
    /// Print the current ish version.
    Version,
}

#[derive(clap::Args)]
struct RoadmapArgs {
    /// Include completed and scrapped items.
    #[arg(long)]
    include_done: bool,
    /// Filter milestones by status.
    #[arg(long = "status")]
    status: Vec<String>,
    /// Exclude milestones by status.
    #[arg(long = "no-status")]
    no_status: Vec<String>,
    /// Render plain IDs instead of markdown links.
    #[arg(long)]
    no_links: bool,
    /// Override the link prefix used in markdown links.
    #[arg(long)]
    link_prefix: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct AppError {
    code: ErrorCode,
    message: String,
}

impl AppError {
    fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }
}

fn version_output() -> String {
    format!("ish {}", env!("CARGO_PKG_VERSION"))
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    let json = cli.json;

    match run(cli) {
        Ok(Some(output)) => {
            println!("{output}");
            ExitCode::SUCCESS
        }
        Ok(None) => ExitCode::SUCCESS,
        Err(error) => {
            if json {
                println!("{}", output_error(error.code, error.message));
            } else {
                eprintln!("ish: {}", error.message);
            }
            ExitCode::FAILURE
        }
    }
}

fn run(cli: Cli) -> Result<Option<String>, AppError> {
    match cli.command {
        Some(Commands::Init) => init_command(cli.json),
        Some(Commands::Prime) => prime_command(cli.json),
        Some(Commands::Roadmap(args)) => roadmap_command(args, cli.json),
        Some(Commands::Version) => {
            if cli.json {
                Ok(Some(
                    output_message(version_output()).map_err(json_output_error)?,
                ))
            } else {
                Ok(Some(version_output()))
            }
        }
        None => {
            let message = "ish: no command specified. Run `ish --help` for usage.";
            if cli.json {
                Ok(Some(output_error(ErrorCode::Validation, message)))
            } else {
                Ok(Some(message.to_string()))
            }
        }
    }
}

fn init_command(json: bool) -> Result<Option<String>, AppError> {
    let current_dir = std::env::current_dir().map_err(|error| {
        AppError::new(
            ErrorCode::FileError,
            format!("failed to determine current directory: {error}"),
        )
    })?;
    let project_name = project_name(&current_dir)?;

    fs::create_dir_all(current_dir.join(STORE_DIRECTORY)).map_err(|error| {
        AppError::new(
            ErrorCode::FileError,
            format!("failed to create `{STORE_DIRECTORY}` directory: {error}"),
        )
    })?;

    let gitignore_path = current_dir.join(STORE_DIRECTORY).join(STORE_GITIGNORE_NAME);
    if !gitignore_path.exists() {
        fs::write(&gitignore_path, STORE_GITIGNORE_CONTENT).map_err(|error| {
            AppError::new(
                ErrorCode::FileError,
                format!("failed to write `{}`: {error}", gitignore_path.display()),
            )
        })?;
    }

    let config_path = current_dir.join(CONFIG_FILE_NAME);
    let message = if config_path.exists() {
        format!(
            "ish project already initialized in `{}`",
            current_dir.display()
        )
    } else {
        let mut config = Config::default_with_prefix(format!("{project_name}-"));
        config.project.name = project_name;
        config.save(&current_dir).map_err(|error| {
            AppError::new(
                ErrorCode::FileError,
                format!("failed to write `{}`: {error}", config_path.display()),
            )
        })?;
        format!("initialized ish project in `{}`", current_dir.display())
    };

    if json {
        Ok(Some(output_message(message).map_err(json_output_error)?))
    } else {
        Ok(Some(message))
    }
}

fn project_name(dir: &Path) -> Result<String, AppError> {
    dir.file_name()
        .and_then(|name| name.to_str())
        .filter(|name| !name.is_empty())
        .map(str::to_owned)
        .ok_or_else(|| {
            AppError::new(
                ErrorCode::Validation,
                "failed to derive project name from current directory",
            )
        })
}

fn roadmap_command(args: RoadmapArgs, json: bool) -> Result<Option<String>, AppError> {
    let current_dir = std::env::current_dir().map_err(|error| {
        AppError::new(
            ErrorCode::FileError,
            format!("failed to determine current directory: {error}"),
        )
    })?;

    let output = roadmap_output(
        &current_dir,
        &RoadmapOptions {
            include_done: args.include_done,
            status: args.status,
            no_status: args.no_status,
            no_links: args.no_links,
            link_prefix: args.link_prefix,
            json,
        },
    )
    .map_err(classify_app_error)?;

    if json {
        let Some(output) = output else {
            return Ok(None);
        };
        let data: Value = serde_json::from_str(&output).map_err(|error| {
            AppError::new(
                ErrorCode::FileError,
                format!("failed to parse command JSON output: {error}"),
            )
        })?;
        Ok(Some(output_success(data).map_err(json_output_error)?))
    } else {
        Ok(output)
    }
}

fn prime_command(json: bool) -> Result<Option<String>, AppError> {
    let current_dir = std::env::current_dir().map_err(|error| {
        AppError::new(
            ErrorCode::FileError,
            format!("failed to determine current directory: {error}"),
        )
    })?;
    let Some(config_path) = find_config(&current_dir) else {
        return Ok(None);
    };

    let config = Config::load(&config_path).map_err(|error| {
        AppError::new(
            ErrorCode::FileError,
            format!("failed to load `{}`: {error}", config_path.display()),
        )
    })?;

    let output = prime_output(&config);
    if json {
        Ok(Some(output_message(output).map_err(json_output_error)?))
    } else {
        Ok(Some(output))
    }
}

fn classify_app_error(message: String) -> AppError {
    let code = if message.contains("no `.ish.yml` found") {
        ErrorCode::NotFound
    } else if message.contains("etag") || message.contains("conflict") {
        ErrorCode::Conflict
    } else if message.contains("invalid") {
        ErrorCode::Validation
    } else {
        ErrorCode::FileError
    };

    AppError::new(code, message)
}

fn json_output_error(message: String) -> AppError {
    AppError::new(ErrorCode::FileError, message)
}

#[cfg(test)]
mod tests {
    use super::{
        Cli, Commands, RoadmapArgs, STORE_GITIGNORE_CONTENT, init_command, prime_command,
        roadmap_command, run, version_output,
    };
    use crate::config::{CONFIG_FILE_NAME, Config};
    use serde_json::Value;
    use std::fs;
    use std::path::{Path, PathBuf};
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
        .expect("prime command should print output");

        assert!(output.contains("# ish Agent Guide"));
        assert!(output.contains("Prime Test"));
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
}

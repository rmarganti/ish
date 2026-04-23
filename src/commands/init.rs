use crate::app::{AppError, current_dir, json_output_error};
use crate::cli::InitArgs;
use crate::config::{CONFIG_FILE_NAME, Config};
use crate::output::{ErrorCode, output_message, success};
use std::fs;
use std::path::Path;

const STORE_DIRECTORY: &str = ".ish";

pub(crate) fn init_command(args: InitArgs, json: bool) -> Result<Option<String>, AppError> {
    let current_dir = current_dir()?;
    let project_name = project_name(&current_dir)?;

    let config_path = current_dir.join(CONFIG_FILE_NAME);
    let already_initialized = config_path.exists();

    let store_path = if already_initialized {
        let config = Config::load(&config_path).map_err(|error| {
            AppError::new(
                ErrorCode::FileError,
                format!("failed to load `{}`: {error}", config_path.display()),
            )
        })?;
        config.ish.path.clone()
    } else {
        STORE_DIRECTORY.to_string()
    };

    fs::create_dir_all(current_dir.join(&store_path)).map_err(|error| {
        AppError::new(
            ErrorCode::FileError,
            format!("failed to create `{store_path}` directory: {error}"),
        )
    })?;

    let was_stealth = is_stealth(&current_dir, &store_path)?;

    let message = match (already_initialized, args.stealth, was_stealth) {
        // Fresh init
        (false, false, _) => {
            save_default_config(&current_dir, &project_name)?;
            format!("initialized ish project in `{}`", current_dir.display())
        }
        // Fresh init with stealth
        (false, true, _) => {
            require_git_repo(&current_dir)?;
            save_default_config(&current_dir, &project_name)?;
            add_stealth_excludes(&current_dir, &store_path)?;
            format!(
                "initialized ish project in `{}` (stealth)",
                current_dir.display()
            )
        }
        // Re-run init on non-stealth repo
        (true, false, false) => {
            format!(
                "ish project already initialized in `{}`",
                current_dir.display()
            )
        }
        // Re-run stealth init on stealth repo
        (true, true, true) => {
            format!(
                "ish project already initialized in `{}` (stealth)",
                current_dir.display()
            )
        }
        // Enable stealth on existing non-stealth repo
        (true, true, false) => {
            require_git_repo(&current_dir)?;
            add_stealth_excludes(&current_dir, &store_path)?;
            "stealth mode enabled".to_string()
        }
        // Disable stealth on existing stealth repo
        (true, false, true) => {
            remove_stealth_excludes(&current_dir, &store_path)?;
            "stealth mode disabled".to_string()
        }
    };

    if json {
        Ok(Some(output_message(message).map_err(json_output_error)?))
    } else {
        Ok(Some(success(&message)))
    }
}

fn save_default_config(current_dir: &Path, project_name: &str) -> Result<(), AppError> {
    let config_path = current_dir.join(CONFIG_FILE_NAME);
    let mut config = Config::default_with_prefix(format!("{project_name}-"));
    config.project.name = project_name.to_string();
    config.save(current_dir).map_err(|error| {
        AppError::new(
            ErrorCode::FileError,
            format!("failed to write `{}`: {error}", config_path.display()),
        )
    })?;
    Ok(())
}

fn require_git_repo(current_dir: &Path) -> Result<(), AppError> {
    if !current_dir.join(".git").is_dir() {
        return Err(AppError::new(
            ErrorCode::Validation,
            "--stealth requires a git repository",
        ));
    }
    Ok(())
}

fn exclude_path(current_dir: &Path) -> std::path::PathBuf {
    current_dir.join(".git/info/exclude")
}

fn exclude_entries(store_path: &str) -> Vec<String> {
    vec![store_path.to_string(), CONFIG_FILE_NAME.to_string()]
}

fn is_stealth(current_dir: &Path, store_path: &str) -> Result<bool, AppError> {
    let path = exclude_path(current_dir);
    if !path.exists() {
        return Ok(false);
    }
    let content = fs::read_to_string(&path).map_err(|error| {
        AppError::new(
            ErrorCode::FileError,
            format!("failed to read `{}`: {error}", path.display()),
        )
    })?;
    let entries = exclude_entries(store_path);
    Ok(entries
        .iter()
        .all(|entry| content.lines().any(|line| line.trim() == entry.as_str())))
}

fn add_stealth_excludes(current_dir: &Path, store_path: &str) -> Result<(), AppError> {
    let path = exclude_path(current_dir);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| {
            AppError::new(
                ErrorCode::FileError,
                format!("failed to create `{}`: {error}", parent.display()),
            )
        })?;
    }

    let mut content = if path.exists() {
        fs::read_to_string(&path).map_err(|error| {
            AppError::new(
                ErrorCode::FileError,
                format!("failed to read `{}`: {error}", path.display()),
            )
        })?
    } else {
        String::new()
    };

    let entries = exclude_entries(store_path);
    for entry in &entries {
        if !content.lines().any(|line| line.trim() == entry.as_str()) {
            if !content.is_empty() && !content.ends_with('\n') {
                content.push('\n');
            }
            content.push_str(entry);
            content.push('\n');
        }
    }

    fs::write(&path, &content).map_err(|error| {
        AppError::new(
            ErrorCode::FileError,
            format!("failed to write `{}`: {error}", path.display()),
        )
    })?;

    Ok(())
}

fn remove_stealth_excludes(current_dir: &Path, store_path: &str) -> Result<(), AppError> {
    let path = exclude_path(current_dir);
    if !path.exists() {
        return Ok(());
    }

    let content = fs::read_to_string(&path).map_err(|error| {
        AppError::new(
            ErrorCode::FileError,
            format!("failed to read `{}`: {error}", path.display()),
        )
    })?;

    let entries = exclude_entries(store_path);
    let filtered: Vec<&str> = content
        .lines()
        .filter(|line| !entries.iter().any(|entry| line.trim() == entry.as_str()))
        .collect();

    let mut result = filtered.join("\n");
    if !result.is_empty() {
        result.push('\n');
    }

    fs::write(&path, &result).map_err(|error| {
        AppError::new(
            ErrorCode::FileError,
            format!("failed to write `{}`: {error}", path.display()),
        )
    })?;

    Ok(())
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

#[cfg(test)]
mod tests {
    use super::init_command;
    use crate::app::run;
    use crate::cli::{Cli, Commands, InitArgs};
    use crate::config::{CONFIG_FILE_NAME, Config};
    use crate::test_support::{TestDir, WorkingDirGuard};
    use serde_json::Value;
    use std::fs;

    fn make_git_repo(project_dir: &std::path::Path) {
        fs::create_dir_all(project_dir.join(".git/info")).expect("git dir should be created");
    }

    #[test]
    fn run_init_creates_project_files_with_defaults() {
        let temp = TestDir::new();
        let project_dir = temp.path().join("demo-project");
        fs::create_dir_all(&project_dir).expect("project dir should exist");
        let _guard = WorkingDirGuard::change_to(&project_dir);

        let output = run(Cli {
            json: false,
            command: Commands::Init(InitArgs { stealth: false }),
        })
        .expect("init command should succeed")
        .output
        .expect("init command should print output");

        assert!(output.contains("initialized ish project"));
        assert!(!output.contains("stealth"));
        assert!(project_dir.join(".ish").is_dir());

        let config = Config::load(project_dir.join(CONFIG_FILE_NAME)).expect("config should load");
        assert_eq!(config.ish.path, ".ish");
        assert_eq!(config.ish.prefix, "demo-project-");
        assert_eq!(config.project.name, "demo-project");
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

        let output = init_command(InitArgs { stealth: false }, false)
            .expect("init command should succeed")
            .expect("init command should print output");

        assert!(output.contains("already initialized"));
        assert!(!output.contains("stealth"));
        let loaded = Config::load(project_dir.join(CONFIG_FILE_NAME)).expect("config should load");
        assert_eq!(loaded.ish.prefix, "custom");
        assert_eq!(loaded.project.name, "Custom Name");
        assert!(project_dir.join(".ish").is_dir());
    }

    #[test]
    fn init_command_wraps_message_in_json_mode() {
        let temp = TestDir::new();
        let project_dir = temp.path().join("json-project");
        fs::create_dir_all(&project_dir).expect("project dir should exist");
        let _guard = WorkingDirGuard::change_to(&project_dir);

        let output = init_command(InitArgs { stealth: false }, true)
            .expect("init command should succeed")
            .expect("init command should print output");

        let parsed: Value = serde_json::from_str(&output).expect("json should parse");
        assert!(
            parsed
                .as_str()
                .expect("should be a string")
                .contains("initialized ish project")
        );
    }

    #[test]
    fn stealth_init_creates_exclude_entries() {
        let temp = TestDir::new();
        let project_dir = temp.path().join("stealth-project");
        fs::create_dir_all(&project_dir).expect("project dir should exist");
        make_git_repo(&project_dir);
        let _guard = WorkingDirGuard::change_to(&project_dir);

        let output = init_command(InitArgs { stealth: true }, false)
            .expect("init command should succeed")
            .expect("init command should print output");

        assert!(output.contains("initialized ish project"));
        assert!(output.contains("stealth"));

        let exclude = fs::read_to_string(project_dir.join(".git/info/exclude"))
            .expect("exclude file should exist");
        assert!(exclude.contains(".ish"));
        assert!(exclude.contains(".ish.yml"));
    }

    #[test]
    fn stealth_init_errors_without_git_repo() {
        let temp = TestDir::new();
        let project_dir = temp.path().join("no-git-project");
        fs::create_dir_all(&project_dir).expect("project dir should exist");
        let _guard = WorkingDirGuard::change_to(&project_dir);

        let err = init_command(InitArgs { stealth: true }, false)
            .expect_err("stealth init should fail without git repo");

        assert!(err.message.contains("--stealth requires a git repository"));
    }

    #[test]
    fn stealth_enable_on_existing_project() {
        let temp = TestDir::new();
        let project_dir = temp.path().join("enable-stealth");
        fs::create_dir_all(&project_dir).expect("project dir should exist");
        make_git_repo(&project_dir);
        let _guard = WorkingDirGuard::change_to(&project_dir);

        // First: normal init
        init_command(InitArgs { stealth: false }, false).expect("init should succeed");

        // Then: enable stealth
        let output = init_command(InitArgs { stealth: true }, false)
            .expect("stealth enable should succeed")
            .expect("should print output");

        assert!(output.contains("stealth mode enabled"));

        let exclude = fs::read_to_string(project_dir.join(".git/info/exclude"))
            .expect("exclude file should exist");
        assert!(exclude.contains(".ish"));
        assert!(exclude.contains(".ish.yml"));
    }

    #[test]
    fn stealth_disable_on_stealth_project() {
        let temp = TestDir::new();
        let project_dir = temp.path().join("disable-stealth");
        fs::create_dir_all(&project_dir).expect("project dir should exist");
        make_git_repo(&project_dir);
        let _guard = WorkingDirGuard::change_to(&project_dir);

        // First: stealth init
        init_command(InitArgs { stealth: true }, false).expect("stealth init should succeed");

        // Then: disable stealth
        let output = init_command(InitArgs { stealth: false }, false)
            .expect("stealth disable should succeed")
            .expect("should print output");

        assert!(output.contains("stealth mode disabled"));

        let exclude = fs::read_to_string(project_dir.join(".git/info/exclude"))
            .expect("exclude file should exist");
        assert!(!exclude.contains(".ish"));
        assert!(!exclude.contains(".ish.yml"));
    }

    #[test]
    fn stealth_reinit_is_idempotent() {
        let temp = TestDir::new();
        let project_dir = temp.path().join("stealth-idem");
        fs::create_dir_all(&project_dir).expect("project dir should exist");
        make_git_repo(&project_dir);
        let _guard = WorkingDirGuard::change_to(&project_dir);

        init_command(InitArgs { stealth: true }, false).expect("stealth init should succeed");

        let output = init_command(InitArgs { stealth: true }, false)
            .expect("stealth reinit should succeed")
            .expect("should print output");

        assert!(output.contains("already initialized"));
        assert!(output.contains("stealth"));
    }

    #[test]
    fn stealth_respects_custom_store_path() {
        let temp = TestDir::new();
        let project_dir = temp.path().join("custom-path");
        fs::create_dir_all(&project_dir).expect("project dir should exist");
        make_git_repo(&project_dir);
        let _guard = WorkingDirGuard::change_to(&project_dir);

        let mut config = Config::default_with_prefix("custom-");
        config.ish.path = ".issues".to_string();
        config.project.name = "custom-path".to_string();
        config.save(&project_dir).expect("config should save");
        fs::create_dir_all(project_dir.join(".issues")).expect("store dir should exist");

        let output = init_command(InitArgs { stealth: true }, false)
            .expect("stealth enable should succeed")
            .expect("should print output");

        assert!(output.contains("stealth mode enabled"));

        let exclude = fs::read_to_string(project_dir.join(".git/info/exclude"))
            .expect("exclude file should exist");
        assert!(exclude.contains(".issues"));
        assert!(exclude.contains(".ish.yml"));
        assert!(!exclude.contains("\n.ish\n"));
    }
}

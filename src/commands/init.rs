use crate::app::{AppError, current_dir, json_output_error};
use crate::config::{CONFIG_FILE_NAME, Config};
use crate::output::{ErrorCode, output_message, success};
use std::fs;
use std::path::Path;

const STORE_DIRECTORY: &str = ".ish";
const STORE_GITIGNORE_NAME: &str = ".gitignore";
pub(crate) const STORE_GITIGNORE_CONTENT: &str = ".conversations/\n";

pub(crate) fn init_command(json: bool) -> Result<Option<String>, AppError> {
    let current_dir = current_dir()?;
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
        Ok(Some(success(&message)))
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

#[cfg(test)]
mod tests {
    use super::{STORE_GITIGNORE_CONTENT, init_command};
    use crate::app::run;
    use crate::cli::{Cli, Commands};
    use crate::config::{CONFIG_FILE_NAME, Config};
    use crate::test_support::{TestDir, WorkingDirGuard};
    use serde_json::Value;
    use std::fs;

    #[test]
    fn run_init_creates_project_files_with_defaults() {
        let temp = TestDir::new();
        let project_dir = temp.path().join("demo-project");
        fs::create_dir_all(&project_dir).expect("project dir should exist");
        let _guard = WorkingDirGuard::change_to(&project_dir);

        let output = run(Cli {
            json: false,
            command: Commands::Init,
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
}

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

use crate::app::AppError;
use crate::output::ErrorCode;
use crossterm::cursor::{Hide, Show};
use crossterm::execute;
use crossterm::terminal::{self, EnterAlternateScreen, LeaveAlternateScreen};
use std::env;
use std::io::{self, Write};
use std::path::Path;
use std::process::{Command, ExitStatus};

pub fn open_editor(path: &Path) -> Result<(), AppError> {
    let mut command = editor_command(path)?;
    let _guard = SuspendedTerminal::suspend()?;
    let status = command.status().map_err(|error| {
        AppError::new(
            ErrorCode::FileError,
            format!("failed to launch editor: {error}"),
        )
    })?;

    if status.success() {
        Ok(())
    } else {
        Err(AppError::new(
            ErrorCode::FileError,
            format!("editor exited with {}", format_exit_status(status)),
        ))
    }
}

fn editor_command(path: &Path) -> Result<Command, AppError> {
    let mut parts = editor_parts()?;
    let program = parts.remove(0);
    let mut command = Command::new(program);
    command.args(parts);
    command.arg(path);
    Ok(command)
}

fn editor_parts() -> Result<Vec<String>, AppError> {
    resolve_editor_parts(env::var("VISUAL").ok(), env::var("EDITOR").ok())
}

fn resolve_editor_parts(
    visual: Option<String>,
    editor: Option<String>,
) -> Result<Vec<String>, AppError> {
    let spec = visual
        .filter(|value| !value.trim().is_empty())
        .or_else(|| editor.filter(|value| !value.trim().is_empty()))
        .unwrap_or_else(|| "vi".to_string());

    let parts = shell_words::split(&spec).map_err(|error| {
        AppError::new(
            ErrorCode::FileError,
            format!("failed to parse editor command `{spec}`: {error}"),
        )
    })?;

    if parts.is_empty() {
        return Err(AppError::new(
            ErrorCode::FileError,
            "editor command is empty",
        ));
    }

    Ok(parts)
}

fn format_exit_status(status: ExitStatus) -> String {
    status
        .code()
        .map(|code| format!("status {code}"))
        .unwrap_or_else(|| status.to_string())
}

struct SuspendedTerminal {
    raw_disabled: bool,
    left_alternate_screen: bool,
}

impl SuspendedTerminal {
    fn suspend() -> Result<Self, AppError> {
        let mut guard = Self {
            raw_disabled: false,
            left_alternate_screen: false,
        };

        terminal::disable_raw_mode().map_err(|error| {
            AppError::new(
                ErrorCode::FileError,
                format!("failed to disable raw mode: {error}"),
            )
        })?;
        guard.raw_disabled = true;

        let mut stdout = io::stdout();
        execute!(stdout, Show, LeaveAlternateScreen).map_err(|error| {
            AppError::new(
                ErrorCode::FileError,
                format!("failed to leave alternate screen: {error}"),
            )
        })?;
        stdout.flush().map_err(|error| {
            AppError::new(
                ErrorCode::FileError,
                format!("failed to flush terminal output: {error}"),
            )
        })?;
        guard.left_alternate_screen = true;

        Ok(guard)
    }
}

impl Drop for SuspendedTerminal {
    fn drop(&mut self) {
        if self.raw_disabled {
            let _ = terminal::enable_raw_mode();
        }

        let mut stdout = io::stdout();
        if self.left_alternate_screen {
            let _ = execute!(stdout, EnterAlternateScreen, Hide);
            let _ = stdout.flush();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{format_exit_status, resolve_editor_parts};
    use std::process::Command;

    #[test]
    fn format_exit_status_includes_numeric_code() {
        let status = Command::new("sh")
            .arg("-c")
            .arg("exit 7")
            .status()
            .expect("shell should run");

        assert_eq!(format_exit_status(status), "status 7");
    }

    #[test]
    fn editor_parts_defaults_to_vi_when_env_is_missing() {
        let parts = resolve_editor_parts(None, None).expect("default editor should resolve");
        assert_eq!(parts, vec!["vi".to_string()]);
    }

    #[test]
    fn editor_parts_prefers_visual_over_editor() {
        let parts = resolve_editor_parts(Some("code --wait".to_string()), Some("vim".to_string()))
            .expect("editor should parse");
        assert_eq!(parts, vec!["code".to_string(), "--wait".to_string()]);
    }

    #[test]
    fn editor_parts_parses_shell_quoting() {
        let parts = resolve_editor_parts(Some("'emacs client' --tty".to_string()), None)
            .expect("quoted command should parse");
        assert_eq!(parts, vec!["emacs client".to_string(), "--tty".to_string()]);
    }
}

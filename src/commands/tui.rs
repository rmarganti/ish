use crate::app::{AppContext, AppError};

pub(crate) fn tui_command(json: bool) -> Result<Option<String>, AppError> {
    if json {
        return Err(AppError::new(
            crate::output::ErrorCode::Validation,
            "`ish tui` does not support --json",
        ));
    }

    let context = AppContext::load()?;
    crate::tui::run(context)?;
    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::tui_command;
    use crate::config::Config;
    use crate::test_support::{TestDir, WorkingDirGuard};
    use std::fs;

    #[test]
    fn tui_command_requires_non_json_mode() {
        let err = tui_command(true).expect_err("json mode should be rejected");

        assert!(err.message.contains("does not support --json"));
    }

    #[test]
    fn tui_command_loads_context_and_returns_without_output() {
        let temp = TestDir::new();
        let config = Config::default();
        config.save(temp.path()).expect("config should save");
        fs::create_dir_all(temp.path().join(".ish")).expect("store dir should exist");
        let _guard = WorkingDirGuard::change_to(temp.path());

        let output = tui_command(false).expect("tui command should succeed");

        assert!(output.is_none());
    }
}

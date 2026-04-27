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

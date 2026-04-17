use crate::core::store::StoreError;
use crate::output::ErrorCode;
use std::path::Path;
use std::process::ExitCode;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppError {
    pub code: ErrorCode,
    pub message: String,
}

pub struct RunOutcome {
    pub output: Option<String>,
    pub exit_code: ExitCode,
}

impl AppError {
    pub fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }
}

pub fn success_outcome(output: Option<String>) -> RunOutcome {
    RunOutcome {
        output,
        exit_code: ExitCode::SUCCESS,
    }
}

pub fn classify_app_error(message: String) -> AppError {
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

pub fn store_open_error(store_root: &Path) -> impl Fn(StoreError) -> AppError + '_ {
    move |error| {
        AppError::new(
            ErrorCode::FileError,
            format!("failed to open store `{}`: {error}", store_root.display()),
        )
    }
}

pub fn store_app_error(error: StoreError) -> AppError {
    let code = match error {
        StoreError::InvalidStatus(_)
        | StoreError::InvalidType(_)
        | StoreError::InvalidPriority(_)
        | StoreError::InvalidTag(_)
        | StoreError::ParentNotAllowed(_)
        | StoreError::InvalidParentType { .. }
        | StoreError::Body(_) => ErrorCode::Validation,
        StoreError::NotFound(_) => ErrorCode::NotFound,
        StoreError::ETagMismatch { .. } => ErrorCode::Conflict,
        StoreError::Io(_)
        | StoreError::InvalidPath(_)
        | StoreError::InvalidFrontmatter(_)
        | StoreError::Yaml { .. } => ErrorCode::FileError,
    };

    AppError::new(code, error.to_string())
}

pub fn json_output_error(message: String) -> AppError {
    AppError::new(ErrorCode::FileError, message)
}

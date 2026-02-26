use thiserror::Error;

#[derive(Debug, Error)]
pub enum StorageError {
    #[error("issue not found: {0}")]
    NotFound(String),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("file not found: {0}")]
    FileNotFound(String),
    #[error("parse error: {0}")]
    ParseError(String),
    #[error("write error: {0}")]
    WriteError(String),
    #[error("lock error: {0}")]
    LockError(String),
}

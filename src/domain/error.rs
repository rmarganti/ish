use thiserror::Error;

#[derive(Debug, Error)]
pub enum DomainError {
    #[error("invalid status: {0}")]
    InvalidStatus(String),
    #[error("issue not found: {0}")]
    IssueNotFound(String),
    #[error("validation error: {0}")]
    ValidationError(String),
}

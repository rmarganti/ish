mod error;
mod repository;

pub use error::StorageError;
pub use repository::{IssueRepository, SqliteRepository};

use crate::{
    domain::{Issue, Status},
    storage::StorageError,
};

pub trait IssueRepository: Send + Sync {
    fn create(&self, issue: &Issue) -> Result<(), StorageError>;
    fn get_by_id(&self, id: &str) -> Result<Issue, StorageError>;
    fn get_all(&self) -> Result<Vec<Issue>, StorageError>;
    fn get_by_status(&self, status: Status) -> Result<Vec<Issue>, StorageError>;
    fn get_by_parent(&self, parent_id: Option<&str>) -> Result<Vec<Issue>, StorageError>;
    fn get_next_todo(&self) -> Result<Option<Issue>, StorageError>;
    fn update(&self, issue: &Issue) -> Result<(), StorageError>;
    fn delete(&self, id: &str) -> Result<(), StorageError>;
    fn clear_all(&self) -> Result<(), StorageError>;
}

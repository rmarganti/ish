use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::{DomainError, Status};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Issue {
    pub id: String,
    pub title: String,
    pub body: Option<String>,
    pub status: Status,
    pub sort: i32,
    pub parent_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListIssue {
    pub id: String,
    pub title: String,
    pub status: Status,
    pub sort: i32,
    pub parent_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<Issue> for ListIssue {
    fn from(issue: Issue) -> Self {
        ListIssue {
            id: issue.id,
            title: issue.title,
            status: issue.status,
            sort: issue.sort,
            parent_id: issue.parent_id,
            created_at: issue.created_at,
            updated_at: issue.updated_at,
        }
    }
}

impl Issue {
    pub fn new(title: String, body: Option<String>, parent_id: Option<String>) -> Self {
        let now = Utc::now();
        let id = generate_id();
        Self {
            id,
            title,
            body,
            status: Status::Todo,
            sort: 0,
            parent_id,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn start(&mut self) -> Result<(), DomainError> {
        if self.status == Status::Done {
            return Err(DomainError::ValidationError(
                "Cannot start a completed issue".to_string(),
            ));
        }
        self.status = Status::InProgress;
        self.updated_at = Utc::now();
        Ok(())
    }

    pub fn finish(&mut self) -> Result<(), DomainError> {
        self.status = Status::Done;
        self.updated_at = Utc::now();
        Ok(())
    }

    pub fn update_title(&mut self, title: String) {
        self.title = title;
        self.updated_at = Utc::now();
    }

    pub fn update_body(&mut self, body: Option<String>) {
        self.body = body;
        self.updated_at = Utc::now();
    }

    pub fn update_sort(&mut self, sort: i32) {
        self.sort = sort;
        self.updated_at = Utc::now();
    }
}

fn generate_id() -> String {
    use uuid::Uuid;
    let uuid = Uuid::new_v4();
    let bytes = uuid.as_bytes();
    hex::encode(&bytes[0..4])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_issue() {
        let issue = Issue::new(
            "Test issue".to_string(),
            Some("Test body".to_string()),
            None,
        );
        assert_eq!(issue.title, "Test issue");
        assert_eq!(issue.body, Some("Test body".to_string()));
        assert_eq!(issue.status, Status::Todo);
        assert!(issue.parent_id.is_none());
    }

    #[test]
    fn test_start_issue() {
        let mut issue = Issue::new("Test".to_string(), None, None);
        issue.start().unwrap();
        assert_eq!(issue.status, Status::InProgress);
    }

    #[test]
    fn test_finish_issue() {
        let mut issue = Issue::new("Test".to_string(), None, None);
        issue.finish().unwrap();
        assert_eq!(issue.status, Status::Done);
    }

    #[test]
    fn test_cannot_start_finished_issue() {
        let mut issue = Issue::new("Test".to_string(), None, None);
        issue.finish().unwrap();
        assert!(issue.start().is_err());
    }

    #[test]
    fn test_id_length() {
        let issue = Issue::new("Test".to_string(), None, None);
        assert_eq!(issue.id.len(), 8);
    }
}

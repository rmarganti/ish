use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::{DomainError, Status};

// ----------------------------------------------------------------
// Issue
// ----------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Issue {
    pub id: String,
    pub title: String,
    pub body: Option<String>,
    pub context: Option<String>,
    pub status: Status,
    pub sort: i32,
    pub parent_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Issue {
    pub fn new(
        title: String,
        body: Option<String>,
        context: Option<String>,
        parent_id: Option<String>,
    ) -> Self {
        let now = Utc::now();
        let id = generate_id();
        Self {
            id,
            title,
            body,
            context,
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

    pub fn update_context(&mut self, context: Option<String>) {
        self.context = context;
        self.updated_at = Utc::now();
    }

    pub fn update_sort(&mut self, sort: i32) {
        self.sort = sort;
        self.updated_at = Utc::now();
    }
}

// ----------------------------------------------------------------
// ListIssue
// ----------------------------------------------------------------

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

// ----------------------------------------------------------------
// ContextEntry
// ----------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextEntry {
    pub depth: usize,
    pub issue_id: String,
    pub title: String,
    pub context: String,
}

// ----------------------------------------------------------------
// ShowIssue
// ----------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShowIssue {
    pub id: String,
    pub title: String,
    pub body: Option<String>,
    pub context: Vec<ContextEntry>,
    pub status: Status,
    pub sort: i32,
    pub parent_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl ShowIssue {
    pub fn from_issue(issue: Issue, context: Vec<ContextEntry>) -> Self {
        ShowIssue {
            id: issue.id,
            title: issue.title,
            body: issue.body,
            context,
            status: issue.status,
            sort: issue.sort,
            parent_id: issue.parent_id,
            created_at: issue.created_at,
            updated_at: issue.updated_at,
        }
    }
}

// ----------------------------------------------------------------
// Ancestor context assembly
// ----------------------------------------------------------------

const MAX_ANCESTOR_DEPTH: usize = 100;

pub fn collect_ancestor_context(
    issue: &Issue,
    repo: &dyn crate::storage::IssueRepository,
) -> Result<Vec<ContextEntry>, crate::storage::StorageError> {
    let mut entries = Vec::new();

    // Depth 0: the issue itself
    if let Some(ref ctx) = issue.context {
        entries.push(ContextEntry {
            depth: 0,
            issue_id: issue.id.clone(),
            title: issue.title.clone(),
            context: ctx.clone(),
        });
    }

    let mut current_parent_id = issue.parent_id.clone();
    let mut depth: usize = 0;

    while let Some(pid) = current_parent_id {
        depth += 1;
        if depth > MAX_ANCESTOR_DEPTH {
            break;
        }

        let parent = repo.get_by_id(&pid)?;

        if let Some(ref ctx) = parent.context {
            entries.push(ContextEntry {
                depth,
                issue_id: parent.id.clone(),
                title: parent.title.clone(),
                context: ctx.clone(),
            });
        }

        current_parent_id = parent.parent_id.clone();
    }

    Ok(entries)
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
    use crate::storage::{IssueRepository, JSONLRepository};

    #[test]
    fn test_create_issue() {
        let issue = Issue::new(
            "Test issue".to_string(),
            Some("Test body".to_string()),
            Some("Test context".to_string()),
            None,
        );
        assert_eq!(issue.title, "Test issue");
        assert_eq!(issue.body, Some("Test body".to_string()));
        assert_eq!(issue.context, Some("Test context".to_string()));
        assert_eq!(issue.status, Status::Todo);
        assert!(issue.parent_id.is_none());
    }

    #[test]
    fn test_create_issue_without_context() {
        let issue = Issue::new(
            "Test issue".to_string(),
            Some("Test body".to_string()),
            None,
            None,
        );
        assert_eq!(issue.context, None);
    }

    #[test]
    fn test_update_context() {
        let mut issue = Issue::new("Test".to_string(), None, None, None);
        issue.update_context(Some("New context".to_string()));
        assert_eq!(issue.context, Some("New context".to_string()));
    }

    #[test]
    fn test_start_issue() {
        let mut issue = Issue::new("Test".to_string(), None, None, None);
        issue.start().unwrap();
        assert_eq!(issue.status, Status::InProgress);
    }

    #[test]
    fn test_finish_issue() {
        let mut issue = Issue::new("Test".to_string(), None, None, None);
        issue.finish().unwrap();
        assert_eq!(issue.status, Status::Done);
    }

    #[test]
    fn test_cannot_start_finished_issue() {
        let mut issue = Issue::new("Test".to_string(), None, None, None);
        issue.finish().unwrap();
        assert!(issue.start().is_err());
    }

    #[test]
    fn test_id_length() {
        let issue = Issue::new("Test".to_string(), None, None, None);
        assert_eq!(issue.id.len(), 8);
    }

    #[test]
    fn test_list_issue_excludes_context() {
        let issue = Issue::new(
            "Test issue".to_string(),
            Some("Test body".to_string()),
            Some("Test context".to_string()),
            None,
        );
        let list_issue = ListIssue::from(issue);
        assert_eq!(list_issue.title, "Test issue");
    }

    // ---- Ancestor context tests ----

    #[test]
    fn test_collect_context_no_parent_with_context() {
        let repo = JSONLRepository::in_memory();
        let issue = Issue::new(
            "Leaf".to_string(),
            None,
            Some("Leaf context".to_string()),
            None,
        );
        repo.create(&issue).unwrap();

        let entries = collect_ancestor_context(&issue, &repo).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].depth, 0);
        assert_eq!(entries[0].issue_id, issue.id);
        assert_eq!(entries[0].context, "Leaf context");
    }

    #[test]
    fn test_collect_context_no_parent_without_context() {
        let repo = JSONLRepository::in_memory();
        let issue = Issue::new("Leaf".to_string(), None, None, None);
        repo.create(&issue).unwrap();

        let entries = collect_ancestor_context(&issue, &repo).unwrap();
        assert!(entries.is_empty());
    }

    #[test]
    fn test_collect_context_with_parent() {
        let repo = JSONLRepository::in_memory();
        let parent = Issue::new(
            "Parent".to_string(),
            None,
            Some("Parent context".to_string()),
            None,
        );
        repo.create(&parent).unwrap();

        let child = Issue::new(
            "Child".to_string(),
            None,
            Some("Child context".to_string()),
            Some(parent.id.clone()),
        );
        repo.create(&child).unwrap();

        let entries = collect_ancestor_context(&child, &repo).unwrap();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].depth, 0);
        assert_eq!(entries[0].issue_id, child.id);
        assert_eq!(entries[1].depth, 1);
        assert_eq!(entries[1].issue_id, parent.id);
    }

    #[test]
    fn test_collect_context_deep_chain() {
        let repo = JSONLRepository::in_memory();
        let grandparent = Issue::new(
            "Grandparent".to_string(),
            None,
            Some("GP context".to_string()),
            None,
        );
        repo.create(&grandparent).unwrap();

        let parent = Issue::new(
            "Parent".to_string(),
            None,
            Some("Parent context".to_string()),
            Some(grandparent.id.clone()),
        );
        repo.create(&parent).unwrap();

        let child = Issue::new(
            "Child".to_string(),
            None,
            Some("Child context".to_string()),
            Some(parent.id.clone()),
        );
        repo.create(&child).unwrap();

        let entries = collect_ancestor_context(&child, &repo).unwrap();
        assert_eq!(entries.len(), 3);
        assert_eq!(entries[0].depth, 0);
        assert_eq!(entries[1].depth, 1);
        assert_eq!(entries[2].depth, 2);
        assert_eq!(entries[2].issue_id, grandparent.id);
    }

    #[test]
    fn test_collect_context_skips_ancestors_without_context() {
        let repo = JSONLRepository::in_memory();
        let grandparent = Issue::new(
            "Grandparent".to_string(),
            None,
            Some("GP context".to_string()),
            None,
        );
        repo.create(&grandparent).unwrap();

        // Parent has no context
        let parent = Issue::new(
            "Parent".to_string(),
            None,
            None,
            Some(grandparent.id.clone()),
        );
        repo.create(&parent).unwrap();

        let child = Issue::new(
            "Child".to_string(),
            None,
            Some("Child context".to_string()),
            Some(parent.id.clone()),
        );
        repo.create(&child).unwrap();

        let entries = collect_ancestor_context(&child, &repo).unwrap();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].depth, 0);
        assert_eq!(entries[0].issue_id, child.id);
        // Depth 1 (parent) is skipped; grandparent is at depth 2
        assert_eq!(entries[1].depth, 2);
        assert_eq!(entries[1].issue_id, grandparent.id);
    }

    #[test]
    fn test_collect_context_all_ancestors_no_context() {
        let repo = JSONLRepository::in_memory();
        let parent = Issue::new("Parent".to_string(), None, None, None);
        repo.create(&parent).unwrap();

        let child = Issue::new("Child".to_string(), None, None, Some(parent.id.clone()));
        repo.create(&child).unwrap();

        let entries = collect_ancestor_context(&child, &repo).unwrap();
        assert!(entries.is_empty());
    }

    #[test]
    fn test_collect_context_cycle_protection() {
        let repo = JSONLRepository::in_memory();

        // Create two issues that point to each other (simulated cycle)
        let mut issue_a = Issue::new("A".to_string(), None, Some("A ctx".to_string()), None);
        let issue_b = Issue::new(
            "B".to_string(),
            None,
            Some("B ctx".to_string()),
            Some(issue_a.id.clone()),
        );

        // Create A first, then B
        repo.create(&issue_a).unwrap();
        repo.create(&issue_b).unwrap();

        // Now make A point to B (creating a cycle)
        issue_a.parent_id = Some(issue_b.id.clone());
        repo.update(&issue_a).unwrap();

        // Should not hang — depth limit kicks in
        let entries = collect_ancestor_context(&issue_a, &repo).unwrap();
        assert!(entries.len() <= 101); // self + up to MAX_ANCESTOR_DEPTH ancestors
    }

    #[test]
    fn test_show_issue_serialization() {
        let issue = Issue::new("Test".to_string(), Some("Body".to_string()), None, None);
        let context = vec![ContextEntry {
            depth: 0,
            issue_id: issue.id.clone(),
            title: issue.title.clone(),
            context: "Some context".to_string(),
        }];
        let show_issue = ShowIssue::from_issue(issue, context);
        let json: serde_json::Value = serde_json::to_value(&show_issue).unwrap();

        assert!(json["context"].is_array());
        assert_eq!(json["context"][0]["depth"], 0);
        assert_eq!(json["context"][0]["context"], "Some context");
    }
}

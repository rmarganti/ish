use std::io::{BufRead, Write};
use std::path::PathBuf;
use std::sync::Mutex;

use crate::domain::{Issue, Status};
use crate::storage::error::StorageError;
use crate::storage::issue_repository::IssueRepository;

/// A repository that stores issues in a JSONL (JSON Lines) file.
/// Each line is a complete JSON object representing a single issue.
/// The file is always sorted by issue ID (ascending) for deterministic ordering.
#[derive(Debug)]
pub struct JSONLRepository {
    path: Option<PathBuf>,
    issues: Mutex<Vec<Issue>>,
}

impl JSONLRepository {
    /// Open (or create) a JSONL repository at the given path.
    pub fn new(path: PathBuf) -> Result<Self, StorageError> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let issues = if path.exists() {
            Self::load_from_file(&path)?
        } else {
            Vec::new()
        };

        Ok(Self {
            path: Some(path),
            issues: Mutex::new(issues),
        })
    }

    /// Create an in-memory repository (for testing).
    pub fn in_memory() -> Self {
        Self {
            path: None,
            issues: Mutex::new(Vec::new()),
        }
    }

    fn load_from_file(path: &PathBuf) -> Result<Vec<Issue>, StorageError> {
        let file = std::fs::File::open(path)?;
        let reader = std::io::BufReader::new(file);
        let mut issues = Vec::new();

        for (line_num, line) in reader.lines().enumerate() {
            let line = line?;
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            let issue: Issue = serde_json::from_str(trimmed)
                .map_err(|e| StorageError::ParseError(format!("line {}: {}", line_num + 1, e)))?;
            issues.push(issue);
        }

        Ok(issues)
    }

    fn persist(&self, issues: &[Issue]) -> Result<(), StorageError> {
        let Some(path) = &self.path else {
            // In-memory mode: no persistence needed
            return Ok(());
        };

        let tmp_path = path.with_extension("jsonl.tmp");

        {
            let file = std::fs::File::create(&tmp_path)
                .map_err(|e| StorageError::WriteError(e.to_string()))?;
            let mut writer = std::io::BufWriter::new(file);

            for issue in issues {
                let line = serde_json::to_string(issue)
                    .map_err(|e| StorageError::WriteError(e.to_string()))?;
                writeln!(writer, "{}", line)
                    .map_err(|e| StorageError::WriteError(e.to_string()))?;
            }

            writer
                .flush()
                .map_err(|e| StorageError::WriteError(e.to_string()))?;
        }

        std::fs::rename(&tmp_path, path).map_err(|e| StorageError::WriteError(e.to_string()))?;

        Ok(())
    }

    fn sort_issues(issues: &mut Vec<Issue>) {
        issues.sort_by(|a, b| a.id.cmp(&b.id));
    }
}

impl IssueRepository for JSONLRepository {
    fn create(&self, issue: &Issue) -> Result<(), StorageError> {
        let mut issues = self
            .issues
            .lock()
            .map_err(|e| StorageError::LockError(e.to_string()))?;

        if issues.iter().any(|i| i.id == issue.id) {
            return Err(StorageError::WriteError(format!(
                "issue with id '{}' already exists",
                issue.id
            )));
        }

        issues.push(issue.clone());
        Self::sort_issues(&mut issues);
        self.persist(&issues)?;
        Ok(())
    }

    fn get_by_id(&self, id: &str) -> Result<Issue, StorageError> {
        let issues = self
            .issues
            .lock()
            .map_err(|e| StorageError::LockError(e.to_string()))?;

        issues
            .iter()
            .find(|i| i.id == id)
            .cloned()
            .ok_or_else(|| StorageError::NotFound(id.to_string()))
    }

    fn get_all(&self) -> Result<Vec<Issue>, StorageError> {
        let issues = self
            .issues
            .lock()
            .map_err(|e| StorageError::LockError(e.to_string()))?;

        // Return sorted by sort asc, created_at desc (matching SQLite behaviour)
        let mut result = issues.clone();
        result.sort_by(|a, b| {
            a.sort
                .cmp(&b.sort)
                .then_with(|| b.created_at.cmp(&a.created_at))
        });
        Ok(result)
    }

    fn get_by_status(&self, status: Status) -> Result<Vec<Issue>, StorageError> {
        let issues = self
            .issues
            .lock()
            .map_err(|e| StorageError::LockError(e.to_string()))?;

        let mut result: Vec<Issue> = issues
            .iter()
            .filter(|i| i.status == status)
            .cloned()
            .collect();
        result.sort_by(|a, b| {
            a.sort
                .cmp(&b.sort)
                .then_with(|| b.created_at.cmp(&a.created_at))
        });
        Ok(result)
    }

    fn get_by_parent(&self, parent_id: Option<&str>) -> Result<Vec<Issue>, StorageError> {
        let issues = self
            .issues
            .lock()
            .map_err(|e| StorageError::LockError(e.to_string()))?;

        let mut result: Vec<Issue> = issues
            .iter()
            .filter(|i| i.parent_id.as_deref() == parent_id)
            .cloned()
            .collect();
        result.sort_by(|a, b| {
            a.sort
                .cmp(&b.sort)
                .then_with(|| b.created_at.cmp(&a.created_at))
        });
        Ok(result)
    }

    fn get_next_todo(&self) -> Result<Option<Issue>, StorageError> {
        let issues = self
            .issues
            .lock()
            .map_err(|e| StorageError::LockError(e.to_string()))?;

        // IDs of issues that are parents of at least one non-done child
        let blocked_parent_ids: std::collections::HashSet<String> = issues
            .iter()
            .filter(|i| i.status != Status::Done)
            .filter_map(|i| i.parent_id.clone())
            .collect();

        let mut candidates: Vec<&Issue> = issues
            .iter()
            .filter(|i| i.status == Status::Todo)
            .filter(|i| !blocked_parent_ids.contains(&i.id))
            .collect();

        candidates.sort_by(|a, b| {
            a.sort
                .cmp(&b.sort)
                .then_with(|| a.created_at.cmp(&b.created_at))
        });

        Ok(candidates.into_iter().next().cloned())
    }

    fn update(&self, issue: &Issue) -> Result<(), StorageError> {
        let mut issues = self
            .issues
            .lock()
            .map_err(|e| StorageError::LockError(e.to_string()))?;

        let pos = issues
            .iter()
            .position(|i| i.id == issue.id)
            .ok_or_else(|| StorageError::NotFound(issue.id.clone()))?;

        issues[pos] = issue.clone();
        // ID doesn't change on update so sort order is preserved, but re-sort to be safe
        Self::sort_issues(&mut issues);
        self.persist(&issues)?;
        Ok(())
    }

    fn delete(&self, id: &str) -> Result<(), StorageError> {
        let mut issues = self
            .issues
            .lock()
            .map_err(|e| StorageError::LockError(e.to_string()))?;

        let pos = issues
            .iter()
            .position(|i| i.id == id)
            .ok_or_else(|| StorageError::NotFound(id.to_string()))?;

        issues.remove(pos);
        self.persist(&issues)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn create_test_repo() -> JSONLRepository {
        JSONLRepository::in_memory()
    }

    // ---- basic CRUD ----

    #[test]
    fn test_create_and_get_issue() {
        let repo = create_test_repo();
        let issue = Issue::new("Test".to_string(), Some("Body".to_string()), None);
        repo.create(&issue).unwrap();

        let retrieved = repo.get_by_id(&issue.id).unwrap();
        assert_eq!(retrieved.id, issue.id);
        assert_eq!(retrieved.title, "Test");
        assert_eq!(retrieved.body, Some("Body".to_string()));
    }

    #[test]
    fn test_get_all_issues() {
        let repo = create_test_repo();
        let issue1 = Issue::new("First".to_string(), None, None);
        let issue2 = Issue::new("Second".to_string(), None, None);
        repo.create(&issue1).unwrap();
        repo.create(&issue2).unwrap();

        let issues = repo.get_all().unwrap();
        assert_eq!(issues.len(), 2);
    }

    #[test]
    fn test_get_by_status() {
        let repo = create_test_repo();
        let mut issue = Issue::new("Test".to_string(), None, None);
        repo.create(&issue).unwrap();

        let todo_issues = repo.get_by_status(Status::Todo).unwrap();
        assert_eq!(todo_issues.len(), 1);

        issue.start().unwrap();
        repo.update(&issue).unwrap();

        let in_progress = repo.get_by_status(Status::InProgress).unwrap();
        assert_eq!(in_progress.len(), 1);

        let todo = repo.get_by_status(Status::Todo).unwrap();
        assert!(todo.is_empty());
    }

    #[test]
    fn test_get_next_todo() {
        let repo = create_test_repo();
        let mut issue1 = Issue::new("First".to_string(), None, None);
        issue1.sort = 10;
        let mut issue2 = Issue::new("Second".to_string(), None, None);
        issue2.sort = 5;
        repo.create(&issue1).unwrap();
        repo.create(&issue2).unwrap();

        let next = repo.get_next_todo().unwrap();
        assert!(next.is_some());
        assert_eq!(next.unwrap().title, "Second");
    }

    #[test]
    fn test_update_issue() {
        let repo = create_test_repo();
        let mut issue = Issue::new("Test".to_string(), None, None);
        repo.create(&issue).unwrap();

        issue.update_title("Updated".to_string());
        repo.update(&issue).unwrap();

        let retrieved = repo.get_by_id(&issue.id).unwrap();
        assert_eq!(retrieved.title, "Updated");
    }

    #[test]
    fn test_delete_issue() {
        let repo = create_test_repo();
        let issue = Issue::new("Test".to_string(), None, None);
        repo.create(&issue).unwrap();

        repo.delete(&issue.id).unwrap();

        let result = repo.get_by_id(&issue.id);
        assert!(result.is_err());
    }

    #[test]
    fn test_get_next_todo_skips_parent_with_incomplete_children() {
        let repo = create_test_repo();

        let parent = Issue::new("Parent".to_string(), None, None);
        repo.create(&parent).unwrap();

        let child = Issue::new("Child".to_string(), None, Some(parent.id.clone()));
        repo.create(&child).unwrap();

        let next = repo.get_next_todo().unwrap();
        assert!(next.is_some());
        assert_eq!(next.unwrap().title, "Child");
    }

    #[test]
    fn test_get_next_todo_includes_parent_with_done_children() {
        let repo = create_test_repo();

        let parent = Issue::new("Parent".to_string(), None, None);
        repo.create(&parent).unwrap();

        let mut child = Issue::new("Child".to_string(), None, Some(parent.id.clone()));
        repo.create(&child).unwrap();
        child.finish().unwrap();
        repo.update(&child).unwrap();

        let next = repo.get_next_todo().unwrap();
        assert!(next.is_some());
        assert_eq!(next.unwrap().title, "Parent");
    }

    #[test]
    fn test_get_next_todo_includes_parent_without_children() {
        let repo = create_test_repo();

        let parent = Issue::new("Parent".to_string(), None, None);
        repo.create(&parent).unwrap();

        let next = repo.get_next_todo().unwrap();
        assert!(next.is_some());
        assert_eq!(next.unwrap().title, "Parent");
    }

    #[test]
    fn test_parent_child_issues() {
        let repo = create_test_repo();
        let parent = Issue::new("Parent".to_string(), None, None);
        repo.create(&parent).unwrap();

        let child = Issue::new("Child".to_string(), None, Some(parent.id.clone()));
        repo.create(&child).unwrap();

        let children = repo.get_by_parent(Some(&parent.id)).unwrap();
        assert_eq!(children.len(), 1);
        assert_eq!(children[0].title, "Child");
    }

    // ---- JSONL-specific behaviour ----

    #[test]
    fn test_file_created_on_first_write() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("issues.jsonl");

        assert!(!path.exists());

        let repo = JSONLRepository::new(path.clone()).unwrap();
        let issue = Issue::new("Test".to_string(), None, None);
        repo.create(&issue).unwrap();

        assert!(path.exists());
    }

    #[test]
    fn test_file_sorted_by_id_after_create() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("issues.jsonl");
        let repo = JSONLRepository::new(path.clone()).unwrap();

        for i in 0..5 {
            let issue = Issue::new(format!("Issue {}", i), None, None);
            repo.create(&issue).unwrap();
        }

        // Re-read file and verify IDs are sorted
        let content = std::fs::read_to_string(&path).unwrap();
        let ids: Vec<String> = content
            .lines()
            .filter(|l| !l.trim().is_empty())
            .map(|l| {
                let v: serde_json::Value = serde_json::from_str(l).unwrap();
                v["id"].as_str().unwrap().to_string()
            })
            .collect();

        let mut sorted = ids.clone();
        sorted.sort();
        assert_eq!(ids, sorted);
    }

    #[test]
    fn test_file_sorted_by_id_after_delete() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("issues.jsonl");
        let repo = JSONLRepository::new(path.clone()).unwrap();

        let issues: Vec<Issue> = (0..5)
            .map(|i| Issue::new(format!("Issue {}", i), None, None))
            .collect();

        for issue in &issues {
            repo.create(issue).unwrap();
        }

        // Delete the middle issue
        repo.delete(&issues[2].id).unwrap();

        let content = std::fs::read_to_string(&path).unwrap();
        let ids: Vec<String> = content
            .lines()
            .filter(|l| !l.trim().is_empty())
            .map(|l| {
                let v: serde_json::Value = serde_json::from_str(l).unwrap();
                v["id"].as_str().unwrap().to_string()
            })
            .collect();

        let mut sorted = ids.clone();
        sorted.sort();
        assert_eq!(ids, sorted);
        assert_eq!(ids.len(), 4);
    }

    #[test]
    fn test_persistence_across_reopen() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("issues.jsonl");

        {
            let repo = JSONLRepository::new(path.clone()).unwrap();
            let issue = Issue::new("Persistent".to_string(), None, None);
            repo.create(&issue).unwrap();
        }

        // Re-open and verify data is still there
        let repo2 = JSONLRepository::new(path).unwrap();
        let issues = repo2.get_all().unwrap();
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].title, "Persistent");
    }

    #[test]
    fn test_parse_error_on_malformed_file() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("issues.jsonl");
        std::fs::write(&path, "not valid json\n").unwrap();

        let result = JSONLRepository::new(path);
        assert!(result.is_err());
        match result.unwrap_err() {
            StorageError::ParseError(_) => {}
            e => panic!("expected ParseError, got {:?}", e),
        }
    }

    #[test]
    fn test_empty_file_is_valid() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("issues.jsonl");
        std::fs::write(&path, "").unwrap();

        let repo = JSONLRepository::new(path).unwrap();
        assert_eq!(repo.get_all().unwrap().len(), 0);
    }

    #[test]
    fn test_not_found_error_on_missing_id() {
        let repo = create_test_repo();
        let result = repo.get_by_id("nonexistent");
        assert!(result.is_err());
        match result.unwrap_err() {
            StorageError::NotFound(_) => {}
            e => panic!("expected NotFound, got {:?}", e),
        }
    }

    #[test]
    fn test_update_not_found() {
        let repo = create_test_repo();
        let issue = Issue::new("Ghost".to_string(), None, None);
        let result = repo.update(&issue);
        assert!(result.is_err());
        match result.unwrap_err() {
            StorageError::NotFound(_) => {}
            e => panic!("expected NotFound, got {:?}", e),
        }
    }

    #[test]
    fn test_delete_not_found() {
        let repo = create_test_repo();
        let result = repo.delete("nonexistent");
        assert!(result.is_err());
        match result.unwrap_err() {
            StorageError::NotFound(_) => {}
            e => panic!("expected NotFound, got {:?}", e),
        }
    }
}

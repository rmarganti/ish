use chrono::{DateTime, Utc};
use rusqlite::{params, Connection};
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Mutex;

use crate::domain::{Issue, Status};
use crate::storage::error::StorageError;
use crate::storage::IssueRepository;

pub struct SqliteRepository {
    conn: Mutex<Connection>,
}

impl SqliteRepository {
    pub fn new(db_path: PathBuf) -> Result<Self, StorageError> {
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let conn = Connection::open(&db_path)?;
        let repo = Self {
            conn: Mutex::new(conn),
        };
        repo.run_migrations()?;
        Ok(repo)
    }

    pub fn in_memory() -> Result<Self, StorageError> {
        let conn = Connection::open_in_memory()?;
        let repo = Self {
            conn: Mutex::new(conn),
        };
        repo.run_migrations()?;
        Ok(repo)
    }

    fn run_migrations(&self) -> Result<(), StorageError> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "CREATE TABLE IF NOT EXISTS issues (
                id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                body TEXT,
                status TEXT NOT NULL DEFAULT 'todo',
                sort INTEGER NOT NULL DEFAULT 0,
                parent_id TEXT REFERENCES issues(id),
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )",
            [],
        )?;
        Ok(())
    }

    fn row_to_issue(row: &rusqlite::Row) -> rusqlite::Result<Issue> {
        let status_str: String = row.get(3)?;
        let created_str: String = row.get(6)?;
        let updated_str: String = row.get(7)?;

        Ok(Issue {
            id: row.get(0)?,
            title: row.get(1)?,
            body: row.get(2)?,
            status: Status::from_str(&status_str).unwrap_or(Status::Todo),
            sort: row.get(4)?,
            parent_id: row.get(5)?,
            created_at: DateTime::parse_from_rfc3339(&created_str)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
            updated_at: DateTime::parse_from_rfc3339(&updated_str)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
        })
    }
}

impl IssueRepository for SqliteRepository {
    fn create(&self, issue: &Issue) -> Result<(), StorageError> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO issues (id, title, body, status, sort, parent_id, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                issue.id,
                issue.title,
                issue.body,
                issue.status.as_str(),
                issue.sort,
                issue.parent_id,
                issue.created_at.to_rfc3339(),
                issue.updated_at.to_rfc3339(),
            ],
        )?;
        Ok(())
    }

    fn get_by_id(&self, id: &str) -> Result<Issue, StorageError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, title, body, status, sort, parent_id, created_at, updated_at
             FROM issues WHERE id = ?1",
        )?;
        let issue = stmt.query_row([id], Self::row_to_issue)?;
        Ok(issue)
    }

    fn get_all(&self) -> Result<Vec<Issue>, StorageError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, title, body, status, sort, parent_id, created_at, updated_at
             FROM issues ORDER BY sort ASC, created_at DESC",
        )?;
        let issues = stmt
            .query_map([], Self::row_to_issue)?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(issues)
    }

    fn get_by_status(&self, status: Status) -> Result<Vec<Issue>, StorageError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, title, body, status, sort, parent_id, created_at, updated_at
             FROM issues WHERE status = ?1 ORDER BY sort ASC, created_at DESC",
        )?;
        let issues = stmt
            .query_map([status.as_str()], Self::row_to_issue)?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(issues)
    }

    fn get_by_parent(&self, parent_id: Option<&str>) -> Result<Vec<Issue>, StorageError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = match parent_id {
            Some(_) => conn.prepare(
                "SELECT id, title, body, status, sort, parent_id, created_at, updated_at
                 FROM issues WHERE parent_id = ?1 ORDER BY sort ASC, created_at DESC",
            )?,
            None => conn.prepare(
                "SELECT id, title, body, status, sort, parent_id, created_at, updated_at
                 FROM issues WHERE parent_id IS NULL ORDER BY sort ASC, created_at DESC",
            )?,
        };

        let issues = match parent_id {
            Some(pid) => stmt.query_map([pid], Self::row_to_issue)?,
            None => stmt.query_map([], Self::row_to_issue)?,
        };

        let issues = issues.collect::<Result<Vec<_>, _>>()?;
        Ok(issues)
    }

    fn get_next_todo(&self) -> Result<Option<Issue>, StorageError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, title, body, status, sort, parent_id, created_at, updated_at
             FROM issues
             WHERE status = 'todo'
             AND id NOT IN (
                 SELECT DISTINCT parent_id
                 FROM issues
                 WHERE parent_id IS NOT NULL AND status != 'done'
             )
             ORDER BY sort ASC, created_at ASC
             LIMIT 1",
        )?;
        let result = stmt.query_row([], Self::row_to_issue);
        match result {
            Ok(issue) => Ok(Some(issue)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    fn update(&self, issue: &Issue) -> Result<(), StorageError> {
        let conn = self.conn.lock().unwrap();
        let rows_affected = conn.execute(
            "UPDATE issues SET title = ?1, body = ?2, status = ?3, sort = ?4,
             parent_id = ?5, updated_at = ?6 WHERE id = ?7",
            params![
                issue.title,
                issue.body,
                issue.status.as_str(),
                issue.sort,
                issue.parent_id,
                issue.updated_at.to_rfc3339(),
                issue.id,
            ],
        )?;
        if rows_affected == 0 {
            return Err(StorageError::NotFound(issue.id.clone()));
        }
        Ok(())
    }

    fn delete(&self, id: &str) -> Result<(), StorageError> {
        let conn = self.conn.lock().unwrap();
        let rows_affected = conn.execute("DELETE FROM issues WHERE id = ?1", [id])?;
        if rows_affected == 0 {
            return Err(StorageError::NotFound(id.to_string()));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_repo() -> SqliteRepository {
        SqliteRepository::in_memory().unwrap()
    }

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
}

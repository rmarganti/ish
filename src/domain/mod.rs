mod error;
mod issue;
mod status;

pub const MAX_NESTING_DEPTH: usize = 3;

pub use error::DomainError;
pub use issue::{collect_ancestor_context, ContextEntry, Issue, ListIssue, ShowIssue};
pub use status::Status;

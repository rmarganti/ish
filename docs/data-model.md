# Data Model

## Issue

The core storage type. Each issue is one line in the JSONL file.

| Field        | Type               | Notes                                |
| ------------ | ------------------ | ------------------------------------ |
| `id`         | `String`           | 8-char hex (first 4 bytes of UUIDv4) |
| `title`      | `String`           | Required                             |
| `body`       | `Option<String>`   | Long-form description                |
| `context`    | `Option<String>`   | Reference material / notes           |
| `status`     | `Status`           | `todo`, `in_progress`, `done`        |
| `sort`       | `i32`              | Lower values surface first           |
| `parent_id`  | `Option<String>`   | Links to parent issue's `id`         |
| `created_at` | `DateTime<Utc>`    | Set on creation                      |
| `updated_at` | `DateTime<Utc>`    | Updated on any mutation              |

## Status

Three possible values, serialized lowercase:

- **`todo`** — Default for new issues.
- **`in_progress`** — Set by `start`. Cannot start a `done` issue.
- **`done`** — Set by `finish`.

## Display types

These are output-only types. They are never stored.

### ListIssue

Used by `list`. Same as `Issue` but omits `body` and `context`.

### ShowIssue

Used by `show`, `next`, and `start`. Same as `Issue` but replaces
`context: Option<String>` with `context: Vec<ContextEntry>` — an array
containing the issue's own context plus its ancestors' context.
See [context.md](context.md).

### ContextEntry

One entry in the ancestor context chain:

| Field      | Type     | Notes                                    |
| ---------- | -------- | ---------------------------------------- |
| `depth`    | `usize`  | 0 = self, 1 = parent, 2 = grandparent … |
| `issue_id` | `String` | Source issue's ID                         |
| `title`    | `String` | Source issue's title                      |
| `context`  | `String` | The context text                         |

# Ancestor Context

## Purpose

When displaying an issue via `show`, `next`, or `start`, the `context` field
is expanded into an array containing the issue's own context plus the context
of all its ancestors (parent → grandparent → …). This gives consumers full
hierarchical context without manual lookups.

## How it works

`collect_ancestor_context(issue, repo)` walks the `parent_id` chain upward.
For each issue (including the original) that has a non-empty `context`, it
emits a `ContextEntry`. Ancestors without context are skipped — gaps in
`depth` values are expected. A depth limit of 100 guards against cycles.

The function lives in the domain layer and accepts `&dyn IssueRepository`,
requiring no changes to the repository trait.

## Storage vs display

- **Storage:** `Issue.context` remains `Option<String>` — a single plain-text
  field per issue.
- **Display:** `ShowIssue.context` is `Vec<ContextEntry>` — the assembled
  ancestor chain.

## Which commands include ancestor context

| Command  | Output type | Ancestor context |
| -------- | ----------- | ---------------- |
| `show`   | `ShowIssue` | ✓                |
| `next`   | `ShowIssue` | ✓                |
| `start`  | `ShowIssue` | ✓                |
| `list`   | `ListIssue` | ✗                |
| `add`    | `Issue`     | ✗                |
| `edit`   | `Issue`     | ✗                |
| `finish` | `Issue`     | ✗                |
| `delete` | `Issue`     | ✗                |

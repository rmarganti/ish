# Commands

All commands output JSON to stdout.

## Global options

| Flag              | Description                                      |
| ----------------- | ------------------------------------------------ |
| `--db-path <PATH>`| Custom database path (default: `.local/issues.jsonl`) |

## add

Create a new issue. Outputs the created `Issue`.

```
ish add <TITLE> [--body <TEXT>] [--context <TEXT>] [--parent <ID>]
```

New issues default to `todo` status with `sort = 0`.

## list

List issues. Outputs `ListIssue[]` (no body or context).

```
ish list [--status <STATUS>] [--parent <ID>]
```

- `--status` accepts `todo`, `in_progress`, or `done`.
- `--parent` filters to children of the given issue.
- With no flags, returns all issues.
- Results are sorted by `sort` ascending, then `created_at` descending.

## show

Display a single issue with ancestor context. Outputs `ShowIssue`.

```
ish show <ID>
```

## next

Show the next actionable todo issue. Outputs `ShowIssue[]` (single-element array).

```
ish next
```

Selection logic:
1. Filter to `todo` issues.
2. Exclude issues that have any non-`done` children (blocked parents).
3. Sort by `sort` ascending, then `created_at` ascending (oldest first).
4. Return the first result.

## start

Mark an issue as `in_progress`. Outputs `ShowIssue`.

```
ish start <ID>
```

Fails if the issue is already `done`.

## finish

Mark an issue as `done`. Outputs the updated `Issue`.

```
ish finish <ID>
```

## edit

Update issue fields. Outputs the updated `Issue`. Only provided fields are changed.

```
ish edit <ID> [--title <TEXT>] [--body <TEXT>] [--context <TEXT>] [--sort <N>]
```

## delete

Remove an issue. Outputs the deleted `Issue`, then removes it from storage.

```
ish delete <ID>
```

## clear

Remove all issues from the database. Outputs empty array `[]` on success.

```
ish clear [--yes]
```

- `--yes` skips the confirmation prompt (useful for scripting).
- If database is empty or doesn't exist, treats as success.

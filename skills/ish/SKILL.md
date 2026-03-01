---
name: ish
description: Create and manage issues in ish from markdown files and PRDs. Use when the user wants to break down a feature spec, PRD, or markdown document into actionable issues.
---

This skill creates issues in ish from a markdown file or PRD. Use when the user
wants to convert a plan or spec into actionable issues.

All commands output JSON to stdout.

## Workflow

1. **Identify the source**: The user should provide a markdown file path or the
   content directly. If they mention a file, read it first.

2. **Parse the content**: Analyze the markdown/PRD and identify actionable items:
   - User stories become top-level issues
   - Implementation tasks become child issues (tasks)
   - Only if REALLY needed, break down into subtasks.
   - Each issue should have a clear title and description
   - Each issue should include important context.

3. **Create issues**: For each actionable item:
   - `ish add "title Use" --body "description"` for top-level issues
   - Use `ish add "title" --body "description" --parent <parent-id>` for subtasks
   - Set appropriate sort order using `ish edit <id> --sort <n>`

4. **Output**: After creating issues, run `ish list` to show all created issues with their IDs.

## ish Command Reference

```
# Add issue (defaults: status=todo, sort=0)
ish add "Fix bug" --body "Description here"
ish add "Subtask" --parent <parent-id>
ish add "Task with context" --body "Work to do" --context "Reference: file.txt, domain logic..."

# List issues (sorted by sort asc, then created_at desc)
ish list                    # all issues
ish list --status todo      # filter by status (todo, in_progress, done)
ish list --parent <id>      # filter by parent

# Work on issues
ish next                    # show next actionable todo issue (skips blocked issues)
ish start <id>              # mark issue as in_progress
ish finish <id>             # mark issue as done

# Edit issues
ish edit <id> --title "New title"
ish edit <id> --body "New body"
ish edit <id> --context "Updated context"
ish edit <id> --sort 1      # set sort order (lower = higher priority)

# View and delete
ish show <id>               # show issue details with ancestor context
ish delete <id>             # delete issue
ish clear --yes             # delete all issues

# Options
-d, --db-path <PATH>        # custom database path (default: .local/issues.jsonl)
```

## Key Concepts

### Context (Important!)

The `--context` field carries reference material needed to complete the work:
file paths, domain logic, API contracts, etc. Context is inherited — when
viewing an issue via `show`, `next`, or `start`, ancestor context is
automatically included.

```
# Parent: shared context for all children
ish add "Add user auth" --context "Using argon2, JWTs signed with ES256..."

# Child: only task-specific context
ish add "Add password hashing" --parent <id> --context "Put in src/auth/hash.rs..."
```
Running `ish show` on the child will include both its own context and the parent's.

## Tips

- Group related tasks under parent issues
- Use `--context` for reference material — it's inherited by children
- Use sort order to prioritize (lower = higher priority)
- The `next` command smartly skips blocked issues (those with incomplete children)
- Issues are stored in `.local/issues.jsonl` by default, or use `--db-path <PATH>`
- All commands output JSON — parse it for scripts

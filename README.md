# ish

A simple terminal-based issue tracker written in Rust. All commands output JSON
to stdout, making ish scriptable and composable with other tools.

## Installation

```bash
cargo build --release
```

The binary will be at `target/release/ish`.

## Usage

```
ish [OPTIONS] <COMMAND>
```

### Commands

**Add a new issue**
```bash
ish add "Fix bug" --body "Description here"
ish add "Fix bug" --body "Description" --context "Reference: file.txt, domain logic: ..."
ish add "Subtask" --parent <parent-id>
```

New issues default to `todo` status with `sort = 0`.

**List issues**
```bash
ish list                    # all issues
ish list --status todo      # filter by status (todo, in_progress, done)
ish list --parent <id>      # filter by parent
```

Results are sorted by `sort` ascending, then `created_at` descending.

**Work on issues**
```bash
ish next                    # show next actionable todo issue
ish start <id>              # mark issue as in_progress
ish finish <id>             # mark issue as done
```

`next` selects the highest-priority `todo` issue that is not blocked (i.e., has
no incomplete children). `start` fails if the issue is already `done`.

**Edit issues**
```bash
ish edit <id> --title "New title"
ish edit <id> --body "New body"
ish edit <id> --context "Updated context"
ish edit <id> --sort 1      # set sort order
```

Only provided fields are changed.

**View and delete**
```bash
ish show <id>               # show issue details with ancestor context
ish delete <id>             # delete issue
```

### Options

- `-d`, `--db-path <PATH>` — Custom database path (default: `.local/issues.jsonl`)

## Features

- **JSON output** — Every command emits structured JSON to stdout
- **JSONL storage** with atomic writes (rename-based) to prevent corruption
- **Issue status tracking:** `todo` → `in_progress` → `done`
- **Parent-child issue relationships** with hierarchical context inheritance
- **Ancestor context** — `show`, `next`, and `start` include context from the
  entire parent chain, so shared reference material can live on the parent
- **Sort ordering** for prioritization (lower values surface first)
- **Smart `next` command** that skips blocked issues (issues with incomplete children)
- **8-character hex IDs** derived from UUIDv4

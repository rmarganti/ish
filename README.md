# ish

A simple terminal-based issue tracker written in Rust.

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

**List issues**
```bash
ish list                    # all issues
ish list --status todo      # filter by status (todo, in_progress, done)
ish list --parent <id>      # filter by parent
```

**Work on issues**
```bash
ish next                    # show ID of next todo issue
ish start <id>              # start working on issue
ish finish <id>             # mark issue as done
```

**Edit issues**
```bash
ish edit <id> --title "New title"
ish edit <id> --body "New body"
ish edit <id> --context "Updated context"
ish edit <id> --sort 1      # set sort order
```

**View and delete**
```bash
ish show <id>               # show issue details
ish delete <id>             # delete issue
```

### Options

- `--db-path <PATH>` - Custom database path (default: `.local/issues.jsonl`)

## Features

- JSONL storage
- Issue status tracking: todo, in_progress, done
- Parent-child issue relationships
- Sort ordering for prioritization
- Issue context field for storing reference material and notes
- Smart "next" command that skips blocked issues (issues with incomplete children)

# ish

A terminal-based issue tracker written in Rust. All commands output JSON to stdout.

## Commands

- `add` — Create a new issue
- `list` — List issues (filter by status, parent)
- `next` — Show next actionable todo issue
- `start` — Mark issue as in_progress
- `finish` — Mark issue as done
- `edit` — Update issue fields
- `show` — Show issue details
- `delete` — Delete an issue
- `clear` — Delete all issues (with optional --yes flag)

## Feedback Loop

### Test

```bash
cargo test
```

### Lint

```bash
cargo fmt --all -- --check
cargo clippy -- -D warnings
```

**Always run the feedback loop after making changes to verify correctness.**

**Always ensure docs, README.md, and AGENTS.md are up to date when adding
features, or changing them significantly.**


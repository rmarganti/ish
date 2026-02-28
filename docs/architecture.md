# Architecture

## Layers

```
cli/        → Command parsing (clap), dispatches to domain + storage
domain/     → Issue types, status enum, validation, ancestor context assembly
storage/    → IssueRepository trait, JSONL implementation
```

## Data flow

1. `main.rs` parses CLI args via `clap::Parser` and calls `run_cli`.
2. `run_cli` resolves the database path and creates a `JSONLRepository`.
3. Each subcommand handler receives a `&dyn IssueRepository`, performs
   domain logic, and prints JSON to stdout.
4. Errors print to stderr and exit with code 1.

## Key design decisions

- **Trait-based storage.** All command handlers accept `&dyn IssueRepository`,
  keeping the domain layer decoupled from the JSONL backend.
- **In-memory testing.** `JSONLRepository::in_memory()` provides a
  zero-setup repository for unit tests.
- **JSON output.** All commands emit `serde_json::to_string_pretty` output,
  making ish scriptable and composable with other tools.

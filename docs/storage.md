# Storage

## Format

Issues are stored in a [JSONL](https://jsonlines.org/) file — one JSON object
per line. Default path: `.local/issues.jsonl`.

## Ordering

The file is kept sorted by `id` (ascending) after every write. This is purely
for deterministic file diffs; query results use their own sort order
(`sort` asc, then `created_at` desc for lists; `created_at` asc for `next`).

## Write strategy

Writes use atomic rename:

1. Serialize all issues to a `.jsonl.tmp` file.
2. `fsync` / flush the temp file.
3. Rename `.jsonl.tmp` → `.jsonl`.

This prevents partial writes from corrupting the database.

## In-memory mode

`JSONLRepository::in_memory()` skips all file I/O. Used exclusively in tests.

## Error cases

| Error         | Cause                                    |
| ------------- | ---------------------------------------- |
| `NotFound`    | `get_by_id`, `update`, or `delete` with unknown ID |
| `ParseError`  | Malformed JSON in the JSONL file         |
| `WriteError`  | File create / write / rename failure     |
| `Io`          | Other filesystem errors                  |
| `LockError`   | Mutex poisoned (should not occur in normal use) |

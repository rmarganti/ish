---
# ish-i6i7
title: 'Core store: archive and unarchive'
status: completed
type: task
priority: normal
created_at: 2026-04-17T13:32:26Z
updated_at: 2026-04-17T15:45:53Z
parent: ish-6cqj
blocked_by:
    - ish-gqts
    - ish-dpfq
    - ish-614c
---

## Description\n\nImplement archive/unarchive: move ishoo files to/from the `archive/` subdirectory.\n\nReference: `beans/pkg/beancore/core.go` — `Archive()`, `Unarchive()`, `IsArchived()`, `LoadAndUnarchive()`.\n\n## Requirements\n\n- [x] `archive(id)` — move file from `.ish/{file}.md` to `.ish/archive/{file}.md`, create archive dir if needed, update path in store\n- [x] `unarchive(id)` — move file back from archive to main dir, update path\n- [x] `is_archived(id)` — check if path starts with `archive/`\n- [x] `load_and_unarchive(id)` — find in archive, move back, add to store\n- [x] Archived ishoos are included in `load()` (they live in the archive subdir which is walked)\n- [x] `archive_all_completed()` — bulk archive all ishoos with archive status (completed/scrapped)\n\n## Verification\n\n```bash\ncargo fmt --all -- --check\ncargo test\ncargo clippy -- -D warnings\n```\n\nIntegration tests cover archive/unarchive path updates, loading from `archive/`, and bulk archiving of completed/scrapped ishoos.\n\n## Summary of Changes\n\n- Added store-level `archive`, `unarchive`, `is_archived`, `load_and_unarchive`, and `archive_all_completed` operations in `src/core/store.rs`.\n- Added a `StoreError::NotFound` path for missing ishoos and small internal helpers for resolving archive paths and relative paths.\n- Added integration-style tests proving file moves on disk, store path updates, archived file loading, and bulk archiving based on archive statuses.

---
# ish-sr0l
title: 'Core store: create, update, delete operations'
status: completed
type: task
priority: normal
created_at: 2026-04-17T13:32:26Z
updated_at: 2026-04-17T15:59:22Z
parent: ish-6cqj
blocked_by:
    - ish-gqts
    - ish-dpfq
    - ish-614c
---

## Description\n\nImplement CRUD operations on the store.\n\nReference: `beans/pkg/beancore/core.go` — `Create()`, `Update()`, `Delete()`, `saveToDisk()`.\n\n## Requirements\n\n- [x] `create(title, status, type, priority, body, tags, parent, blocking, blocked_by)` — generate ID, build filename with slug, write to disk, add to store, return ishoo\n- [x] `update(id, changes)` — find ishoo, apply field changes, update `updated_at`, save to disk. Support: status, type, priority, title, body (full replace), body_replace (old→new), body_append, add/remove tags, parent, add/remove blocking, add/remove blocked_by\n- [x] ETag validation on update: if `if_match` provided, compare with current ETag, return `ETagMismatchError` on mismatch\n- [x] `delete(id)` — find ishoo, remove file from disk, remove from store, clean up incoming references (other ishoos that reference this ID in parent/blocking/blocked_by)\n- [x] `save_to_disk(ishoo)` — render to markdown, write to file path\n\n## Verification\n\n```bash\ncargo test\n```\n\nIntegration tests: create → verify file exists, update → verify file changed, delete → verify file removed and references cleaned.

## Summary of Changes

- Added `Store::create`, `Store::update`, `Store::delete`, and `Store::save_to_disk` with config-backed status/type/priority validation, ID generation, slug-based filenames, and markdown persistence.
- `update` now supports ETag checks, body replace/append helpers, tag mutations, parent/blocking edits, and renames files when a title change produces a new slug.
- `delete` now removes the target file and rewrites any remaining ishoos that referenced the deleted ID in `parent`, `blocking`, or `blocked_by`.
- Added store tests covering create persistence, update behavior and ETag mismatch handling, and delete reference cleanup.

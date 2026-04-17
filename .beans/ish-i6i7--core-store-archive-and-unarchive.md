---
# ish-i6i7
title: 'Core store: archive and unarchive'
status: todo
type: task
created_at: 2026-04-17T13:32:26Z
updated_at: 2026-04-17T13:32:26Z
parent: ish-6cqj
blocked_by:
    - ish-gqts
    - ish-dpfq
    - ish-614c
---

## Description\n\nImplement archive/unarchive: move ishoo files to/from the `archive/` subdirectory.\n\nReference: `beans/pkg/beancore/core.go` — `Archive()`, `Unarchive()`, `IsArchived()`, `LoadAndUnarchive()`.\n\n## Requirements\n\n- [ ] `archive(id)` — move file from `.ish/{file}.md` to `.ish/archive/{file}.md`, create archive dir if needed, update path in store\n- [ ] `unarchive(id)` — move file back from archive to main dir, update path\n- [ ] `is_archived(id)` — check if path starts with `archive/`\n- [ ] `load_and_unarchive(id)` — find in archive, move back, add to store\n- [ ] Archived ishoos are included in `load()` (they live in the archive subdir which is walked)\n- [ ] `archive_all_completed()` — bulk archive all ishoos with archive status (completed/scrapped)\n\n## Verification\n\n```bash\ncargo test\n```\n\nIntegration tests: archive → verify file moved, unarchive → verify file restored, archived ishoos appear in `all()`.

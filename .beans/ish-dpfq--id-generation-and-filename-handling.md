---
# ish-dpfq
title: ID generation and filename handling
status: todo
type: task
created_at: 2026-04-17T13:31:37Z
updated_at: 2026-04-17T13:31:37Z
parent: ish-4qx4
blocked_by:
    - ish-0iv6
---

## Description\n\nImplement NanoID-based ID generation and filename parsing/building.\n\nReference: `beans/pkg/bean/id.go`\n\n## Requirements\n\n- [ ] `new_id(prefix, length)` — generate NanoID from alphabet `[a-z0-9]`, configurable length, with prefix\n- [ ] Offensive-word filter — maintain blocklist, regenerate if ID contains blocked substring\n- [ ] `parse_filename(name)` — extract `(id, slug)` from filename. Support format: `{id}--{slug}.md` and `{id}.md`\n- [ ] `build_filename(id, slug)` — construct `{id}--{slug}.md` or `{id}.md`\n- [ ] `slugify(title)` — lowercase, replace spaces/underscores with hyphens, strip non-alphanumeric, collapse hyphens, truncate to 50 chars\n\n## Verification\n\n```bash\ncargo test\n```\n\nUnit tests:\n- Generated IDs match expected alphabet and length\n- Offensive words are never in generated IDs\n- Filename round-trip: build → parse\n- Slugify edge cases (unicode, long strings, consecutive special chars)

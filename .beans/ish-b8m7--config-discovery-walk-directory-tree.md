---
# ish-b8m7
title: Config discovery (walk directory tree)
status: todo
type: task
created_at: 2026-04-17T13:31:55Z
updated_at: 2026-04-17T13:31:55Z
parent: ish-ksgz
blocked_by:
    - ish-0iv6
---

## Description\n\nImplement upward directory-tree search to find the nearest `.ish.yml` config file.\n\nReference: `beans/pkg/config/config.go` — `FindConfig()`, `FindConfigWithin()`.\n\n## Requirements\n\n- [ ] `find_config(start_dir)` — walk up from `start_dir` to filesystem root, return path to first `.ish.yml` found\n- [ ] `find_config_within(start_dir, root_dir)` — same but stop at `root_dir`\n- [ ] Return `None` if no config found (not an error)\n\n## Verification\n\n```bash\ncargo test\n```\n\nUnit tests with temp directories: config in current dir, config in parent, no config found.

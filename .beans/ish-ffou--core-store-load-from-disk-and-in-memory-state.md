---
# ish-ffou
title: 'Core store: load from disk and in-memory state'
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

## Description\n\nImplement the core in-memory store that loads ishoos from disk.\n\nReference: `beans/pkg/beancore/core.go` — `Core` struct, `New()`, `Load()`, `loadFromDisk()`, `loadIshoo()`, `All()`, `Get()`.\n\n## Requirements\n\n- [ ] `Store` struct with `root: PathBuf`, `config: Config`, `ishoos: HashMap<String, Ishoo>`\n- [ ] `Store::new(root, config)`\n- [ ] `Store::load()` — walk `.ish/` directory tree, parse all `.md` files, skip dot-prefixed subdirs and `archive/`'s contents are loaded too\n- [ ] `load_ishoo(path)` — parse file, set `id`/`slug` from filename, set `path` relative to root, apply defaults for empty fields (type→task, priority→normal, tags→[], blocking→[], created_at from file mtime)\n- [ ] `all()` — return all ishoos\n- [ ] `get(id)` — exact match, then try with prefix prepended\n- [ ] `normalize_id(id)` — prepend prefix if not already present\n- [ ] Initialize `.ish/` directory with `.gitignore`\n\n## Verification\n\n```bash\ncargo test\n```\n\nIntegration tests with temp directories: create files, load, verify in-memory state.

---
# ish-ffou
title: 'Core store: load from disk and in-memory state'
status: completed
type: task
priority: normal
created_at: 2026-04-17T13:32:26Z
updated_at: 2026-04-17T15:37:31Z
parent: ish-6cqj
blocked_by:
    - ish-gqts
    - ish-dpfq
    - ish-614c
---

## Description\n\nImplement the core in-memory store that loads ishoos from disk.\n\nReference: `beans/pkg/beancore/core.go` — `Core` struct, `New()`, `Load()`, `loadFromDisk()`, `loadIshoo()`, `All()`, `Get()`.\n\n## Requirements\n\n- [x] `Store` struct with `root: PathBuf`, `config: Config`, `ishoos: HashMap<String, Ishoo>`\n- [x] `Store::new(root, config)`\n- [x] `Store::load()` — walk `.ish/` directory tree, parse all `.md` files, skip dot-prefixed subdirs and `archive/`'s contents are loaded too\n- [x] `load_ishoo(path)` — parse file, set `id`/`slug` from filename, set `path` relative to root, apply defaults for empty fields (type→task, priority→normal, tags→[], blocking→[], created_at from file mtime)\n- [x] `all()` — return all ishoos\n- [x] `get(id)` — exact match, then try with prefix prepended\n- [x] `normalize_id(id)` — prepend prefix if not already present\n- [x] Initialize `.ish/` directory with `.gitignore`\n\n## Verification\n\n```bash\ncargo fmt --all\ncargo test\ncargo clippy -- -D warnings\n```\n\nIntegration-style unit tests create temp directories and verify recursive loading, archive inclusion, hidden-directory skipping, default backfilling, and normalized lookups.\n\n## Summary of Changes\n\nAdded a filesystem-backed `Store` in `src/core/store.rs` with initialization, recursive markdown loading, archive inclusion, hidden-directory skipping, filename-based ID normalization, and tolerant frontmatter defaults driven by `Config`. Added tests covering `.gitignore` initialization, directory walking, archive loading, hidden-directory skipping, default backfilling, and normalized lookups.\n\n## Verification Notes\n\nValidated with `cargo fmt --all`, `cargo test`, and `cargo clippy -- -D warnings`.

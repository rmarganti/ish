---
# ish-614c
title: Config struct and .ish.yml parsing
status: todo
type: task
created_at: 2026-04-17T13:31:55Z
updated_at: 2026-04-17T13:31:55Z
parent: ish-ksgz
blocked_by:
    - ish-0iv6
---

## Description\n\nDefine the config struct and implement `.ish.yml` file parsing, writing, and defaults.\n\nReference: `beans/pkg/config/config.go`\n\n## Requirements\n\n- [ ] `Config` struct with sections: `IshConfig` (path, prefix, id_length, default_status, default_type), `ProjectConfig` (name)\n- [ ] Hardcoded `StatusConfig` list: in-progress (yellow), todo (green), draft (blue), completed (gray, archive), scrapped (gray, archive)\n- [ ] Hardcoded `TypeConfig` list: milestone (cyan), epic (purple), bug (red), feature (green), task (blue)\n- [ ] Hardcoded `PriorityConfig` list: critical (red), high (yellow), normal (white), low (gray), deferred (gray)\n- [ ] `Config::default()` — sensible defaults (path=`.ish`, id_length=4, default_status=todo, default_type=task)\n- [ ] `Config::default_with_prefix(prefix)`\n- [ ] `Config::load(path)` — read and deserialize `.ish.yml`\n- [ ] `Config::save(dir)` — serialize and write `.ish.yml` to directory\n- [ ] Validation helpers: `is_valid_status()`, `is_valid_type()`, `is_valid_priority()`, `status_names()`, `type_names()`, `priority_names()`\n- [ ] `is_archive_status(status)` — returns true for completed/scrapped\n- [ ] `get_status(name)`, `get_type(name)`, `get_priority(name)` — lookup by name, return config with color\n\n## Verification\n\n```bash\ncargo test\n```\n\nUnit tests: default values, round-trip serialize/deserialize, validation edge cases.

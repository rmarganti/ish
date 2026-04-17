---
# ish-gqts
title: Ishoo struct and YAML frontmatter parsing
status: todo
type: task
created_at: 2026-04-17T13:31:37Z
updated_at: 2026-04-17T13:31:37Z
parent: ish-4qx4
blocked_by:
    - ish-0iv6
---

## Description\n\nDefine the core `Ishoo` struct and implement parsing/rendering of markdown files with YAML frontmatter.\n\nReference: `beans/pkg/bean/bean.go` — the `Bean` struct, `Parse()`, and `Render()` functions.\n\n## Requirements\n\n- [ ] Define `Ishoo` struct with fields: `id`, `slug`, `path`, `title`, `status`, `ishoo_type`, `priority`, `tags`, `created_at`, `updated_at`, `order`, `body`, `parent`, `blocking`, `blocked_by`\n- [ ] Implement `serde` Serialize/Deserialize for the frontmatter subset (exclude `id`, `slug`, `path`, `body` from YAML)\n- [ ] Parse `.md` file: split `---` delimited YAML frontmatter from markdown body\n- [ ] Render `Ishoo` back to `.md`: `---\n# {id}\n{yaml}\n---\n\n{body}\n`\n- [ ] Handle edge cases: missing optional fields, empty body, trailing newlines\n- [ ] Implement `serde_json::Serialize` for JSON output (include all fields + computed `etag`)\n\n## Verification\n\n```bash\ncargo test\n```\n\nUnit tests: round-trip parse → render → parse with various field combinations.

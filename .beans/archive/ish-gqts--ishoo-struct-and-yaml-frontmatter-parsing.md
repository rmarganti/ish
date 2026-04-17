---
# ish-gqts
title: Ishoo struct and YAML frontmatter parsing
status: completed
type: task
priority: normal
created_at: 2026-04-17T13:31:37Z
updated_at: 2026-04-17T13:55:23Z
parent: ish-4qx4
blocked_by:
    - ish-0iv6
---

## Description\n\nDefine the core `Ishoo` struct and implement parsing/rendering of markdown files with YAML frontmatter.\n\nReference: `beans/pkg/bean/bean.go` — the `Bean` struct, `Parse()`, and `Render()` functions.\n\n## Requirements\n\n- [x] Define `Ishoo` struct with fields: `id`, `slug`, `path`, `title`, `status`, `ishoo_type`, `priority`, `tags`, `created_at`, `updated_at`, `order`, `body`, `parent`, `blocking`, `blocked_by`\n- [x] Implement `serde` Serialize/Deserialize for the frontmatter subset (exclude `id`, `slug`, `path`, `body` from YAML)\n- [x] Parse `.md` file: split `---` delimited YAML frontmatter from markdown body\n- [x] Render `Ishoo` back to `.md`: `---\n# {id}\n{yaml}\n---\n\n{body}\n`\n- [x] Handle edge cases: missing optional fields, empty body, trailing newlines\n- [x] Implement `serde_json::Serialize` for JSON output (include all fields + computed `etag`)\n\n## Verification\n\n```bash\ncargo test\n```\n\nUnit tests: round-trip parse → render → parse with various field combinations.


## Summary of Changes

Implemented `Ishoo` struct and YAML frontmatter parsing in `src/model/ishoo.rs`:

- **`Ishoo` struct**: Core data type with all specified fields
- **`Frontmatter` struct**: Internal serde type for YAML serialization (excludes `id`, `slug`, `path`, `body`); optional fields use `skip_serializing_if`
- **`Ishoo::parse(filename, content)`**: Parses `.md` files by splitting `---`-delimited frontmatter, extracting the `# {id}` comment, deserializing YAML, and separating the body
- **`Ishoo::render()`**: Renders back to the `---\n# {id}\n{yaml}\n---\n\n{body}\n` format
- **`Ishoo::to_json(etag)`**: Converts to `IshooJson` struct for JSON output with all fields + computed etag
- **`ParseError` enum**: `MissingFrontmatter`, `MissingId`, `Yaml(serde_yaml::Error)` with `Display` and `Error` impls
- **11 unit tests**: Basic parse, minimal fields, render round-trip, empty body, missing frontmatter/ID errors, filename parsing, JSON output

### Implementation Notes

- Body is trimmed of surrounding whitespace during parse for clean round-trips
- `parse_filename()` helper supports `{id}--{slug}.md` and `{id}.md` formats
- Module exposed via `src/model/mod.rs` with `#[allow(dead_code)]` since no CLI commands reference it yet
- All validations pass: `cargo test`, `cargo fmt --check`, `cargo clippy -- -D warnings`

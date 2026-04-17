---
# ish-dpfq
title: ID generation and filename handling
status: completed
type: task
priority: normal
created_at: 2026-04-17T13:31:37Z
updated_at: 2026-04-17T15:27:49Z
parent: ish-4qx4
blocked_by:
    - ish-0iv6
---

## Description

Implement NanoID-based ID generation and filename parsing/building.

Reference: `beans/pkg/bean/id.go`

## Requirements

- [x] `new_id(prefix, length)` — generate NanoID from alphabet `[a-z0-9]`, configurable length, with prefix
- [x] Offensive-word filter — maintain blocklist, regenerate if ID contains blocked substring
- [x] `parse_filename(name)` — extract `(id, slug)` from filename. Support format: `{id}--{slug}.md` and `{id}.md`
- [x] `build_filename(id, slug)` — construct `{id}--{slug}.md` or `{id}.md`
- [x] `slugify(title)` — lowercase, replace spaces/underscores with hyphens, strip non-alphanumeric, collapse hyphens, truncate to 50 chars

## Verification

```bash
cargo test
```

Unit tests:
- Generated IDs match expected alphabet and length
- Offensive words are never in generated IDs
- Filename round-trip: build -> parse
- Slugify edge cases (unicode, long strings, consecutive special chars)

## Summary of Changes

- Added reusable `new_id`, `parse_filename`, `build_filename`, and `slugify` helpers in `src/model/ishoo.rs` so upcoming model and storage work can share one filename/ID implementation.
- `new_id` now uses the repo's `nanoid` dependency with a fixed `[a-z0-9]` alphabet and retries when a generated ID matches a maintained offensive substring blocklist.
- `parse_filename` now reads the basename before parsing, which keeps filename extraction correct even when callers pass relative paths.

## Notes for Future Workers

- `slugify` is intentionally ASCII-only today: Unicode letters are dropped rather than transliterated, matching this bean's current requirement to strip non-alphanumeric characters.
- The offensive-word filter is implemented as a substring check over the full generated ID, including the prefix.
- Validation run: `cargo fmt --all -- --check`, `cargo test`, `cargo clippy -- -D warnings`.

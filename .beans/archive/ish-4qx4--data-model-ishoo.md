---
# ish-4qx4
title: Data model (Ishoo)
status: completed
type: epic
priority: normal
created_at: 2026-04-17T13:30:45Z
updated_at: 2026-04-17T18:35:54Z
parent: ish-orp4
blocked_by:
    - ish-gqts
    - ish-dpfq
    - ish-2iuf
    - ish-ba6i
    - ish-fkhv
    - ish-l4gd
---

The core Ishoo data type: struct definition, YAML frontmatter parsing/rendering, ID generation, filename handling, ETag, fractional indexing, tag validation, and body helpers.

## Summary of Changes

Completed the Ishoo data model epic by landing the child tasks that define the core `Ishoo` struct and markdown/YAML representation, ID and filename helpers, deterministic ETag generation, fractional ordering keys, and tag/body helper utilities.

Useful implementation entry points for future work:
- `src/model/ishoo.rs` holds parsing, rendering, ID/slug/filename helpers, ETag, ordering helpers, tag normalization, and body update helpers.
- `src/core/store.rs` loads and persists Ishoo records from disk and applies config-backed defaults during read/write operations.
- Coverage for the completed model behaviors lives alongside those modules in their unit tests.

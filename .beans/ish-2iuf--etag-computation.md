---
# ish-2iuf
title: ETag computation
status: completed
type: task
priority: normal
created_at: 2026-04-17T13:31:37Z
updated_at: 2026-04-17T15:23:49Z
parent: ish-4qx4
blocked_by:
    - ish-0iv6
---

## Description\n\nImplement ETag generation for optimistic concurrency control.\n\nReference: `beans/pkg/bean/bean.go` — `ETag()` method.\n\n## Requirements\n\n- [x] `etag()` method on `Ishoo` — render to bytes, FNV-1a 64-bit hash, return 16-char hex string\n- [x] Sentinel value `"0000000000000000"` if rendering fails\n- [x] Include `etag` in JSON serialization output\n\n## Verification\n\n```bash\ncargo test\n```\n\nUnit tests:\n- Same ishoo content produces same ETag\n- Different content produces different ETag\n- ETag is deterministic (no randomness)

## Summary of Changes

- Added `Ishoo::etag()` using the canonical rendered markdown as the hash input, with FNV-1a 64-bit output formatted as a 16-character lowercase hex string.
- Introduced an internal fallible render path so ETag computation can return the sentinel `0000000000000000` if YAML serialization ever fails while preserving the existing `render()` API.
- Added unit coverage for deterministic hashes, content-sensitive hash changes, and the current canonical hash for the sample issue.

## Notes for Future Workers

- The hash is computed from `Ishoo::try_render()` output rather than JSON, so ETags stay aligned with the on-disk markdown representation.
- `IshooJson` already carried an `etag` field; this task kept that shape and focused on computing the value inside the model layer.
- Validation run: `cargo test`, `cargo fmt --all -- --check`, `cargo clippy -- -D warnings`.

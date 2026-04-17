---
# ish-2iuf
title: ETag computation
status: todo
type: task
created_at: 2026-04-17T13:31:37Z
updated_at: 2026-04-17T13:31:37Z
parent: ish-4qx4
blocked_by:
    - ish-0iv6
---

## Description\n\nImplement ETag generation for optimistic concurrency control.\n\nReference: `beans/pkg/bean/bean.go` — `ETag()` method.\n\n## Requirements\n\n- [ ] `etag()` method on `Ishoo` — render to bytes, FNV-1a 64-bit hash, return 16-char hex string\n- [ ] Sentinel value `"0000000000000000"` if rendering fails\n- [ ] Include `etag` in JSON serialization output\n\n## Verification\n\n```bash\ncargo test\n```\n\nUnit tests:\n- Same ishoo content produces same ETag\n- Different content produces different ETag\n- ETag is deterministic (no randomness)

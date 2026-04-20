---
# ish-1tsc
title: Add release checksums and end-to-end validation
status: todo
type: task
priority: normal
created_at: 2026-04-20T03:01:19.222174Z
updated_at: 2026-04-20T03:01:19.222174Z
parent: ish-gv4k
blocked_by:
- ish-dki9
---

## Context
We want each published release to include checksum files and a final end-to-end validation pass that proves the artifact names, tag creation, version embedding, and GitHub Release publication all work together. This is the last safety net before treating the flow as production-ready.

## Dependencies
- Blocked by the release workflow existing (`ish-dki9`).

## Work
- Generate SHA256 checksums for all release artifacts.
- Make sure checksum files are uploaded with the release assets.
- Capture the operator-facing release checklist or short release notes in the repo if the workflow needs a handoff guide.
- Run a full manual release validation and confirm the final artifact naming matches the agreed convention.

## Verification
- Release assets include checksum files.
- The release contains the expected macOS and Linux musl artifacts with stable names.
- A manual dry run or real release confirms the whole flow is repeatable.

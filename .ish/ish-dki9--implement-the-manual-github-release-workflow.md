---
# ish-dki9
title: Implement the manual GitHub release workflow
status: todo
type: task
priority: normal
created_at: 2026-04-20T03:01:16.030605Z
updated_at: 2026-04-20T03:01:31.109135Z
parent: ish-gv4k
blocking:
- ish-1tsc
blocked_by:
- ish-ent4
- ish-s4mn
---

## Context
Add `.github/workflows/release.yml` as a manual-only release pipeline. The workflow should take a required version input (for example `0.1.0`), create an annotated git tag from that version, fail if the tag already exists, build the release matrix, generate GitHub Release notes automatically, and upload the packaged artifacts.

## Dependencies
- Blocked by build-time version metadata being embedded in the binary (`ish-ent4`).
- Blocked by confirming the musl cross-build setup for Linux release targets (`ish-s4mn`).

## Work
- Add `workflow_dispatch` input validation for the release version.
- Create the tag from the workflow and use that tag as the source of truth.
- Build macOS arm64 and Linux musl artifacts in the job matrix.
- Generate and publish the GitHub Release with uploaded assets and release notes.
- Make reruns fail loudly if the tag already exists.

## Verification
- Workflow file passes `actionlint` or equivalent YAML/workflow validation.
- A manual dispatch with a test version reaches a published GitHub Release.
- The release page shows generated notes and all expected artifacts.

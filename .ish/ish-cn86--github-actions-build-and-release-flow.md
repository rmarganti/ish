---
# ish-cn86
title: GitHub Actions build and release flow
status: todo
type: milestone
priority: normal
created_at: 2026-04-20T03:00:48.224362Z
updated_at: 2026-04-20T03:00:48.224362Z
---

## Context
We need a manual GitHub Actions release flow for Ish that can ship Apple Silicon macOS binaries and Alpine-compatible Linux binaries. The workflow must create a tag from the requested version, build release artifacts, generate release notes, upload checksummed assets, and publish a GitHub Release.

## Success criteria
- Manual dispatch only, with a required version input that does not include a `v` prefix.
- Release artifacts for macOS arm64, Linux x86_64 musl, and Linux aarch64 musl.
- The built binary reports the release version from the tag/source of truth.
- The workflow creates/publishes the GitHub Release and fails if the tag already exists.

## Verification
- `mise run ci` continues to pass.
- A release dry run or real release proves the full workflow end to end.

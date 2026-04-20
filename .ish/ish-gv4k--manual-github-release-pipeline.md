---
# ish-gv4k
title: Manual GitHub release pipeline
status: todo
type: epic
priority: normal
created_at: 2026-04-20T03:00:52.357431Z
updated_at: 2026-04-20T03:00:52.357431Z
parent: ish-cn86
---

## Context
This epic covers the implementation work needed to ship Ish from GitHub Actions. The release flow should be idiomatic for Rust: keep normal CI separate, add a manual release workflow, embed release metadata into the binary, build macOS arm64 and Linux musl targets, and publish a real GitHub Release with notes and checksums.

## Scope
- Release metadata plumbing in Rust
- Cross-build proof for Alpine-compatible Linux binaries
- GitHub Actions release workflow
- Checksums and end-to-end validation

## Verification
- Every child task is complete.
- A maintainer can manually dispatch a release and publish artifacts without ad hoc steps.

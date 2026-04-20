---
# ish-gv4k
title: Manual GitHub release pipeline
status: completed
type: epic
priority: normal
created_at: 2026-04-20T03:00:52.357431Z
updated_at: 2026-04-20T04:00:00Z
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
- [x] Every child task is complete.
- [x] A maintainer can manually dispatch a release and publish artifacts without ad hoc steps.

## Implementation notes
- Completed the child tasks for build-time version injection, Linux musl cross-builds, the manual `release.yml` workflow, and checksum/release documentation.
- The release workflow now tags `v<version>` from `workflow_dispatch`, verifies the tag does not already exist, builds `aarch64-apple-darwin`, `x86_64-unknown-linux-musl`, and `aarch64-unknown-linux-musl`, injects `ISH_BUILD_VERSION`, packages stable tarballs, generates checksum assets, and publishes a GitHub Release with generated notes.
- `docs/release-builds.md` is the maintainer handoff for the release checklist, asset naming, checksum expectations, and the post-merge end-to-end validation step on the default branch.
- Final validation also exposed a parallel-test temp-directory collision in the Rust test harness; `src/test_support.rs`, `src/config/mod.rs`, and `src/core/store.rs` now use process/counter-scoped temp paths, and the working-directory test lock recovers from poison so CI remains deterministic.

## Validation
- `python3` loop: 10x `mise exec -- cargo test`
- `mise run ci`
- `mise exec -- ish check`

## Follow-up notes
- The first real GitHub Actions dispatch still has to be run from the default branch after merge; use the checklist in `docs/release-builds.md` and record the result in milestone `ish-cn86`.

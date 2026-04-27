---
# ish-cn86
title: GitHub Actions build and release flow
status: completed
type: milestone
priority: normal
created_at: 2026-04-20T03:00:48.224362Z
updated_at: 2026-04-20T03:27:31.554657Z
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


## Implementation notes
- Completed the release-delivery stack under child epic `ish-gv4k`: build-time version injection, musl cross-build support for both Linux targets, the manual `.github/workflows/release.yml` pipeline, checksum asset generation, and maintainer-facing release docs in `docs/release-builds.md`.
- The release workflow now dispatches manually with a bare `version` input, creates/pushes `v<version>` as the source-of-truth tag, builds `aarch64-apple-darwin`, `x86_64-unknown-linux-musl`, and `aarch64-unknown-linux-musl`, verifies `ish version`, generates checksum assets, and publishes a GitHub Release with generated notes.
- Final stabilization work also fixed parallel-test temp-directory collisions so the release-related CI validations remain deterministic.

## Validation
- `mise run ci`
- `mise exec -- ish check`
- `gh workflow list` (default branch currently exposes `CI`; the release workflow is implemented on this branch and becomes dispatchable from GitHub Actions once merged to the default branch)

## Follow-up notes
- After merging the release-workflow commits to the default branch, run the operator checklist in `docs/release-builds.md` and record the first successful GitHub Actions release dispatch against this milestone if a stronger production proof is needed.

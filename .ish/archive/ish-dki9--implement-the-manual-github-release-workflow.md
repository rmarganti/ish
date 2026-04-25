---
# ish-dki9
title: Implement the manual GitHub release workflow
status: completed
type: task
priority: normal
created_at: 2026-04-20T03:01:16.030605Z
updated_at: 2026-04-20T03:16:11Z
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

## Implementation notes
- Added `.github/workflows/release.yml` as a manual-only release workflow with a required `version` input, semantic-version validation, and an explicit check that rejects reruns if `v<version>` already exists remotely.
- The workflow creates and pushes an annotated `v<version>` tag first, then checks out that tag for every build job so the release tag is the source of truth for the published artifacts.
- Added a three-target build matrix covering `aarch64-apple-darwin`, `x86_64-unknown-linux-musl`, and `aarch64-unknown-linux-musl`; Linux builds use `cargo-zigbuild`, verify the musl artifacts, and smoke-test the x86_64 binary in Alpine.
- Each build injects `ISH_BUILD_VERSION=<version>`, verifies `ish version` reports the requested release version, packages a tarball named `ish-v<version>-<target>.tar.gz`, and uploads it for the final publish job.
- The publish job downloads the packaged artifacts and runs `gh release create --generate-notes --verify-tag` so GitHub Release notes are generated automatically from the pushed tag.
- Documented the release artifact naming and workflow behavior in `docs/release-builds.md` for the checksum/final-validation follow-up task.

## Validation
- `go install github.com/rhysd/actionlint/cmd/actionlint@latest && ./.tmp/bin/actionlint .github/workflows/release.yml`
- `ruby -e 'require "yaml"; YAML.load_file(".github/workflows/release.yml")'`
- `mise run ci`
- `mise exec -- ish check`
- Attempted `gh workflow run release.yml --ref build -f version=0.1.0-agent.1`; GitHub returned `404 workflow release.yml not found on the default branch`, which confirms the end-to-end dispatch cannot be exercised until this workflow lands on the default branch.

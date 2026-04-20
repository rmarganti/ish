---
# ish-1tsc
title: Add release checksums and end-to-end validation
status: completed
type: task
priority: normal
created_at: 2026-04-20T03:01:19.222174Z
updated_at: 2026-04-20T03:19:31.887711Z
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

## Implementation notes
- Added `scripts/generate-release-checksums.sh` so release packaging can emit both per-artifact `.sha256` files and a combined `SHA256SUMS` manifest from the same artifact directory.
- Updated `.github/workflows/release.yml` so the publish job checks out the tagged source, generates checksums for every packaged tarball, and uploads the checksum assets alongside the release artifacts.
- Expanded `docs/release-builds.md` with the checksum outputs, a maintainer-facing release checklist, and the expected final asset set/naming convention.

## Validation
- `mise install --locked`
- `./scripts/generate-release-checksums.sh <tempdir>/*.tar.gz`
- `ruby -e 'require "yaml"; YAML.load_file(".github/workflows/release.yml")'`
- `PATH="$PWD/.tmp/bin:$PATH" actionlint .github/workflows/release.yml`
- `mise run ci`
- `mise exec -- ish check`
- Attempted `gh workflow run release.yml --ref build -f version=0.1.0-agent.2`; GitHub returned `404 workflow release.yml not found on the default branch`, so the final end-to-end dispatch still has to be performed after this workflow lands on the default branch.

## Follow-up notes
- When this branch is merged, run the documented release checklist from `docs/release-builds.md` to capture the first successful default-branch release validation in GitHub Actions.

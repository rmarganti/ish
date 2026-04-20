---
# ish-s4mn
title: Prove Linux musl builds for the release targets
status: completed
type: task
priority: normal
created_at: 2026-04-20T03:01:11.696975Z
updated_at: 2026-04-20T03:10:32.627306Z
parent: ish-gv4k
blocking:
- ish-dki9
---

## Context
We need release binaries that run in minimal Linux containers like Alpine, which means the Linux targets should be built against musl instead of glibc. The workflow also needs an idiomatic way to cross-build both `x86_64-unknown-linux-musl` and `aarch64-unknown-linux-musl` on GitHub-hosted runners.

## Work
- Evaluate the common Rust options for musl cross-compilation in CI and pick the least surprising one for this repo.
- Prove that both Linux release targets build successfully in a reproducible way.
- Capture the exact setup/build commands so the final workflow can reuse them.

## Verification
- Produce release binaries for `x86_64-unknown-linux-musl` and `aarch64-unknown-linux-musl`.
- Confirm the binaries are musl-compatible / not glibc-linked.
- Smoke-test at least one binary inside an Alpine-based container, or document an equivalent verification if a target-specific container run is not feasible in CI.



## Implementation notes
- Standardized the Linux release build approach on `cargo-zigbuild` + Zig so a normal GitHub-hosted runner can emit both musl targets without introducing Docker-based cross compilation into the build step itself.
- Added `zig` to `.mise.toml` and locked it in `mise.lock`, plus reusable `mise run build-release-linux-musl*` tasks for `x86_64-unknown-linux-musl` and `aarch64-unknown-linux-musl`.
- Added `scripts/verify-linux-musl-artifact.sh` to assert the built artifacts are static ELF binaries for the expected architecture, and `scripts/smoke-test-alpine.sh` so the release workflow can run the x86_64 artifact inside Alpine.
- Captured the setup/build/verification commands in `docs/release-builds.md` for the follow-on release workflow work.

## Validation
- `mise install --locked`
- `mise run build-release-linux-musl`
- `mise run verify-release-linux-musl`
- `file target/x86_64-unknown-linux-musl/release/ish target/aarch64-unknown-linux-musl/release/ish`
- `mise run ci`

## Follow-up notes
- Local Docker pulls were unavailable on this workstation (`docker.sock` API / credential-helper issues), so the Alpine smoke test was captured as a checked-in script for the GitHub Actions workflow to execute on a Linux runner where Docker is available.

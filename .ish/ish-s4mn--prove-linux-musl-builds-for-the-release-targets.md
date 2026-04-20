---
# ish-s4mn
title: Prove Linux musl builds for the release targets
status: todo
type: task
priority: normal
created_at: 2026-04-20T03:01:11.696975Z
updated_at: 2026-04-20T03:01:30.844653Z
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

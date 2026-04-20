# Release build notes

## Linux musl strategy

Use `cargo-zigbuild` with Zig for the Linux release binaries.

Why this repo uses it:
- it builds both `x86_64-unknown-linux-musl` and `aarch64-unknown-linux-musl` from a standard GitHub-hosted runner
- it avoids introducing Docker-based cross-compilation just to produce static musl artifacts
- the commands are close to ordinary Cargo release builds, which makes the eventual release workflow easier to read

## One-time setup

Install the Rust targets and the `cargo-zigbuild` Cargo subcommand:

```sh
mise exec -- rustup target add x86_64-unknown-linux-musl aarch64-unknown-linux-musl
mise exec -- cargo install cargo-zigbuild
```

`zig` is declared in `.mise.toml`, so `mise install --locked` will provide the linker/toolchain dependency.

## Build commands

Build each Linux release target with the same command pattern the release workflow should use:

```sh
mise run build-release-linux-musl-x86_64
mise run build-release-linux-musl-aarch64
```

Or build both together:

```sh
mise run build-release-linux-musl
```

The resulting artifacts land at:
- `target/x86_64-unknown-linux-musl/release/ish`
- `target/aarch64-unknown-linux-musl/release/ish`

## Verification commands

Check that both artifacts are static ELF binaries without a glibc dependency:

```sh
mise run verify-release-linux-musl
```

That task wraps:

```sh
./scripts/verify-linux-musl-artifact.sh x86_64-unknown-linux-musl target/x86_64-unknown-linux-musl/release/ish
./scripts/verify-linux-musl-artifact.sh aarch64-unknown-linux-musl target/aarch64-unknown-linux-musl/release/ish
```

The release workflow can also smoke-test the x86_64 artifact inside Alpine with:

```sh
./scripts/smoke-test-alpine.sh linux/amd64 target/x86_64-unknown-linux-musl/release/ish version
```

On this workstation, Docker image pulls were not available, so the local proof relied on static ELF verification via `file`. The Alpine smoke-test script is checked in so the release workflow can run the same command on a GitHub-hosted Linux runner where Docker is available.

## GitHub Actions release workflow

`.github/workflows/release.yml` reuses these same build commands and packages each target as a tarball named:

- `ish-v<version>-aarch64-apple-darwin.tar.gz`
- `ish-v<version>-x86_64-unknown-linux-musl.tar.gz`
- `ish-v<version>-aarch64-unknown-linux-musl.tar.gz`

Before publishing, the workflow runs `./scripts/generate-release-checksums.sh dist/*.tar.gz`, which produces:

- one per-artifact checksum file next to each tarball, for example `ish-v<version>-x86_64-unknown-linux-musl.tar.gz.sha256`
- a combined `SHA256SUMS` manifest covering every packaged tarball

The workflow dispatch input expects the bare version number (for example `0.1.0`), creates the matching annotated `v<version>` tag, injects `ISH_BUILD_VERSION=<version>` at build time, verifies `ish version`, and then publishes the GitHub Release with generated notes and the checksum assets.

## Operator release checklist

1. Confirm the branch to release is merged to the default branch so `workflow_dispatch` can see `release.yml`.
2. Trigger **Release** in GitHub Actions with `version=<semver-without-v>`.
3. Wait for the `prepare`, `build`, and `publish` jobs to finish successfully.
4. Open the GitHub Release for `v<version>` and verify these assets exist:
   - the three target tarballs
   - three matching `.tar.gz.sha256` files
   - `SHA256SUMS`
5. Spot-check the asset names match the documented convention above.
6. Download one artifact and confirm its checksum matches either the per-file `.sha256` asset or the `SHA256SUMS` manifest.
7. Confirm the release page shows GitHub-generated notes.

## Current end-to-end validation status

- Local validation covers YAML parsing, `actionlint`, checksum generation, and `mise run ci`.
- A real GitHub Actions dispatch still has to happen from the default branch; GitHub rejects `gh workflow run release.yml` while the workflow only exists on a non-default branch.

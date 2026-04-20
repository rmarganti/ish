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

# ish

Terminal-based issue tracker written in Rust.

## Agent notes

- Use the project toolchain via `mise`.
- Do not run `cargo ...` directly; use `mise exec -- cargo ...` if needed.
- Install the toolchain with `mise install --locked`.
- Common commands:
  - `mise run test`
  - `mise run lint`
  - `mise run ci`
- **IMPORTANT:** After changes, run `mise run ci`.

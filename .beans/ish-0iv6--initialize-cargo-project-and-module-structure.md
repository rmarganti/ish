---
# ish-0iv6
title: Initialize Cargo project and module structure
status: todo
type: task
created_at: 2026-04-17T13:30:56Z
updated_at: 2026-04-17T13:30:56Z
parent: ish-m4jr
---

## Description\n\nSet up the Rust project with Cargo. Create the binary crate and establish the module layout.\n\n## Requirements\n\n- [ ] Initialize with `cargo init --name ish`\n- [ ] Create module structure: `src/model/`, `src/config/`, `src/core/`, `src/cli/`, `src/output/`\n- [ ] Add dependencies to `Cargo.toml`:\n  - `clap` (derive) — CLI argument parsing\n  - `serde`, `serde_yaml`, `serde_json` — serialization\n  - `chrono` (serde feature) — timestamps\n  - `nanoid` — ID generation\n  - `termcolor` or `colored` — terminal styling\n  - `terminal_size` — detect terminal width\n  - `termimad` or `termimad` — markdown rendering in terminal\n  - `fnv` — FNV-1a hashing for ETags\n- [ ] Set up `main.rs` with a placeholder clap App\n- [ ] Ensure `cargo build` and `cargo test` pass\n\n## Verification\n\n```bash\ncargo build\ncargo test\n```

# ish

A terminal-based issue tracker written in Rust. s flag)

## Feedback Loop

Install the configured toolchain first if needed:

```bash
mise install --locked
```

### Test

```bash
mise run test
```

### Lint

Includes both format checking and clippy:

```bash
mise run lint
```

### Full feedback loop

```bash
mise run ci
```

**Always run the feedback loop after making changes to verify correctness.**

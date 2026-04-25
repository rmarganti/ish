# Progress

## 2026-04-25
- Completed `ish-loy6` (`TUI: scaffolding, deps, and ish tui CLI entry`).
- Added the initial `ish tui` CLI plumbing:
  - `Commands::Tui` in `src/cli/mod.rs`
  - `src/commands/tui.rs`
  - app dispatch in `src/app/mod.rs`
  - top-level `mod tui;` in `src/main.rs`
- Added the initial TUI module skeleton under `src/tui/` with stub modules:
  `model`, `msg`, `keymap`, `update`, `effect`, `runtime`, `view`, and `theme`.
  `src/tui/mod.rs::run` delegates to `runtime::run`, which is currently a no-op.
- Added `ratatui` and `crossterm` to `Cargo.toml` and refreshed `Cargo.lock`.
- `ish tui` currently requires a normal TTY flow and rejects `--json` with a
  validation error; future TUI work can assume JSON output compatibility is
  intentionally out of scope for this subcommand.
- Verification completed for this foundation step:
  - `mise exec -- cargo test`
  - `mise exec -- cargo run -- tui`
  - `mise exec -- cargo run -- --help | rg "\btui\b"`
  - `mise run ci`
- Completed `ish-8dtp` (`TUI: define Model/Msg/Effect/Screen types and bucketing helpers`).
- Added the foundational shared TUI data model in `src/tui/model.rs`:
  - `Model`, `Screen`, `BoardState`, `DetailState`, `PickerState`, `CreateFormState`, `HelpState`
  - TUI enums for `Status`, `IshType`, `Priority`, plus `Severity`/`StatusLine`
  - `BOARD_COLUMNS` and `Model::bucket_for_status(...)`
- Added the initial message and effect contracts used by later TUI work:
  - `src/tui/msg.rs` now defines navigation/screen/form/async `Msg` variants plus `FormFieldEdit`, `SaveFailure`, `SaveSuccess`, and `EditorRequest`
  - `src/tui/effect.rs` now defines `Effect`, `IssuePatch`, and `IssueDraft`
- Re-exported the public TUI types from `src/tui/mod.rs` so downstream TUI modules can import from `crate::tui::*` instead of reaching into leaf modules.
- Added a `bucket_for_status` unit test covering archived exclusion, scrapped exclusion, priority ordering, updated-at ordering, and an empty completed bucket.
- Notes for future workers:
  - The TUI layer currently uses typed enums (`Status`, `IshType`, `Priority`) even though the store model still uses strings; future executor/update work should convert at the TUI boundary.
  - `CreateFormState::new(&Config)` seeds the form type from `config.ish.default_type` and leaves priority at `normal`; if later work wants config-driven default priority, add it deliberately rather than assuming the store has one.
- Verification completed for this types/foundation step:
  - `mise exec -- cargo test`
  - `mise run ci`
  - `mise exec -- ish check`
- Completed `ish-fww3` (`TUI: extend test_support with TUI helpers`).
- Extended `src/test_support.rs` with a test-only `tui` namespace that now provides:
  - `IshBuilder` for concise in-memory TUI fixture construction
  - `model_with_board(...)` to seed a board-first `Model` with cached etags
  - `dispatch(...)` to fold `Msg`s through `tui::update::update(...)` while collecting effects
  - `key(...)` plus exported `k!` macro support for compact `KeyEvent` construction in future keymap tests
- Added focused helper coverage in `src/test_support.rs` so the new fixtures stay clippy-clean until the downstream keymap/update test ishes start consuming them directly.
- Added `Store::load_one(id)` in `src/core/store.rs` ahead of the runtime/editor work:
  - resolves both short and fully-prefixed ids
  - searches the `.ish/` tree recursively (including archived files while still skipping hidden directories)
  - returns existing typed `StoreError::NotFound` / `StoreError::Yaml` failures for focused single-issue reloads
- Notes for future workers:
  - `src/tui/update.rs` now exposes the shared `update(model, msg) -> (Model, Vec<Effect>)` function signature as a temporary no-op so the new `dispatch(...)` helper can compile before the real update task lands.
  - The new `k!` macro is exported at crate scope for tests as `crate::k!(...)`.
  - `Store::load_one(...)` is available for the eventual editor-return path, but the runtime still needs to start calling it once the editor/runtime ishes are implemented.
- Verification completed for this helper groundwork step:
  - `mise exec -- cargo test`
  - `mise run ci`

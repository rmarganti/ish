---
# ish-fww3
title: 'TUI: extend test_support with TUI helpers'
status: completed
type: task
priority: high
tags:
- tui
created_at: 2026-04-25T03:20:55.781027Z
updated_at: 2026-04-25T03:21:17.802477Z
parent: ish-q6t1
blocked_by:
- ish-8dtp
---

## Goal
Extend `src/test_support.rs` with TUI helpers so update/keymap tests can
be written concisely.

## Scope
- Add a `tui` submodule (or namespace) exposing:
  - `IshBuilder` with sensible defaults (id, title, status, type,
    priority, tags, updated_at). Reuse the existing builder if one exists
    in `test_support` already.
  - `model_with_board(ishes: Vec<Ish>) -> Model` — builds a `Model`
    seeded with a board screen and an empty status line.
  - `dispatch(model: Model, msgs: &[Msg]) -> (Model, Vec<Effect>)` —
    folds a sequence of `Msg`s through `update`, accumulating effects in
    order. Returns the final model and concatenated effects.
  - `key(code: KeyCode, mods: KeyModifiers) -> KeyEvent` and a `k!`
    macro for compact key construction in keymap tests.

## Files
- `src/test_support.rs` (extend).

## Verification
- `mise run ci` passes.
- The helpers are exercised by the unit-test ishes that depend on this.

## Implementation notes
- Extended `src/test_support.rs` with a test-only `tui` namespace that provides `IshBuilder`, `model_with_board(...)`, `dispatch(...)`, and `key(...)` helpers for upcoming keymap/update tests.
- Added an exported `k!` macro (`crate::k!(...)`) so future test modules can construct `KeyEvent`s tersely without repeating modifier boilerplate.
- Added local helper tests in `src/test_support.rs` so the new fixtures stay compiled and clippy-clean until downstream TUI test tasks start using them directly.
- Added the shared `tui::update::update(...)` function signature as a temporary no-op implementation so `dispatch(...)` can fold messages immediately without each future test having to paper over a missing symbol.
- Landed `Store::load_one(id)` in `src/core/store.rs` as enabling work for the editor/runtime follow-up: it supports short/full ids, can load archived issues directly, and surfaces focused not-found / parse failures for single-issue reloads.

## Validation
- `mise exec -- cargo test`
- `mise run ci`

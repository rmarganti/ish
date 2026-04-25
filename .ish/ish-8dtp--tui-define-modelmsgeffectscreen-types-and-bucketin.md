---
# ish-8dtp
title: 'TUI: define Model/Msg/Effect/Screen types and bucketing helpers'
status: todo
type: task
priority: high
tags:
- tui
created_at: 2026-04-25T03:20:55.514741Z
updated_at: 2026-04-25T03:21:17.715584Z
parent: ish-q6t1
blocked_by:
- ish-loy6
---

## Goal
Define the core Elm-style data types: `Model`, `Msg`, `Effect`, `Screen`
stack, plus per-screen state structs and the sort/bucketing helpers.

## Scope
### `src/tui/model.rs`
- `Model { issues: Vec<Ish>, etags: HashMap<String, String>, config: ConfigHandle, screens: Vec<Screen>, status_line: Option<StatusLine>, status_line_set_at: Option<Instant>, quit: bool, term_too_small: bool }`.
- `Screen` enum: `Board(BoardState)`, `IssueDetail(DetailState)`,
  `StatusPicker(PickerState)`, `CreateForm(CreateFormState)`,
  `Help(HelpState)`.
- Per-screen states:
  - `BoardState { selected_column: usize, column_cursors: [Option<usize>; 4], column_offsets: [usize; 4] }`.
  - `DetailState { id: String, scroll: u16 }`.
  - `PickerState { issue_id: String, options: Vec<Status>, selected: usize }`.
  - `CreateFormState { title: String, ish_type: IshType, priority: Priority, tags: String, focused_field: usize }`.
- `StatusLine { text: String, severity: Severity }` with `Severity::{Info, Success, Error}`.
- Helper `Model::bucket_for_status(&self, Status) -> Vec<&Ish>` that
  filters out archived/scrapped and sorts by priority desc → updated_at desc.
- Constants `BOARD_COLUMNS: [Status; 4] = [Draft, Todo, InProgress, Completed]`.

### `src/tui/msg.rs`
- `Msg` enum covering: navigation (`MoveLeft/Right/Up/Down`, `JumpTop`,
  `JumpBottom`, `HalfPageUp/Down`), screen transitions (`OpenDetail`,
  `OpenStatusPicker`, `OpenCreateForm`, `OpenHelp`, `PopScreen`),
  mutations (`SubmitStatusChange`, `SubmitCreateForm`,
  `SubmitCreateAndEdit`, `EditCurrentIssue`, `RequestRefresh`),
  form input (`CreateFormInput(FormFieldEdit)`, `CreateFormCycleType(i32)`,
  etc.), async results (`IssuesLoaded(Result<Vec<Ish>>)`,
  `SaveCompleted(...)`, `SaveFailed(...)`, `EditorReturned(Result<()>)`),
  housekeeping (`Tick`, `Resize(u16, u16)`, `Quit`,
  `DismissStatusLine`).

### `src/tui/effect.rs`
- `Effect` enum: `LoadIssues`, `SaveIssue { patch: IssuePatch, etag: String }`,
  `CreateIssue { draft: IssueDraft, open_in_editor: bool }`,
  `OpenEditorForIssue { id: String }`, `Quit`.

## Files
- `src/tui/model.rs`, `src/tui/msg.rs`, `src/tui/effect.rs`.
- Re-export the public types from `src/tui/mod.rs`.

## Verification
- `mise run ci` passes (types compile, no clippy warnings).
- Add a unit test in `model.rs` for `bucket_for_status` covering: archive
  exclusion, scrapped exclusion, priority/updated-at sort order, and an
  empty bucket.

#![allow(dead_code)]

use crate::model::ish::{Ish, normalize_tag};
use crate::tui::{
    BOARD_COLUMNS, BoardState, CreateFormState, DetailState, Effect, HelpState, IssueDraft,
    IssuePatch, Model, Msg, PickerState, Priority, PriorityPickerState, SaveFailure, Screen,
    Severity, Status, StatusLine,
};
use std::collections::HashMap;
use std::time::{Duration, Instant};

const BOARD_VISIBLE_ROWS: usize = 5;
const HALF_PAGE_ROWS: usize = BOARD_VISIBLE_ROWS / 2;
const DETAIL_HALF_PAGE: u16 = 10;
const STATUS_LINE_TTL: Duration = Duration::from_secs(3);
const ERROR_STICKY_TTL: Duration = Duration::from_secs(1);
const CREATE_FORM_FIELD_COUNT: usize = 5;

pub fn update(mut model: Model, msg: Msg) -> (Model, Vec<Effect>) {
    if let Some(effects) = handle_global(&mut model, &msg) {
        return (model, effects);
    }

    let Some(screen) = model.screens.last().cloned() else {
        model.screens.push(Screen::Board(BoardState::default()));
        return (model, Vec::new());
    };

    match screen {
        Screen::Board(state) => update_board(model, state, msg),
        Screen::IssueDetail(state) => update_detail(model, state, msg),
        Screen::StatusPicker(state) => update_picker(model, state, msg),
        Screen::PriorityPicker(state) => update_priority_picker(model, state, msg),
        Screen::CreateForm(state) => update_create_form(model, state, msg),
        Screen::Help(state) => update_help(model, state, msg),
    }
}

fn handle_global(model: &mut Model, msg: &Msg) -> Option<Vec<Effect>> {
    match msg {
        Msg::Quit => {
            model.quit = true;
            Some(Vec::new())
        }
        Msg::Tick => {
            expire_status_line(model);
            Some(Vec::new())
        }
        Msg::Resize(width, height) => {
            model.term_too_small = *width < 80 || *height < 20;
            Some(Vec::new())
        }
        Msg::DismissStatusLine => {
            clear_status_line(model);
            Some(Vec::new())
        }
        Msg::IssuesLoaded(result) => {
            match result {
                Ok(issues) => {
                    model.issues = issues.clone();
                    model.etags = issues
                        .iter()
                        .map(|ish| (ish.id.clone(), ish.etag()))
                        .collect::<HashMap<_, _>>();
                    clamp_board_state(model);
                }
                Err(error) => {
                    set_status_line(model, error.clone(), Severity::Error);
                }
            }
            Some(Vec::new())
        }
        Msg::SaveCompleted(success) => {
            set_status_line(model, format!("Saved {}", success.id), Severity::Success);
            Some(Vec::new())
        }
        Msg::SaveFailed(failure) => {
            let text = match failure {
                SaveFailure::Conflict { id } => {
                    format!("{id} changed externally — press r to reload")
                }
                SaveFailure::Message(message) => message.clone(),
            };
            set_status_line(model, text, Severity::Error);
            Some(Vec::new())
        }
        Msg::EditorReturned(result) => {
            match result {
                Ok(()) => set_status_line(model, "Editor closed".to_string(), Severity::Info),
                Err(error) => set_status_line(model, error.clone(), Severity::Error),
            }
            Some(Vec::new())
        }
        _ => None,
    }
}

fn update_board(mut model: Model, mut state: BoardState, msg: Msg) -> (Model, Vec<Effect>) {
    let selected_column = state.selected_column;
    ensure_board_cursor(&model, &mut state, selected_column);

    let mut effects = Vec::new();
    match msg {
        Msg::MoveLeft if state.selected_column > 0 => {
            state.selected_column -= 1;
            let column = state.selected_column;
            ensure_board_cursor(&model, &mut state, column);
        }
        Msg::MoveRight if state.selected_column + 1 < BOARD_COLUMNS.len() => {
            state.selected_column += 1;
            let column = state.selected_column;
            ensure_board_cursor(&model, &mut state, column);
        }
        Msg::MoveUp => move_board_cursor(&model, &mut state, -1),
        Msg::MoveDown => move_board_cursor(&model, &mut state, 1),
        Msg::JumpTop => jump_board_cursor(&model, &mut state, true),
        Msg::JumpBottom => jump_board_cursor(&model, &mut state, false),
        Msg::HalfPageUp => page_board_cursor(&model, &mut state, -(HALF_PAGE_ROWS as isize)),
        Msg::HalfPageDown => page_board_cursor(&model, &mut state, HALF_PAGE_ROWS as isize),
        Msg::OpenDetail => {
            if let Some(issue) = selected_board_issue(&model, &state) {
                model.screens.push(Screen::IssueDetail(DetailState {
                    id: issue.id.clone(),
                    scroll: 0,
                }));
                return (model, effects);
            }
        }
        Msg::OpenCreateForm => {
            model
                .screens
                .push(Screen::CreateForm(CreateFormState::new(&model.config)));
            return (model, effects);
        }
        Msg::OpenHelp => {
            model.screens.push(Screen::Help(HelpState));
            return (model, effects);
        }
        Msg::RequestRefresh => effects.push(Effect::LoadIssues),
        _ => {}
    }

    replace_top_screen(&mut model, Screen::Board(state));
    (model, effects)
}

fn update_detail(mut model: Model, mut state: DetailState, msg: Msg) -> (Model, Vec<Effect>) {
    let mut effects = Vec::new();

    match msg {
        Msg::MoveUp => state.scroll = state.scroll.saturating_sub(1),
        Msg::MoveDown => state.scroll = state.scroll.saturating_add(1),
        Msg::JumpTop => state.scroll = 0,
        Msg::JumpBottom => state.scroll = detail_max_scroll(&model, &state),
        Msg::HalfPageUp => state.scroll = state.scroll.saturating_sub(DETAIL_HALF_PAGE),
        Msg::HalfPageDown => state.scroll = state.scroll.saturating_add(DETAIL_HALF_PAGE),
        Msg::EditCurrentIssue => effects.push(Effect::OpenEditorForIssue {
            id: state.id.clone(),
        }),
        Msg::OpenStatusPicker => {
            if let Some(issue) = find_issue(&model, &state.id) {
                let options = Status::ALL.to_vec();
                let selected = options
                    .iter()
                    .position(|status| issue.status == status.as_str())
                    .unwrap_or(0);
                model.screens.push(Screen::StatusPicker(PickerState {
                    issue_id: state.id.clone(),
                    options,
                    selected,
                }));
                return (model, effects);
            }
        }
        Msg::OpenPriorityPicker => {
            if let Some(issue) = find_issue(&model, &state.id) {
                let options = Priority::ALL.to_vec();
                let selected = options
                    .iter()
                    .position(|priority| {
                        priority_from_issue(issue) == Some(*priority)
                            || (issue.priority.is_none() && *priority == Priority::Normal)
                    })
                    .unwrap_or_else(|| {
                        options
                            .iter()
                            .position(|priority| *priority == Priority::Normal)
                            .unwrap_or(0)
                    });
                model
                    .screens
                    .push(Screen::PriorityPicker(PriorityPickerState {
                        issue_id: state.id.clone(),
                        options,
                        selected,
                    }));
                return (model, effects);
            }
        }
        Msg::PopScreen => {
            pop_screen(&mut model);
            return (model, effects);
        }
        _ => {}
    }

    replace_top_screen(&mut model, Screen::IssueDetail(state));
    (model, effects)
}

fn update_picker(mut model: Model, mut state: PickerState, msg: Msg) -> (Model, Vec<Effect>) {
    let mut effects = Vec::new();

    match msg {
        Msg::MoveUp => {
            state.selected = state.selected.saturating_sub(1);
        }
        Msg::MoveDown => {
            let max_index = state.options.len().saturating_sub(1);
            state.selected = (state.selected + 1).min(max_index);
        }
        Msg::JumpTop => state.selected = 0,
        Msg::JumpBottom => state.selected = state.options.len().saturating_sub(1),
        Msg::SubmitStatusChange => {
            if let Some(status) = state.options.get(state.selected).copied() {
                let etag = model
                    .etags
                    .get(&state.issue_id)
                    .cloned()
                    .unwrap_or_default();
                effects.push(Effect::SaveIssue {
                    patch: IssuePatch {
                        id: state.issue_id.clone(),
                        status: Some(status),
                        priority: None,
                    },
                    etag,
                });
                pop_screen(&mut model);
                return (model, effects);
            }
        }
        Msg::PopScreen => {
            pop_screen(&mut model);
            return (model, effects);
        }
        _ => {}
    }

    replace_top_screen(&mut model, Screen::StatusPicker(state));
    (model, effects)
}

fn update_priority_picker(
    mut model: Model,
    mut state: PriorityPickerState,
    msg: Msg,
) -> (Model, Vec<Effect>) {
    let mut effects = Vec::new();

    match msg {
        Msg::MoveUp => {
            state.selected = state.selected.saturating_sub(1);
        }
        Msg::MoveDown => {
            let max_index = state.options.len().saturating_sub(1);
            state.selected = (state.selected + 1).min(max_index);
        }
        Msg::JumpTop => state.selected = 0,
        Msg::JumpBottom => state.selected = state.options.len().saturating_sub(1),
        Msg::SubmitPriorityChange => {
            if let Some(priority) = state.options.get(state.selected).copied() {
                let etag = model
                    .etags
                    .get(&state.issue_id)
                    .cloned()
                    .unwrap_or_default();
                effects.push(Effect::SaveIssue {
                    patch: IssuePatch {
                        id: state.issue_id.clone(),
                        status: None,
                        priority: Some(priority),
                    },
                    etag,
                });
                pop_screen(&mut model);
                return (model, effects);
            }
        }
        Msg::PopScreen => {
            pop_screen(&mut model);
            return (model, effects);
        }
        _ => {}
    }

    replace_top_screen(&mut model, Screen::PriorityPicker(state));
    (model, effects)
}

fn update_create_form(
    mut model: Model,
    mut state: CreateFormState,
    msg: Msg,
) -> (Model, Vec<Effect>) {
    let mut effects = Vec::new();

    match msg {
        Msg::FocusNextField => {
            state.focused_field = (state.focused_field + 1) % CREATE_FORM_FIELD_COUNT;
            state.pending_cancel = false;
        }
        Msg::FocusPreviousField => {
            state.focused_field =
                (state.focused_field + CREATE_FORM_FIELD_COUNT - 1) % CREATE_FORM_FIELD_COUNT;
            state.pending_cancel = false;
        }
        Msg::CreateFormInput(edit) => {
            apply_form_edit(&mut state, edit);
            state.pending_cancel = false;
        }
        Msg::CreateFormCycleType(direction) => {
            state.ish_type = cycle_ish_type(state.ish_type, direction);
            state.pending_cancel = false;
        }
        Msg::CreateFormCyclePriority(direction) => {
            state.priority = cycle_priority(state.priority, direction);
            state.pending_cancel = false;
        }
        Msg::SubmitCreateForm => {
            if let Some(effect) = submit_create_form(&mut model, &state, false) {
                effects.push(effect);
                pop_screen(&mut model);
                return (model, effects);
            }
            return (model, effects);
        }
        Msg::SubmitCreateAndEdit => {
            if let Some(effect) = submit_create_form(&mut model, &state, true) {
                effects.push(effect);
                pop_screen(&mut model);
                return (model, effects);
            }
            return (model, effects);
        }
        Msg::SubmitCreateFormWithStatus(status) => {
            if let Some(effect) = submit_create_form_with_status(&mut model, &state, status, false)
            {
                effects.push(effect);
                pop_screen(&mut model);
                return (model, effects);
            }
            return (model, effects);
        }
        Msg::PopScreen => {
            if !create_form_dirty(&state) {
                clear_status_line(&mut model);
                pop_screen(&mut model);
                return (model, effects);
            }

            state.pending_cancel = true;
            set_status_line(
                &mut model,
                "Discard new issue? y/n".to_string(),
                Severity::Info,
            );
        }
        Msg::ConfirmDiscardCreateForm => {
            clear_status_line(&mut model);
            pop_screen(&mut model);
            return (model, effects);
        }
        Msg::CancelDiscardCreateForm => {
            state.pending_cancel = false;
            clear_status_line(&mut model);
        }
        _ => {}
    }

    replace_top_screen(&mut model, Screen::CreateForm(state));
    (model, effects)
}

fn update_help(mut model: Model, _state: HelpState, msg: Msg) -> (Model, Vec<Effect>) {
    if matches!(msg, Msg::PopScreen | Msg::OpenHelp) {
        pop_screen(&mut model);
    }
    (model, Vec::new())
}

fn submit_create_form(
    model: &mut Model,
    state: &CreateFormState,
    open_in_editor: bool,
) -> Option<Effect> {
    submit_create_form_with_status(model, state, default_create_status(model), open_in_editor)
}

fn submit_create_form_with_status(
    model: &mut Model,
    state: &CreateFormState,
    status: Status,
    open_in_editor: bool,
) -> Option<Effect> {
    let title = state.title.trim();
    if title.is_empty() {
        set_status_line(model, "Title is required".to_string(), Severity::Error);
        return None;
    }

    Some(Effect::CreateIssue {
        draft: IssueDraft {
            title: title.to_string(),
            status,
            ish_type: state.ish_type,
            priority: state.priority,
            tags: parse_tags(&state.tags),
            body: String::new(),
            parent: None,
            blocking: Vec::new(),
            blocked_by: Vec::new(),
        },
        open_in_editor,
    })
}

fn apply_form_edit(state: &mut CreateFormState, edit: crate::tui::FormFieldEdit) {
    let target = match state.focused_field {
        0 => Some(&mut state.title),
        3 => Some(&mut state.tags),
        _ => None,
    };

    let Some(target) = target else {
        return;
    };

    match edit {
        crate::tui::FormFieldEdit::Insert(ch) => target.push(ch),
        crate::tui::FormFieldEdit::Backspace => {
            target.pop();
        }
        crate::tui::FormFieldEdit::Clear => target.clear(),
    }
}

fn create_form_dirty(state: &CreateFormState) -> bool {
    !state.title.trim().is_empty() || !state.tags.trim().is_empty()
}

fn default_create_status(model: &Model) -> Status {
    Status::from_str(&model.config.ish.default_status).unwrap_or(Status::Todo)
}

fn cycle_ish_type(current: crate::tui::IshType, direction: i32) -> crate::tui::IshType {
    cycle_enum(current, &crate::tui::IshType::ALL, direction)
}

fn cycle_priority(current: Priority, direction: i32) -> Priority {
    cycle_enum(current, &Priority::ALL, direction)
}

fn cycle_enum<T: Copy + PartialEq>(current: T, all: &[T], direction: i32) -> T {
    let index = all
        .iter()
        .position(|candidate| *candidate == current)
        .unwrap_or(0) as i32;
    let len = all.len() as i32;
    let next = (index + direction).rem_euclid(len) as usize;
    all[next]
}

fn parse_tags(tags: &str) -> Vec<String> {
    let mut parsed = Vec::new();

    for tag in tags
        .split(',')
        .map(normalize_tag)
        .filter(|tag| !tag.is_empty())
    {
        if !parsed.iter().any(|existing| existing == &tag) {
            parsed.push(tag);
        }
    }

    parsed
}

fn selected_board_issue<'a>(model: &'a Model, state: &BoardState) -> Option<&'a Ish> {
    let column = state.selected_column;
    let cursor = state
        .column_cursors
        .get(column)
        .and_then(|cursor| *cursor)?;
    model
        .bucket_for_status(BOARD_COLUMNS[column])
        .into_iter()
        .nth(cursor)
        .map(|row| row.ish)
}

fn find_issue<'a>(model: &'a Model, id: &str) -> Option<&'a Ish> {
    model.issues.iter().find(|issue| issue.id == id)
}

fn priority_from_issue(issue: &Ish) -> Option<Priority> {
    issue.priority.as_deref().and_then(Priority::from_str)
}

fn detail_max_scroll(model: &Model, state: &DetailState) -> u16 {
    find_issue(model, &state.id)
        .map(|issue| issue.body.lines().count().saturating_sub(1) as u16)
        .unwrap_or(0)
}

fn move_board_cursor(model: &Model, state: &mut BoardState, delta: isize) {
    let column = state.selected_column;
    let bucket_len = model.bucket_for_status(BOARD_COLUMNS[column]).len();
    if bucket_len == 0 {
        state.column_cursors[column] = None;
        state.column_offsets[column] = 0;
        return;
    }

    let current = state.column_cursors[column].unwrap_or(0) as isize;
    let next = (current + delta).clamp(0, bucket_len.saturating_sub(1) as isize) as usize;
    state.column_cursors[column] = Some(next);
    keep_cursor_visible(state, column, bucket_len);
}

fn jump_board_cursor(model: &Model, state: &mut BoardState, to_top: bool) {
    let column = state.selected_column;
    let bucket_len = model.bucket_for_status(BOARD_COLUMNS[column]).len();
    if bucket_len == 0 {
        state.column_cursors[column] = None;
        state.column_offsets[column] = 0;
        return;
    }

    state.column_cursors[column] = Some(if to_top { 0 } else { bucket_len - 1 });
    keep_cursor_visible(state, column, bucket_len);
}

fn page_board_cursor(model: &Model, state: &mut BoardState, delta: isize) {
    move_board_cursor(model, state, delta);
}

fn ensure_board_cursor(model: &Model, state: &mut BoardState, column: usize) {
    let bucket_len = model.bucket_for_status(BOARD_COLUMNS[column]).len();
    if bucket_len == 0 {
        state.column_cursors[column] = None;
        state.column_offsets[column] = 0;
        return;
    }

    let cursor = state.column_cursors[column]
        .unwrap_or(0)
        .min(bucket_len - 1);
    state.column_cursors[column] = Some(cursor);
    keep_cursor_visible(state, column, bucket_len);
}

fn keep_cursor_visible(state: &mut BoardState, column: usize, bucket_len: usize) {
    let Some(cursor) = state.column_cursors[column] else {
        state.column_offsets[column] = 0;
        return;
    };

    let max_offset = bucket_len.saturating_sub(BOARD_VISIBLE_ROWS);
    let mut offset = state.column_offsets[column].min(max_offset);

    if cursor < offset {
        offset = cursor;
    } else if cursor >= offset + BOARD_VISIBLE_ROWS {
        offset = cursor + 1 - BOARD_VISIBLE_ROWS;
    }

    state.column_offsets[column] = offset.min(max_offset);
}

fn clamp_board_state(model: &mut Model) {
    let bucket_lengths = BOARD_COLUMNS.map(|status| model.bucket_for_status(status).len());

    for screen in &mut model.screens {
        if let Screen::Board(state) = screen {
            state.selected_column = state
                .selected_column
                .min(BOARD_COLUMNS.len().saturating_sub(1));
            for (column, bucket_len) in bucket_lengths.iter().copied().enumerate() {
                if bucket_len == 0 {
                    state.column_cursors[column] = None;
                    state.column_offsets[column] = 0;
                    continue;
                }

                let cursor = state.column_cursors[column]
                    .unwrap_or(0)
                    .min(bucket_len - 1);
                state.column_cursors[column] = Some(cursor);
                keep_cursor_visible(state, column, bucket_len);
            }
        }
    }
}

fn pop_screen(model: &mut Model) {
    if model.screens.len() > 1 {
        model.screens.pop();
    }
}

fn replace_top_screen(model: &mut Model, screen: Screen) {
    if let Some(slot) = model.screens.last_mut() {
        *slot = screen;
    } else {
        model.screens.push(screen);
    }
}

fn expire_status_line(model: &mut Model) {
    let Some(status_line) = &model.status_line else {
        return;
    };
    let Some(set_at) = model.status_line_set_at else {
        return;
    };

    if matches!(status_line.severity, Severity::Info | Severity::Success)
        && set_at.elapsed() >= STATUS_LINE_TTL
    {
        clear_status_line(model);
    }
}

fn clear_status_line(model: &mut Model) {
    model.status_line = None;
    model.status_line_set_at = None;
}

fn set_status_line(model: &mut Model, text: String, severity: Severity) {
    if let (Some(existing), Some(set_at)) = (&model.status_line, model.status_line_set_at)
        && existing.severity == Severity::Error
        && severity != Severity::Error
        && set_at.elapsed() < ERROR_STICKY_TTL
    {
        return;
    }

    model.status_line = Some(StatusLine { text, severity });
    model.status_line_set_at = Some(Instant::now());
}

#[cfg(test)]
mod tests {
    use super::{ERROR_STICKY_TTL, STATUS_LINE_TTL, update};
    use crate::test_support::tui::{IshBuilder, dispatch, model_with_board};
    use crate::tui::{
        BoardState, CreateFormState, DetailState, Effect, HelpState, IssuePatch, Model, Msg,
        PickerState, Priority, PriorityPickerState, SaveFailure, SaveSuccess, Screen, Severity,
        Status, StatusLine,
    };
    use std::time::{Duration, Instant};

    fn board_state(model: &Model) -> &BoardState {
        match model.screens.first() {
            Some(Screen::Board(state)) => state,
            other => panic!("expected board screen, got {other:?}"),
        }
    }

    fn top_screen(model: &Model) -> &Screen {
        model
            .screens
            .last()
            .expect("screen stack should not be empty")
    }

    fn todo_model(todo_count: usize) -> Model {
        let mut issues = vec![
            IshBuilder::new("draft")
                .status("draft")
                .title("Draft")
                .build(),
            IshBuilder::new("progress")
                .status("in-progress")
                .title("Doing")
                .build(),
            IshBuilder::new("done")
                .status("completed")
                .title("Done")
                .build(),
        ];

        for index in 0..todo_count {
            issues.push(
                IshBuilder::new(&format!("todo-{index}"))
                    .status("todo")
                    .title(&format!("Todo {index}"))
                    .build(),
            );
        }

        model_with_board(issues)
    }

    #[test]
    fn empty_board_tick_smoke_test() {
        let model = model_with_board(vec![]);
        let (model, effects) = update(model, Msg::Tick);

        assert!(!model.quit);
        assert!(effects.is_empty());
    }

    #[test]
    fn resize_toggles_terminal_too_small_flag() {
        let model = model_with_board(vec![]);
        let (model, effects) = update(model, Msg::Resize(40, 10));

        assert!(effects.is_empty());
        assert!(model.term_too_small);

        let (model, effects) = update(model, Msg::Resize(120, 30));

        assert!(effects.is_empty());
        assert!(!model.term_too_small);
    }

    #[test]
    fn board_navigation_stops_at_edges_remembers_columns_and_pages_by_half_screen() {
        let model = todo_model(6);
        let (model, effects) = dispatch(
            model,
            &[
                Msg::MoveLeft,
                Msg::MoveRight,
                Msg::MoveDown,
                Msg::MoveDown,
                Msg::MoveDown,
                Msg::MoveRight,
                Msg::MoveLeft,
                Msg::HalfPageUp,
                Msg::JumpBottom,
                Msg::MoveDown,
                Msg::HalfPageDown,
                Msg::JumpTop,
                Msg::MoveUp,
            ],
        );

        let board = board_state(&model);
        assert!(effects.is_empty());
        assert_eq!(board.selected_column, 1);
        assert_eq!(board.column_cursors[0], Some(0));
        assert_eq!(board.column_cursors[1], Some(0));
        assert_eq!(board.column_cursors[2], Some(0));
        assert_eq!(board.column_offsets[1], 0);
    }

    #[test]
    fn empty_columns_are_navigable_but_do_not_open_detail() {
        let model = model_with_board(vec![IshBuilder::new("todo").status("todo").build()]);
        let (model, effects) = dispatch(model, &[Msg::OpenDetail, Msg::MoveRight, Msg::MoveLeft]);

        assert!(effects.is_empty());
        assert!(matches!(model.screens.as_slice(), [Screen::Board(_)]));
        let board = board_state(&model);
        assert_eq!(board.selected_column, 0);
        assert_eq!(board.column_cursors[0], None);
        assert_eq!(board.column_cursors[1], Some(0));
    }

    #[test]
    fn issues_loaded_replaces_cache_and_clamps_board_cursors() {
        let mut model = todo_model(3);
        model.screens = vec![Screen::Board(BoardState {
            selected_column: 1,
            column_cursors: [Some(0), Some(2), Some(0), Some(0)],
            column_offsets: [0, 2, 0, 0],
        })];

        let new_issues = vec![
            IshBuilder::new("todo-keep")
                .status("todo")
                .title("Keep me")
                .build(),
            IshBuilder::new("scrapped")
                .status("scrapped")
                .title("Ignore me")
                .build(),
            IshBuilder::new("archived")
                .status("todo")
                .title("Archived")
                .path("archive/ish-archived--archived.md")
                .build(),
        ];

        let (model, effects) = update(model, Msg::IssuesLoaded(Ok(new_issues)));

        assert!(effects.is_empty());
        let board = board_state(&model);
        assert_eq!(board.selected_column, 1);
        assert_eq!(board.column_cursors[1], Some(0));
        assert_eq!(board.column_offsets[1], 0);
        assert_eq!(model.bucket_for_status(Status::Todo).len(), 1);
        assert!(model.bucket_for_status(Status::Completed).is_empty());
    }

    #[test]
    fn screen_transitions_open_detail_picker_create_form_and_pop_cleanly() {
        let model = model_with_board(vec![IshBuilder::new("todo").status("todo").build()]);
        let (model, effects) = dispatch(
            model,
            &[
                Msg::MoveRight,
                Msg::OpenDetail,
                Msg::OpenStatusPicker,
                Msg::PopScreen,
                Msg::PopScreen,
                Msg::OpenHelp,
                Msg::PopScreen,
                Msg::OpenCreateForm,
            ],
        );

        assert!(effects.is_empty());
        assert!(matches!(top_screen(&model), Screen::CreateForm(_)));
        assert_eq!(model.screens.len(), 2);
    }

    #[test]
    fn submit_status_change_emits_save_effect_with_current_etag_and_pops_picker() {
        let mut model = model_with_board(vec![IshBuilder::new("todo").status("todo").build()]);
        model.screens = vec![
            Screen::Board(BoardState {
                selected_column: 1,
                column_cursors: [None, Some(0), None, None],
                column_offsets: [0; 4],
            }),
            Screen::IssueDetail(DetailState {
                id: "ish-todo".to_string(),
                scroll: 0,
            }),
            Screen::StatusPicker(PickerState {
                issue_id: "ish-todo".to_string(),
                options: Status::ALL.to_vec(),
                selected: 2,
            }),
        ];
        let expected_etag = model.etags.get("ish-todo").cloned().unwrap();

        let (model, effects) = update(model, Msg::SubmitStatusChange);

        assert_eq!(model.screens.len(), 2);
        assert!(matches!(top_screen(&model), Screen::IssueDetail(_)));
        assert_eq!(
            effects,
            vec![Effect::SaveIssue {
                patch: IssuePatch {
                    id: "ish-todo".to_string(),
                    status: Some(Status::InProgress),
                    priority: None,
                },
                etag: expected_etag,
            }]
        );
    }

    #[test]
    fn priority_picker_defaults_missing_priority_to_normal_and_submits_priority_save() {
        let mut model = model_with_board(vec![IshBuilder::new("todo").status("todo").build()]);
        model.screens = vec![
            Screen::Board(BoardState {
                selected_column: 1,
                column_cursors: [None, Some(0), None, None],
                column_offsets: [0; 4],
            }),
            Screen::IssueDetail(DetailState {
                id: "ish-todo".to_string(),
                scroll: 0,
            }),
        ];
        let expected_etag = model.etags.get("ish-todo").cloned().unwrap();

        let (model, effects) = update(model, Msg::OpenPriorityPicker);

        assert!(effects.is_empty());
        match top_screen(&model) {
            Screen::PriorityPicker(PriorityPickerState { selected, .. }) => {
                assert_eq!(*selected, 2);
            }
            other => panic!("expected priority picker, got {other:?}"),
        }

        let (model, effects) = update(model, Msg::SubmitPriorityChange);

        assert_eq!(model.screens.len(), 2);
        assert!(matches!(top_screen(&model), Screen::IssueDetail(_)));
        assert_eq!(
            effects,
            vec![Effect::SaveIssue {
                patch: IssuePatch {
                    id: "ish-todo".to_string(),
                    status: None,
                    priority: Some(Priority::Normal),
                },
                etag: expected_etag,
            }]
        );
    }

    #[test]
    fn save_messages_update_status_line_without_mutating_screen_stack() {
        let mut model = model_with_board(vec![IshBuilder::new("todo").status("todo").build()]);
        model.screens = vec![
            Screen::Board(BoardState::default()),
            Screen::IssueDetail(DetailState {
                id: "ish-todo".to_string(),
                scroll: 0,
            }),
        ];

        let (model, save_effects) = update(
            model,
            Msg::SaveCompleted(SaveSuccess {
                id: "ish-todo".to_string(),
            }),
        );
        assert!(save_effects.is_empty());
        assert_eq!(model.screens.len(), 2);
        assert_eq!(
            model.status_line,
            Some(StatusLine {
                text: "Saved ish-todo".to_string(),
                severity: Severity::Success,
            })
        );

        let (model, conflict_effects) = update(
            model,
            Msg::SaveFailed(SaveFailure::Conflict {
                id: "ish-todo".to_string(),
            }),
        );
        assert!(conflict_effects.is_empty());
        assert_eq!(model.screens.len(), 2);
        assert_eq!(
            model.status_line,
            Some(StatusLine {
                text: "ish-todo changed externally — press r to reload".to_string(),
                severity: Severity::Error,
            })
        );
    }

    #[test]
    fn status_line_tick_dismiss_and_error_stickiness_follow_the_prd_rules() {
        let mut info_model = model_with_board(vec![]);
        info_model.status_line = Some(StatusLine {
            text: "Refreshed".to_string(),
            severity: Severity::Info,
        });
        info_model.status_line_set_at =
            Some(Instant::now() - STATUS_LINE_TTL - Duration::from_millis(1));
        let (info_model, _) = update(info_model, Msg::Tick);
        assert!(info_model.status_line.is_none());

        let mut error_model = model_with_board(vec![]);
        error_model.status_line = Some(StatusLine {
            text: "Conflict".to_string(),
            severity: Severity::Error,
        });
        error_model.status_line_set_at =
            Some(Instant::now() - STATUS_LINE_TTL - Duration::from_millis(1));
        let (error_model, _) = update(error_model, Msg::Tick);
        assert_eq!(
            error_model.status_line,
            Some(StatusLine {
                text: "Conflict".to_string(),
                severity: Severity::Error,
            })
        );

        let mut sticky_model = error_model.clone();
        sticky_model.status_line_set_at = Some(Instant::now());
        let (sticky_model, _) = update(
            sticky_model,
            Msg::SaveCompleted(SaveSuccess {
                id: "ish-todo".to_string(),
            }),
        );
        assert_eq!(
            sticky_model.status_line,
            Some(StatusLine {
                text: "Conflict".to_string(),
                severity: Severity::Error,
            })
        );

        let mut replaceable_model = sticky_model;
        replaceable_model.status_line_set_at =
            Some(Instant::now() - ERROR_STICKY_TTL - Duration::from_millis(1));
        let (replaceable_model, _) = update(
            replaceable_model,
            Msg::SaveCompleted(SaveSuccess {
                id: "ish-todo".to_string(),
            }),
        );
        assert_eq!(
            replaceable_model.status_line,
            Some(StatusLine {
                text: "Saved ish-todo".to_string(),
                severity: Severity::Success,
            })
        );

        let (dismissed_model, _) = update(error_model, Msg::DismissStatusLine);
        assert!(dismissed_model.status_line.is_none());
    }

    #[test]
    fn create_form_cycles_fields_wraps_values_and_confirms_before_discard() {
        let mut model = model_with_board(vec![]);
        model.screens = vec![
            Screen::Board(BoardState::default()),
            Screen::CreateForm(CreateFormState::new(&model.config)),
        ];

        let (model, effects) = dispatch(
            model,
            &[
                Msg::CreateFormInput(crate::tui::FormFieldEdit::Insert('A')),
                Msg::CreateFormInput(crate::tui::FormFieldEdit::Insert('B')),
                Msg::CreateFormInput(crate::tui::FormFieldEdit::Backspace),
                Msg::FocusNextField,
                Msg::FocusNextField,
                Msg::FocusNextField,
                Msg::FocusNextField,
                Msg::FocusNextField,
                Msg::CreateFormCycleType(1),
                Msg::CreateFormCyclePriority(-1),
                Msg::PopScreen,
            ],
        );

        assert!(effects.is_empty());
        match top_screen(&model) {
            Screen::CreateForm(state) => {
                assert_eq!(state.title, "A");
                assert_eq!(state.focused_field, 0);
                assert!(state.pending_cancel);
                assert_eq!(state.ish_type.as_str(), "milestone");
                assert_eq!(state.priority.as_str(), "high");
            }
            other => panic!("expected create form, got {other:?}"),
        }
        assert_eq!(
            model.status_line,
            Some(StatusLine {
                text: "Discard new issue? y/n".to_string(),
                severity: Severity::Info,
            })
        );

        let (model, effects) = update(model, Msg::CancelDiscardCreateForm);
        assert!(effects.is_empty());
        assert!(model.status_line.is_none());
        match top_screen(&model) {
            Screen::CreateForm(state) => assert!(!state.pending_cancel),
            other => panic!("expected create form, got {other:?}"),
        }

        let (model, effects) = update(model, Msg::ConfirmDiscardCreateForm);
        assert!(effects.is_empty());
        assert!(model.status_line.is_none());
        assert!(matches!(top_screen(&model), Screen::Board(_)));
    }

    #[test]
    fn quit_sets_model_quit_even_with_modal_screen_open() {
        let mut model = model_with_board(vec![]);
        model.screens = vec![
            Screen::Board(BoardState::default()),
            Screen::Help(HelpState),
        ];

        let (model, effects) = update(model, Msg::Quit);

        assert!(effects.is_empty());
        assert!(model.quit);
        assert_eq!(model.screens.len(), 2);
    }
}

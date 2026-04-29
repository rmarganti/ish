use super::{ERROR_STICKY_TTL, STATUS_LINE_TTL, update};
use crate::test_support::tui::{IshBuilder, dispatch, model_with_board};
use crate::tui::{
    BoardState, CreateFormState, DetailState, Effect, HelpState, IssuePatch, Model, Msg,
    PickerState, Priority, PriorityPickerState, SaveFailure, SaveSuccess, Screen, Severity, Status,
    StatusLine,
};
use crossterm::event::KeyCode;
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

fn selected_board_issue_id(model: &Model) -> Option<&str> {
    let state = board_state(model);
    let cursor = state.column_cursors[state.selected_column]?;
    model
        .bucket_for_status(crate::tui::BOARD_COLUMNS[state.selected_column])
        .get(cursor)
        .map(|row| row.ish.id.as_str())
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
fn key_pressed_path_tracks_pending_prefixes_and_dispatches_gg() {
    let model = todo_model(6);
    let (model, effects) = dispatch(
        model,
        &[
            Msg::MoveRight,
            Msg::MoveDown,
            Msg::MoveDown,
            Msg::KeyPressed(crate::k!(KeyCode::Char('g'))),
        ],
    );

    assert!(effects.is_empty());
    assert_eq!(model.input.pending_keys.len(), 1);
    assert_eq!(board_state(&model).column_cursors[1], Some(2));

    let (model, effects) = update(model, Msg::KeyPressed(crate::k!(KeyCode::Char('g'))));

    assert!(effects.is_empty());
    assert!(model.input.pending_keys.is_empty());
    assert_eq!(board_state(&model).column_cursors[1], Some(0));
}

#[test]
fn invalid_key_sequence_clears_pending_and_retries_current_key() {
    let model = todo_model(2);
    let (model, effects) = dispatch(
        model,
        &[
            Msg::MoveRight,
            Msg::MoveDown,
            Msg::KeyPressed(crate::k!(KeyCode::Char('g'))),
            Msg::KeyPressed(crate::k!(KeyCode::Char('j'))),
        ],
    );

    assert!(effects.is_empty());
    assert!(model.input.pending_keys.is_empty());
    assert_eq!(board_state(&model).column_cursors[1], Some(1));

    let (model, effects) = dispatch(
        model,
        &[
            Msg::KeyPressed(crate::k!(KeyCode::Char('g'))),
            Msg::KeyPressed(crate::k!(KeyCode::Char('x'))),
        ],
    );

    assert!(effects.is_empty());
    assert!(model.input.pending_keys.is_empty());
    assert_eq!(board_state(&model).column_cursors[1], Some(1));
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
fn go_to_parent_from_detail_replaces_the_current_detail_screen() {
    let mut model = model_with_board(vec![
        IshBuilder::new("parent").status("todo").build(),
        IshBuilder::new("child")
            .status("todo")
            .parent("ish-parent")
            .build(),
    ]);
    model.screens = vec![
        Screen::Board(BoardState::default()),
        Screen::IssueDetail(DetailState {
            id: "ish-child".to_string(),
            scroll: 7,
        }),
    ];

    let (model, effects) = update(model, Msg::GoToParent);

    assert!(effects.is_empty());
    assert_eq!(model.screens.len(), 2);
    assert_eq!(
        top_screen(&model),
        &Screen::IssueDetail(DetailState {
            id: "ish-parent".to_string(),
            scroll: 0,
        })
    );
}

#[test]
fn go_to_parent_from_detail_without_a_parent_sets_a_status_message() {
    let mut model = model_with_board(vec![IshBuilder::new("solo").status("todo").build()]);
    model.screens = vec![
        Screen::Board(BoardState::default()),
        Screen::IssueDetail(DetailState {
            id: "ish-solo".to_string(),
            scroll: 3,
        }),
    ];

    let (model, effects) = update(model, Msg::GoToParent);

    assert!(effects.is_empty());
    assert_eq!(
        top_screen(&model),
        &Screen::IssueDetail(DetailState {
            id: "ish-solo".to_string(),
            scroll: 3,
        })
    );
    assert_eq!(
        model.status_line,
        Some(StatusLine {
            text: "ish-solo has no parent".to_string(),
            severity: Severity::Info,
        })
    );
}

#[test]
fn go_to_parent_from_board_selects_the_parent_column_and_row() {
    let mut model = model_with_board(vec![
        IshBuilder::new("parent")
            .status("in-progress")
            .updated_at(2026, 1, 2)
            .build(),
        IshBuilder::new("sibling")
            .status("in-progress")
            .updated_at(2026, 1, 1)
            .build(),
        IshBuilder::new("child")
            .status("todo")
            .parent("ish-parent")
            .build(),
    ]);
    model.screens = vec![Screen::Board(BoardState {
        selected_column: 1,
        column_cursors: [None, Some(0), Some(1), None],
        column_offsets: [0; 4],
    })];

    let (model, effects) = update(model, Msg::GoToParent);

    assert!(effects.is_empty());
    let board = board_state(&model);
    assert_eq!(board.selected_column, 2);
    assert_eq!(board.column_cursors[2], Some(0));
    assert_eq!(selected_board_issue_id(&model), Some("ish-parent"));
}

#[test]
fn go_to_parent_from_board_without_a_parent_keeps_selection_stable_and_sets_status() {
    let mut model = model_with_board(vec![IshBuilder::new("solo").status("todo").build()]);
    model.screens = vec![Screen::Board(BoardState {
        selected_column: 1,
        column_cursors: [None, Some(0), None, None],
        column_offsets: [0; 4],
    })];

    let (model, effects) = update(model, Msg::GoToParent);

    assert!(effects.is_empty());
    let board = board_state(&model);
    assert_eq!(board.selected_column, 1);
    assert_eq!(board.column_cursors[1], Some(0));
    assert_eq!(selected_board_issue_id(&model), Some("ish-solo"));
    assert_eq!(
        model.status_line,
        Some(StatusLine {
            text: "ish-solo has no parent".to_string(),
            severity: Severity::Info,
        })
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

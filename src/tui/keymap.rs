#![allow(dead_code)]

use crate::tui::{FormFieldEdit, Msg, Screen};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

pub fn map_key(screen: &Screen, key: KeyEvent) -> Option<Msg> {
    if is_ctrl_char(&key, 'c') {
        return Some(Msg::Quit);
    }

    if !matches!(screen, Screen::Help(_))
        && key == KeyEvent::new(KeyCode::Char('?'), KeyModifiers::NONE)
    {
        return Some(Msg::OpenHelp);
    }

    match screen {
        Screen::Board(_) => map_board_key(key),
        Screen::IssueDetail(_) => map_detail_key(key),
        Screen::StatusPicker(_) => map_picker_key(key),
        Screen::CreateForm(state) => map_create_form_key(state, key),
        Screen::Help(_) => Some(Msg::PopScreen),
    }
}

fn map_board_key(key: KeyEvent) -> Option<Msg> {
    match key {
        KeyEvent {
            code: KeyCode::Char('h'),
            modifiers: KeyModifiers::NONE,
            ..
        }
        | KeyEvent {
            code: KeyCode::Left,
            modifiers: KeyModifiers::NONE,
            ..
        } => Some(Msg::MoveLeft),
        KeyEvent {
            code: KeyCode::Char('j'),
            modifiers: KeyModifiers::NONE,
            ..
        }
        | KeyEvent {
            code: KeyCode::Down,
            modifiers: KeyModifiers::NONE,
            ..
        } => Some(Msg::MoveDown),
        KeyEvent {
            code: KeyCode::Char('k'),
            modifiers: KeyModifiers::NONE,
            ..
        }
        | KeyEvent {
            code: KeyCode::Up,
            modifiers: KeyModifiers::NONE,
            ..
        } => Some(Msg::MoveUp),
        KeyEvent {
            code: KeyCode::Char('l'),
            modifiers: KeyModifiers::NONE,
            ..
        }
        | KeyEvent {
            code: KeyCode::Right,
            modifiers: KeyModifiers::NONE,
            ..
        } => Some(Msg::MoveRight),
        KeyEvent {
            code: KeyCode::Char('g'),
            modifiers: KeyModifiers::NONE,
            ..
        }
        | KeyEvent {
            code: KeyCode::Home,
            modifiers: KeyModifiers::NONE,
            ..
        } => Some(Msg::JumpTop),
        KeyEvent {
            code: KeyCode::Char('G'),
            modifiers: KeyModifiers::SHIFT,
            ..
        }
        | KeyEvent {
            code: KeyCode::End,
            modifiers: KeyModifiers::NONE,
            ..
        } => Some(Msg::JumpBottom),
        key if is_ctrl_char(&key, 'd') => Some(Msg::HalfPageDown),
        key if is_ctrl_char(&key, 'u') => Some(Msg::HalfPageUp),
        KeyEvent {
            code: KeyCode::Enter,
            modifiers: KeyModifiers::NONE,
            ..
        }
        | KeyEvent {
            code: KeyCode::Char(' '),
            modifiers: KeyModifiers::NONE,
            ..
        } => Some(Msg::OpenDetail),
        KeyEvent {
            code: KeyCode::Char('c'),
            modifiers: KeyModifiers::NONE,
            ..
        } => Some(Msg::OpenCreateForm),
        KeyEvent {
            code: KeyCode::Char('r'),
            modifiers: KeyModifiers::NONE,
            ..
        } => Some(Msg::RequestRefresh),
        KeyEvent {
            code: KeyCode::Char('q'),
            modifiers: KeyModifiers::NONE,
            ..
        } => Some(Msg::Quit),
        _ => None,
    }
}

fn map_detail_key(key: KeyEvent) -> Option<Msg> {
    match key {
        KeyEvent {
            code: KeyCode::Char('j'),
            modifiers: KeyModifiers::NONE,
            ..
        }
        | KeyEvent {
            code: KeyCode::Down,
            modifiers: KeyModifiers::NONE,
            ..
        } => Some(Msg::MoveDown),
        KeyEvent {
            code: KeyCode::Char('k'),
            modifiers: KeyModifiers::NONE,
            ..
        }
        | KeyEvent {
            code: KeyCode::Up,
            modifiers: KeyModifiers::NONE,
            ..
        } => Some(Msg::MoveUp),
        KeyEvent {
            code: KeyCode::Char('g'),
            modifiers: KeyModifiers::NONE,
            ..
        }
        | KeyEvent {
            code: KeyCode::Home,
            modifiers: KeyModifiers::NONE,
            ..
        } => Some(Msg::JumpTop),
        KeyEvent {
            code: KeyCode::Char('G'),
            modifiers: KeyModifiers::SHIFT,
            ..
        }
        | KeyEvent {
            code: KeyCode::End,
            modifiers: KeyModifiers::NONE,
            ..
        } => Some(Msg::JumpBottom),
        key if is_ctrl_char(&key, 'd') => Some(Msg::HalfPageDown),
        key if is_ctrl_char(&key, 'u') => Some(Msg::HalfPageUp),
        KeyEvent {
            code: KeyCode::Char('e'),
            modifiers: KeyModifiers::NONE,
            ..
        } => Some(Msg::EditCurrentIssue),
        KeyEvent {
            code: KeyCode::Char('s'),
            modifiers: KeyModifiers::NONE,
            ..
        } => Some(Msg::OpenStatusPicker),
        KeyEvent {
            code: KeyCode::Char('q'),
            modifiers: KeyModifiers::NONE,
            ..
        }
        | KeyEvent {
            code: KeyCode::Esc,
            modifiers: KeyModifiers::NONE,
            ..
        } => Some(Msg::PopScreen),
        _ => None,
    }
}

fn map_picker_key(key: KeyEvent) -> Option<Msg> {
    match key {
        KeyEvent {
            code: KeyCode::Char('j'),
            modifiers: KeyModifiers::NONE,
            ..
        }
        | KeyEvent {
            code: KeyCode::Down,
            modifiers: KeyModifiers::NONE,
            ..
        } => Some(Msg::MoveDown),
        key if is_ctrl_char(&key, 'n') => Some(Msg::MoveDown),
        KeyEvent {
            code: KeyCode::Char('k'),
            modifiers: KeyModifiers::NONE,
            ..
        }
        | KeyEvent {
            code: KeyCode::Up,
            modifiers: KeyModifiers::NONE,
            ..
        } => Some(Msg::MoveUp),
        key if is_ctrl_char(&key, 'p') => Some(Msg::MoveUp),
        KeyEvent {
            code: KeyCode::Enter,
            modifiers: KeyModifiers::NONE,
            ..
        } => Some(Msg::SubmitStatusChange),
        KeyEvent {
            code: KeyCode::Char('q'),
            modifiers: KeyModifiers::NONE,
            ..
        }
        | KeyEvent {
            code: KeyCode::Esc,
            modifiers: KeyModifiers::NONE,
            ..
        } => Some(Msg::PopScreen),
        _ => None,
    }
}

fn map_create_form_key(state: &crate::tui::CreateFormState, key: KeyEvent) -> Option<Msg> {
    if state.pending_cancel {
        return match key {
            KeyEvent {
                code: KeyCode::Char('y'),
                modifiers: KeyModifiers::NONE,
                ..
            }
            | KeyEvent {
                code: KeyCode::Char('Y'),
                modifiers: KeyModifiers::SHIFT,
                ..
            } => Some(Msg::ConfirmDiscardCreateForm),
            KeyEvent {
                code: KeyCode::Char('n'),
                modifiers: KeyModifiers::NONE,
                ..
            }
            | KeyEvent {
                code: KeyCode::Char('N'),
                modifiers: KeyModifiers::SHIFT,
                ..
            }
            | KeyEvent {
                code: KeyCode::Esc,
                modifiers: KeyModifiers::NONE,
                ..
            } => Some(Msg::CancelDiscardCreateForm),
            _ => None,
        };
    }

    let focused_field = state.focused_field;
    match key {
        KeyEvent {
            code: KeyCode::Tab,
            modifiers: KeyModifiers::NONE,
            ..
        } => Some(Msg::FocusNextField),
        key if is_ctrl_char(&key, 'n') => Some(Msg::FocusNextField),
        KeyEvent {
            code: KeyCode::BackTab,
            modifiers: KeyModifiers::SHIFT,
            ..
        } => Some(Msg::FocusPreviousField),
        key if is_ctrl_char(&key, 'p') => Some(Msg::FocusPreviousField),
        key if is_ctrl_char(&key, 's') => Some(Msg::SubmitCreateForm),
        key if is_ctrl_char(&key, 'e') => Some(Msg::SubmitCreateAndEdit),
        KeyEvent {
            code: KeyCode::Esc,
            modifiers: KeyModifiers::NONE,
            ..
        } => Some(Msg::PopScreen),
        KeyEvent {
            code: KeyCode::Enter,
            modifiers: KeyModifiers::NONE,
            ..
        } if focused_field == 4 => Some(Msg::SubmitCreateForm),
        KeyEvent {
            code: KeyCode::Left,
            modifiers: KeyModifiers::NONE,
            ..
        } if focused_field == 1 => Some(Msg::CreateFormCycleType(-1)),
        KeyEvent {
            code: KeyCode::Right,
            modifiers: KeyModifiers::NONE,
            ..
        } if focused_field == 1 => Some(Msg::CreateFormCycleType(1)),
        KeyEvent {
            code: KeyCode::Left,
            modifiers: KeyModifiers::NONE,
            ..
        } if focused_field == 2 => Some(Msg::CreateFormCyclePriority(-1)),
        KeyEvent {
            code: KeyCode::Right,
            modifiers: KeyModifiers::NONE,
            ..
        } if focused_field == 2 => Some(Msg::CreateFormCyclePriority(1)),
        KeyEvent {
            code: KeyCode::Char('h'),
            modifiers: KeyModifiers::NONE,
            ..
        } if focused_field == 1 => Some(Msg::CreateFormCycleType(-1)),
        KeyEvent {
            code: KeyCode::Char('l'),
            modifiers: KeyModifiers::NONE,
            ..
        } if focused_field == 1 => Some(Msg::CreateFormCycleType(1)),
        KeyEvent {
            code: KeyCode::Char('h'),
            modifiers: KeyModifiers::NONE,
            ..
        } if focused_field == 2 => Some(Msg::CreateFormCyclePriority(-1)),
        KeyEvent {
            code: KeyCode::Char('l'),
            modifiers: KeyModifiers::NONE,
            ..
        } if focused_field == 2 => Some(Msg::CreateFormCyclePriority(1)),
        KeyEvent {
            code: KeyCode::Backspace,
            modifiers: KeyModifiers::NONE,
            ..
        } => Some(Msg::CreateFormInput(FormFieldEdit::Backspace)),
        KeyEvent {
            code: KeyCode::Delete,
            modifiers: KeyModifiers::NONE,
            ..
        }
        | KeyEvent {
            code: KeyCode::Char('u'),
            modifiers: KeyModifiers::CONTROL,
            ..
        } => Some(Msg::CreateFormInput(FormFieldEdit::Clear)),
        KeyEvent {
            code: KeyCode::Char(ch),
            modifiers: KeyModifiers::NONE,
            ..
        } if is_printable(ch) => Some(Msg::CreateFormInput(FormFieldEdit::Insert(ch))),
        KeyEvent {
            code: KeyCode::Char(ch),
            modifiers: KeyModifiers::SHIFT,
            ..
        } if is_printable(ch) => Some(Msg::CreateFormInput(FormFieldEdit::Insert(ch))),
        _ => None,
    }
}

fn is_ctrl_char(key: &KeyEvent, ch: char) -> bool {
    key.code == KeyCode::Char(ch) && key.modifiers.contains(KeyModifiers::CONTROL)
}

fn is_printable(ch: char) -> bool {
    !ch.is_control()
}

#[cfg(test)]
mod tests {
    use super::map_key;
    use crate::config::Config;
    use crate::tui::{CreateFormState, DetailState, HelpState, Msg, PickerState, Screen, Status};
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    fn assert_bindings(screen: &Screen, bindings: &[(KeyEvent, Msg)]) {
        for (key, expected) in bindings {
            assert_eq!(map_key(screen, *key), Some(expected.clone()), "key {key:?}");
        }
    }

    fn board_screen() -> Screen {
        Screen::Board(Default::default())
    }

    fn detail_screen() -> Screen {
        Screen::IssueDetail(DetailState {
            id: "ish-abcd".to_string(),
            scroll: 0,
        })
    }

    fn picker_screen() -> Screen {
        Screen::StatusPicker(PickerState {
            issue_id: "ish-abcd".to_string(),
            options: Status::ALL.to_vec(),
            selected: 0,
        })
    }

    fn create_form_screen(focused_field: usize) -> Screen {
        create_form_screen_with_pending_cancel(focused_field, false)
    }

    fn create_form_screen_with_pending_cancel(
        focused_field: usize,
        pending_cancel: bool,
    ) -> Screen {
        let mut state = CreateFormState::new(&Config::default());
        state.focused_field = focused_field;
        state.pending_cancel = pending_cancel;
        Screen::CreateForm(state)
    }

    fn help_screen() -> Screen {
        Screen::Help(HelpState)
    }

    mod board {
        use super::*;

        #[test]
        fn maps_documented_board_bindings() {
            let screen = board_screen();
            assert_bindings(
                &screen,
                &[
                    (
                        KeyEvent::new(KeyCode::Char('h'), KeyModifiers::NONE),
                        Msg::MoveLeft,
                    ),
                    (
                        KeyEvent::new(KeyCode::Left, KeyModifiers::NONE),
                        Msg::MoveLeft,
                    ),
                    (
                        KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE),
                        Msg::MoveDown,
                    ),
                    (
                        KeyEvent::new(KeyCode::Down, KeyModifiers::NONE),
                        Msg::MoveDown,
                    ),
                    (
                        KeyEvent::new(KeyCode::Char('k'), KeyModifiers::NONE),
                        Msg::MoveUp,
                    ),
                    (KeyEvent::new(KeyCode::Up, KeyModifiers::NONE), Msg::MoveUp),
                    (
                        KeyEvent::new(KeyCode::Char('l'), KeyModifiers::NONE),
                        Msg::MoveRight,
                    ),
                    (
                        KeyEvent::new(KeyCode::Right, KeyModifiers::NONE),
                        Msg::MoveRight,
                    ),
                    (
                        KeyEvent::new(KeyCode::Char('g'), KeyModifiers::NONE),
                        Msg::JumpTop,
                    ),
                    (
                        KeyEvent::new(KeyCode::Home, KeyModifiers::NONE),
                        Msg::JumpTop,
                    ),
                    (
                        KeyEvent::new(KeyCode::Char('G'), KeyModifiers::SHIFT),
                        Msg::JumpBottom,
                    ),
                    (
                        KeyEvent::new(KeyCode::End, KeyModifiers::NONE),
                        Msg::JumpBottom,
                    ),
                    (
                        KeyEvent::new(KeyCode::Char('d'), KeyModifiers::CONTROL),
                        Msg::HalfPageDown,
                    ),
                    (
                        KeyEvent::new(KeyCode::Char('u'), KeyModifiers::CONTROL),
                        Msg::HalfPageUp,
                    ),
                    (
                        KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE),
                        Msg::OpenDetail,
                    ),
                    (
                        KeyEvent::new(KeyCode::Char(' '), KeyModifiers::NONE),
                        Msg::OpenDetail,
                    ),
                    (
                        KeyEvent::new(KeyCode::Char('c'), KeyModifiers::NONE),
                        Msg::OpenCreateForm,
                    ),
                    (
                        KeyEvent::new(KeyCode::Char('r'), KeyModifiers::NONE),
                        Msg::RequestRefresh,
                    ),
                    (
                        KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE),
                        Msg::Quit,
                    ),
                    (
                        KeyEvent::new(KeyCode::Char('?'), KeyModifiers::NONE),
                        Msg::OpenHelp,
                    ),
                    (
                        KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL),
                        Msg::Quit,
                    ),
                ],
            );
        }
    }

    mod detail {
        use super::*;

        #[test]
        fn maps_documented_detail_bindings() {
            let screen = detail_screen();
            assert_bindings(
                &screen,
                &[
                    (
                        KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE),
                        Msg::MoveDown,
                    ),
                    (
                        KeyEvent::new(KeyCode::Down, KeyModifiers::NONE),
                        Msg::MoveDown,
                    ),
                    (
                        KeyEvent::new(KeyCode::Char('k'), KeyModifiers::NONE),
                        Msg::MoveUp,
                    ),
                    (KeyEvent::new(KeyCode::Up, KeyModifiers::NONE), Msg::MoveUp),
                    (
                        KeyEvent::new(KeyCode::Char('g'), KeyModifiers::NONE),
                        Msg::JumpTop,
                    ),
                    (
                        KeyEvent::new(KeyCode::Home, KeyModifiers::NONE),
                        Msg::JumpTop,
                    ),
                    (
                        KeyEvent::new(KeyCode::Char('G'), KeyModifiers::SHIFT),
                        Msg::JumpBottom,
                    ),
                    (
                        KeyEvent::new(KeyCode::End, KeyModifiers::NONE),
                        Msg::JumpBottom,
                    ),
                    (
                        KeyEvent::new(KeyCode::Char('d'), KeyModifiers::CONTROL),
                        Msg::HalfPageDown,
                    ),
                    (
                        KeyEvent::new(KeyCode::Char('u'), KeyModifiers::CONTROL),
                        Msg::HalfPageUp,
                    ),
                    (
                        KeyEvent::new(KeyCode::Char('e'), KeyModifiers::NONE),
                        Msg::EditCurrentIssue,
                    ),
                    (
                        KeyEvent::new(KeyCode::Char('s'), KeyModifiers::NONE),
                        Msg::OpenStatusPicker,
                    ),
                    (
                        KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE),
                        Msg::PopScreen,
                    ),
                    (
                        KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE),
                        Msg::PopScreen,
                    ),
                    (
                        KeyEvent::new(KeyCode::Char('?'), KeyModifiers::NONE),
                        Msg::OpenHelp,
                    ),
                    (
                        KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL),
                        Msg::Quit,
                    ),
                ],
            );
        }
    }

    mod picker {
        use super::*;

        #[test]
        fn maps_documented_picker_bindings() {
            let screen = picker_screen();
            assert_bindings(
                &screen,
                &[
                    (
                        KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE),
                        Msg::MoveDown,
                    ),
                    (
                        KeyEvent::new(KeyCode::Down, KeyModifiers::NONE),
                        Msg::MoveDown,
                    ),
                    (
                        KeyEvent::new(KeyCode::Char('n'), KeyModifiers::CONTROL),
                        Msg::MoveDown,
                    ),
                    (
                        KeyEvent::new(KeyCode::Char('k'), KeyModifiers::NONE),
                        Msg::MoveUp,
                    ),
                    (KeyEvent::new(KeyCode::Up, KeyModifiers::NONE), Msg::MoveUp),
                    (
                        KeyEvent::new(KeyCode::Char('p'), KeyModifiers::CONTROL),
                        Msg::MoveUp,
                    ),
                    (
                        KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE),
                        Msg::SubmitStatusChange,
                    ),
                    (
                        KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE),
                        Msg::PopScreen,
                    ),
                    (
                        KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE),
                        Msg::PopScreen,
                    ),
                    (
                        KeyEvent::new(KeyCode::Char('?'), KeyModifiers::NONE),
                        Msg::OpenHelp,
                    ),
                    (
                        KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL),
                        Msg::Quit,
                    ),
                ],
            );
        }
    }

    mod create_form {
        use super::*;

        #[test]
        fn maps_navigation_and_submit_bindings() {
            assert_bindings(
                &create_form_screen(4),
                &[
                    (
                        KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE),
                        Msg::FocusNextField,
                    ),
                    (
                        KeyEvent::new(KeyCode::Char('n'), KeyModifiers::CONTROL),
                        Msg::FocusNextField,
                    ),
                    (
                        KeyEvent::new(KeyCode::BackTab, KeyModifiers::SHIFT),
                        Msg::FocusPreviousField,
                    ),
                    (
                        KeyEvent::new(KeyCode::Char('p'), KeyModifiers::CONTROL),
                        Msg::FocusPreviousField,
                    ),
                    (
                        KeyEvent::new(KeyCode::Char('s'), KeyModifiers::CONTROL),
                        Msg::SubmitCreateForm,
                    ),
                    (
                        KeyEvent::new(KeyCode::Char('e'), KeyModifiers::CONTROL),
                        Msg::SubmitCreateAndEdit,
                    ),
                    (
                        KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE),
                        Msg::SubmitCreateForm,
                    ),
                    (
                        KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE),
                        Msg::PopScreen,
                    ),
                    (
                        KeyEvent::new(KeyCode::Char('?'), KeyModifiers::NONE),
                        Msg::OpenHelp,
                    ),
                    (
                        KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL),
                        Msg::Quit,
                    ),
                ],
            );
        }

        #[test]
        fn maps_type_and_priority_cycling_bindings() {
            assert_bindings(
                &create_form_screen(1),
                &[
                    (
                        KeyEvent::new(KeyCode::Left, KeyModifiers::NONE),
                        Msg::CreateFormCycleType(-1),
                    ),
                    (
                        KeyEvent::new(KeyCode::Right, KeyModifiers::NONE),
                        Msg::CreateFormCycleType(1),
                    ),
                    (
                        KeyEvent::new(KeyCode::Char('h'), KeyModifiers::NONE),
                        Msg::CreateFormCycleType(-1),
                    ),
                    (
                        KeyEvent::new(KeyCode::Char('l'), KeyModifiers::NONE),
                        Msg::CreateFormCycleType(1),
                    ),
                ],
            );

            assert_bindings(
                &create_form_screen(2),
                &[
                    (
                        KeyEvent::new(KeyCode::Left, KeyModifiers::NONE),
                        Msg::CreateFormCyclePriority(-1),
                    ),
                    (
                        KeyEvent::new(KeyCode::Right, KeyModifiers::NONE),
                        Msg::CreateFormCyclePriority(1),
                    ),
                    (
                        KeyEvent::new(KeyCode::Char('h'), KeyModifiers::NONE),
                        Msg::CreateFormCyclePriority(-1),
                    ),
                    (
                        KeyEvent::new(KeyCode::Char('l'), KeyModifiers::NONE),
                        Msg::CreateFormCyclePriority(1),
                    ),
                ],
            );
        }

        #[test]
        fn maps_text_edit_bindings() {
            assert_bindings(
                &create_form_screen(0),
                &[
                    (
                        KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE),
                        Msg::CreateFormInput(crate::tui::FormFieldEdit::Backspace),
                    ),
                    (
                        KeyEvent::new(KeyCode::Delete, KeyModifiers::NONE),
                        Msg::CreateFormInput(crate::tui::FormFieldEdit::Clear),
                    ),
                    (
                        KeyEvent::new(KeyCode::Char('u'), KeyModifiers::CONTROL),
                        Msg::CreateFormInput(crate::tui::FormFieldEdit::Clear),
                    ),
                    (
                        KeyEvent::new(KeyCode::Char('x'), KeyModifiers::NONE),
                        Msg::CreateFormInput(crate::tui::FormFieldEdit::Insert('x')),
                    ),
                    (
                        KeyEvent::new(KeyCode::Char('X'), KeyModifiers::SHIFT),
                        Msg::CreateFormInput(crate::tui::FormFieldEdit::Insert('X')),
                    ),
                ],
            );

            assert_eq!(
                map_key(
                    &create_form_screen(0),
                    KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE)
                ),
                None
            );
        }

        #[test]
        fn maps_discard_confirmation_bindings_when_modal_is_open() {
            let screen = create_form_screen_with_pending_cancel(0, true);
            assert_bindings(
                &screen,
                &[
                    (
                        KeyEvent::new(KeyCode::Char('y'), KeyModifiers::NONE),
                        Msg::ConfirmDiscardCreateForm,
                    ),
                    (
                        KeyEvent::new(KeyCode::Char('Y'), KeyModifiers::SHIFT),
                        Msg::ConfirmDiscardCreateForm,
                    ),
                    (
                        KeyEvent::new(KeyCode::Char('n'), KeyModifiers::NONE),
                        Msg::CancelDiscardCreateForm,
                    ),
                    (
                        KeyEvent::new(KeyCode::Char('N'), KeyModifiers::SHIFT),
                        Msg::CancelDiscardCreateForm,
                    ),
                    (
                        KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE),
                        Msg::CancelDiscardCreateForm,
                    ),
                    (
                        KeyEvent::new(KeyCode::Char('?'), KeyModifiers::NONE),
                        Msg::OpenHelp,
                    ),
                    (
                        KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL),
                        Msg::Quit,
                    ),
                ],
            );

            assert_eq!(
                map_key(
                    &screen,
                    KeyEvent::new(KeyCode::Char('x'), KeyModifiers::NONE)
                ),
                None
            );
            assert_eq!(
                map_key(&screen, KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE)),
                None
            );
        }
    }

    mod help {
        use super::*;

        #[test]
        fn maps_any_key_to_pop_screen_except_global_quit() {
            let screen = help_screen();
            assert_bindings(
                &screen,
                &[
                    (
                        KeyEvent::new(KeyCode::Char('x'), KeyModifiers::NONE),
                        Msg::PopScreen,
                    ),
                    (
                        KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE),
                        Msg::PopScreen,
                    ),
                    (
                        KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE),
                        Msg::PopScreen,
                    ),
                    (
                        KeyEvent::new(KeyCode::Char('?'), KeyModifiers::NONE),
                        Msg::PopScreen,
                    ),
                    (
                        KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL),
                        Msg::Quit,
                    ),
                ],
            );
        }
    }

    mod leaks {
        use super::*;

        #[test]
        fn create_form_only_bindings_do_not_leak_to_board_detail_or_picker() {
            let unique_create_form_bindings = [
                KeyEvent::new(KeyCode::Char('s'), KeyModifiers::CONTROL),
                KeyEvent::new(KeyCode::Char('e'), KeyModifiers::CONTROL),
                KeyEvent::new(KeyCode::Delete, KeyModifiers::NONE),
            ];

            let other_screens = [board_screen(), detail_screen(), picker_screen()];

            for screen in &other_screens {
                for key in unique_create_form_bindings {
                    assert_eq!(map_key(screen, key), None, "screen {screen:?} key {key:?}");
                }
            }
        }
    }
}

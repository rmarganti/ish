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
        Screen::CreateForm(state) => map_create_form_key(state.focused_field, key),
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

fn map_create_form_key(focused_field: usize, key: KeyEvent) -> Option<Msg> {
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

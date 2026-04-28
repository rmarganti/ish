#![allow(dead_code)]

use crate::tui::{CreateFormState, FormFieldEdit, InputState, KeyPattern, Msg, Screen};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

#[derive(Debug, Clone, PartialEq)]
pub enum KeyResolution {
    Dispatch(Msg),
    Pending(InputState),
    Ignore,
}

#[derive(Debug, Clone, PartialEq)]
struct Binding {
    sequence: Vec<KeyPattern>,
    msg: Msg,
}

pub fn resolve_key(screen: &Screen, input: &InputState, key: KeyEvent) -> KeyResolution {
    if matches!(screen, Screen::Help(_)) {
        return if is_ctrl_char(&key, 'c') {
            KeyResolution::Dispatch(Msg::Quit)
        } else {
            KeyResolution::Dispatch(Msg::PopScreen)
        };
    }

    let key = KeyPattern::from(key);
    let bindings = bindings_for(screen);
    resolve_bindings(&bindings, input, key)
}

fn resolve_bindings(bindings: &[Binding], input: &InputState, key: KeyPattern) -> KeyResolution {
    let mut candidate = input.pending_keys.clone();
    candidate.push(key);

    let exact = bindings
        .iter()
        .find(|binding| binding.sequence == candidate);
    let has_longer_prefix = bindings.iter().any(|binding| {
        binding.sequence.starts_with(&candidate) && binding.sequence.len() > candidate.len()
    });

    if let Some(exact) = exact
        && !has_longer_prefix
    {
        return KeyResolution::Dispatch(exact.msg.clone());
    }

    if has_longer_prefix {
        return KeyResolution::Pending(InputState {
            pending_keys: candidate,
        });
    }

    if !input.pending_keys.is_empty() {
        return resolve_bindings(bindings, &InputState::default(), key);
    }

    KeyResolution::Ignore
}

fn bindings_for(screen: &Screen) -> Vec<Binding> {
    let mut bindings = global_bindings();
    bindings.extend(match screen {
        Screen::Board(_) => board_bindings(),
        Screen::IssueDetail(_) => detail_bindings(),
        Screen::StatusPicker(_) => picker_bindings(Msg::SubmitStatusChange),
        Screen::PriorityPicker(_) => picker_bindings(Msg::SubmitPriorityChange),
        Screen::CreateForm(state) => create_form_bindings(state),
        Screen::Help(_) => Vec::new(),
    });
    bindings
}

fn global_bindings() -> Vec<Binding> {
    vec![
        binding(seq(&[ctrl('c')]), Msg::Quit),
        binding(seq(&[plain('?')]), Msg::OpenHelp),
    ]
}

fn board_bindings() -> Vec<Binding> {
    vec![
        binding(seq(&[plain('h')]), Msg::MoveLeft),
        binding(seq(&[special(KeyCode::Left)]), Msg::MoveLeft),
        binding(seq(&[plain('j')]), Msg::MoveDown),
        binding(seq(&[special(KeyCode::Down)]), Msg::MoveDown),
        binding(seq(&[plain('k')]), Msg::MoveUp),
        binding(seq(&[special(KeyCode::Up)]), Msg::MoveUp),
        binding(seq(&[plain('l')]), Msg::MoveRight),
        binding(seq(&[special(KeyCode::Right)]), Msg::MoveRight),
        binding(seq(&[plain('g'), plain('g')]), Msg::JumpTop),
        binding(seq(&[plain('g'), plain('p')]), Msg::GoToParent),
        binding(seq(&[special(KeyCode::Home)]), Msg::JumpTop),
        binding(seq(&[shift('G')]), Msg::JumpBottom),
        binding(seq(&[special(KeyCode::End)]), Msg::JumpBottom),
        binding(seq(&[ctrl('d')]), Msg::HalfPageDown),
        binding(seq(&[ctrl('u')]), Msg::HalfPageUp),
        binding(seq(&[special(KeyCode::Enter)]), Msg::OpenDetail),
        binding(seq(&[plain(' ')]), Msg::OpenDetail),
        binding(seq(&[plain('c')]), Msg::OpenCreateForm),
        binding(seq(&[plain('r')]), Msg::RequestRefresh),
        binding(seq(&[plain('q')]), Msg::Quit),
    ]
}

fn detail_bindings() -> Vec<Binding> {
    vec![
        binding(seq(&[plain('j')]), Msg::MoveDown),
        binding(seq(&[special(KeyCode::Down)]), Msg::MoveDown),
        binding(seq(&[plain('k')]), Msg::MoveUp),
        binding(seq(&[special(KeyCode::Up)]), Msg::MoveUp),
        binding(seq(&[plain('g'), plain('g')]), Msg::JumpTop),
        binding(seq(&[plain('g'), plain('p')]), Msg::GoToParent),
        binding(seq(&[special(KeyCode::Home)]), Msg::JumpTop),
        binding(seq(&[shift('G')]), Msg::JumpBottom),
        binding(seq(&[special(KeyCode::End)]), Msg::JumpBottom),
        binding(seq(&[ctrl('d')]), Msg::HalfPageDown),
        binding(seq(&[ctrl('u')]), Msg::HalfPageUp),
        binding(seq(&[plain('e')]), Msg::EditCurrentIssue),
        binding(seq(&[plain('s')]), Msg::OpenStatusPicker),
        binding(seq(&[plain('p')]), Msg::OpenPriorityPicker),
        binding(seq(&[plain('q')]), Msg::PopScreen),
        binding(seq(&[special(KeyCode::Esc)]), Msg::PopScreen),
    ]
}

fn picker_bindings(submit_msg: Msg) -> Vec<Binding> {
    vec![
        binding(seq(&[plain('j')]), Msg::MoveDown),
        binding(seq(&[special(KeyCode::Down)]), Msg::MoveDown),
        binding(seq(&[ctrl('n')]), Msg::MoveDown),
        binding(seq(&[plain('k')]), Msg::MoveUp),
        binding(seq(&[special(KeyCode::Up)]), Msg::MoveUp),
        binding(seq(&[ctrl('p')]), Msg::MoveUp),
        binding(seq(&[special(KeyCode::Enter)]), submit_msg),
        binding(seq(&[plain('q')]), Msg::PopScreen),
        binding(seq(&[special(KeyCode::Esc)]), Msg::PopScreen),
    ]
}

fn create_form_bindings(state: &CreateFormState) -> Vec<Binding> {
    if state.pending_cancel {
        return vec![
            binding(seq(&[plain('y')]), Msg::ConfirmDiscardCreateForm),
            binding(seq(&[shift('Y')]), Msg::ConfirmDiscardCreateForm),
            binding(seq(&[plain('n')]), Msg::CancelDiscardCreateForm),
            binding(seq(&[shift('N')]), Msg::CancelDiscardCreateForm),
            binding(seq(&[special(KeyCode::Esc)]), Msg::CancelDiscardCreateForm),
        ];
    }

    let mut bindings = vec![
        binding(seq(&[special(KeyCode::Tab)]), Msg::FocusNextField),
        binding(seq(&[ctrl('n')]), Msg::FocusNextField),
        binding(
            seq(&[special_mod(KeyCode::BackTab, KeyModifiers::SHIFT)]),
            Msg::FocusPreviousField,
        ),
        binding(seq(&[ctrl('p')]), Msg::FocusPreviousField),
        binding(seq(&[ctrl('s')]), Msg::SubmitCreateForm),
        binding(seq(&[ctrl('e')]), Msg::SubmitCreateAndEdit),
        binding(seq(&[special(KeyCode::Esc)]), Msg::PopScreen),
        binding(
            seq(&[special(KeyCode::Backspace)]),
            Msg::CreateFormInput(FormFieldEdit::Backspace),
        ),
        binding(
            seq(&[special(KeyCode::Delete)]),
            Msg::CreateFormInput(FormFieldEdit::Clear),
        ),
        binding(
            seq(&[ctrl('u')]),
            Msg::CreateFormInput(FormFieldEdit::Clear),
        ),
    ];

    if state.focused_field == 4 {
        bindings.push(binding(
            seq(&[special(KeyCode::Enter)]),
            Msg::SubmitCreateForm,
        ));
    }

    if state.focused_field == 1 {
        bindings.extend([
            binding(seq(&[special(KeyCode::Left)]), Msg::CreateFormCycleType(-1)),
            binding(seq(&[special(KeyCode::Right)]), Msg::CreateFormCycleType(1)),
            binding(seq(&[plain('h')]), Msg::CreateFormCycleType(-1)),
            binding(seq(&[plain('l')]), Msg::CreateFormCycleType(1)),
        ]);
    }

    if state.focused_field == 2 {
        bindings.extend([
            binding(
                seq(&[special(KeyCode::Left)]),
                Msg::CreateFormCyclePriority(-1),
            ),
            binding(
                seq(&[special(KeyCode::Right)]),
                Msg::CreateFormCyclePriority(1),
            ),
            binding(seq(&[plain('h')]), Msg::CreateFormCyclePriority(-1)),
            binding(seq(&[plain('l')]), Msg::CreateFormCyclePriority(1)),
        ]);
    }

    if matches!(state.focused_field, 0 | 3) {
        bindings.extend(printable_bindings());
    }

    bindings
}

fn printable_bindings() -> Vec<Binding> {
    let mut bindings = Vec::new();

    for byte in 0u8..=127 {
        let ch = byte as char;
        if !is_printable(ch) {
            continue;
        }

        bindings.push(binding(
            seq(&[plain(ch)]),
            Msg::CreateFormInput(FormFieldEdit::Insert(ch)),
        ));

        if ch.is_ascii_alphabetic() {
            bindings.push(binding(
                seq(&[shift(ch.to_ascii_uppercase())]),
                Msg::CreateFormInput(FormFieldEdit::Insert(ch.to_ascii_uppercase())),
            ));
        }
    }

    bindings
}

fn binding(sequence: Vec<KeyPattern>, msg: Msg) -> Binding {
    Binding { sequence, msg }
}

fn seq(keys: &[KeyPattern]) -> Vec<KeyPattern> {
    keys.to_vec()
}

fn plain(ch: char) -> KeyPattern {
    KeyPattern::new(KeyCode::Char(ch), KeyModifiers::NONE)
}

fn shift(ch: char) -> KeyPattern {
    KeyPattern::new(KeyCode::Char(ch), KeyModifiers::SHIFT)
}

fn ctrl(ch: char) -> KeyPattern {
    KeyPattern::new(KeyCode::Char(ch), KeyModifiers::CONTROL)
}

fn special(code: KeyCode) -> KeyPattern {
    KeyPattern::new(code, KeyModifiers::NONE)
}

fn special_mod(code: KeyCode, modifiers: KeyModifiers) -> KeyPattern {
    KeyPattern::new(code, modifiers)
}

fn is_ctrl_char(key: &KeyEvent, ch: char) -> bool {
    key.code == KeyCode::Char(ch) && key.modifiers.contains(KeyModifiers::CONTROL)
}

fn is_printable(ch: char) -> bool {
    !ch.is_control()
}

#[cfg(test)]
mod tests {
    use super::{KeyResolution, resolve_key};
    use crate::config::Config;
    use crate::tui::{
        CreateFormState, DetailState, HelpState, InputState, Msg, PickerState, Priority,
        PriorityPickerState, Screen, Status,
    };
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

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

    fn priority_picker_screen() -> Screen {
        Screen::PriorityPicker(PriorityPickerState {
            issue_id: "ish-abcd".to_string(),
            options: Priority::ALL.to_vec(),
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

    fn key(code: KeyCode, modifiers: KeyModifiers) -> KeyEvent {
        KeyEvent::new(code, modifiers)
    }

    fn assert_single_key(screen: &Screen, key: KeyEvent, expected: Msg) {
        assert_eq!(
            resolve_key(screen, &InputState::default(), key),
            KeyResolution::Dispatch(expected)
        );
    }

    #[test]
    fn board_and_detail_g_prefix_resolves_gg_and_gp() {
        for screen in [board_screen(), detail_screen()] {
            let pending = resolve_key(
                &screen,
                &InputState::default(),
                key(KeyCode::Char('g'), KeyModifiers::NONE),
            );
            let KeyResolution::Pending(input) = pending else {
                panic!("expected pending g prefix, got {pending:?}");
            };
            assert_eq!(input.pending_keys.len(), 1);

            assert_eq!(
                resolve_key(&screen, &input, key(KeyCode::Char('g'), KeyModifiers::NONE)),
                KeyResolution::Dispatch(Msg::JumpTop)
            );
            assert_eq!(
                resolve_key(&screen, &input, key(KeyCode::Char('p'), KeyModifiers::NONE)),
                KeyResolution::Dispatch(Msg::GoToParent)
            );

            assert_single_key(
                &screen,
                key(KeyCode::Home, KeyModifiers::NONE),
                Msg::JumpTop,
            );
            assert_single_key(
                &screen,
                key(KeyCode::Char('G'), KeyModifiers::SHIFT),
                Msg::JumpBottom,
            );
        }
    }

    #[test]
    fn invalid_g_continuation_clears_pending_and_retries_current_key() {
        let screen = board_screen();
        let KeyResolution::Pending(input) = resolve_key(
            &screen,
            &InputState::default(),
            key(KeyCode::Char('g'), KeyModifiers::NONE),
        ) else {
            panic!("expected pending g prefix");
        };

        assert_eq!(
            resolve_key(&screen, &input, key(KeyCode::Char('h'), KeyModifiers::NONE)),
            KeyResolution::Dispatch(Msg::MoveLeft)
        );
        assert_eq!(
            resolve_key(&screen, &input, key(KeyCode::Char('x'), KeyModifiers::NONE)),
            KeyResolution::Ignore
        );
        assert_eq!(
            resolve_key(&screen, &input, key(KeyCode::Char('?'), KeyModifiers::NONE)),
            KeyResolution::Dispatch(Msg::OpenHelp)
        );
    }

    #[test]
    fn single_key_bindings_still_work_across_screens() {
        assert_single_key(
            &board_screen(),
            key(KeyCode::Char('j'), KeyModifiers::NONE),
            Msg::MoveDown,
        );
        assert_single_key(
            &board_screen(),
            key(KeyCode::Enter, KeyModifiers::NONE),
            Msg::OpenDetail,
        );
        assert_single_key(
            &detail_screen(),
            key(KeyCode::Char('e'), KeyModifiers::NONE),
            Msg::EditCurrentIssue,
        );
        assert_single_key(
            &detail_screen(),
            key(KeyCode::Char('p'), KeyModifiers::NONE),
            Msg::OpenPriorityPicker,
        );
        assert_single_key(
            &picker_screen(),
            key(KeyCode::Char('n'), KeyModifiers::CONTROL),
            Msg::MoveDown,
        );
        assert_single_key(
            &priority_picker_screen(),
            key(KeyCode::Enter, KeyModifiers::NONE),
            Msg::SubmitPriorityChange,
        );
    }

    #[test]
    fn create_form_bindings_remain_field_sensitive() {
        assert_single_key(
            &create_form_screen(4),
            key(KeyCode::Enter, KeyModifiers::NONE),
            Msg::SubmitCreateForm,
        );
        assert_single_key(
            &create_form_screen(1),
            key(KeyCode::Char('h'), KeyModifiers::NONE),
            Msg::CreateFormCycleType(-1),
        );
        assert_single_key(
            &create_form_screen(2),
            key(KeyCode::Right, KeyModifiers::NONE),
            Msg::CreateFormCyclePriority(1),
        );
        assert_single_key(
            &create_form_screen(0),
            key(KeyCode::Char('x'), KeyModifiers::NONE),
            Msg::CreateFormInput(crate::tui::FormFieldEdit::Insert('x')),
        );
        assert_eq!(
            resolve_key(
                &create_form_screen(0),
                &InputState::default(),
                key(KeyCode::Enter, KeyModifiers::NONE)
            ),
            KeyResolution::Ignore
        );
    }

    #[test]
    fn create_form_discard_modal_only_accepts_confirmation_keys_plus_globals() {
        let screen = create_form_screen_with_pending_cancel(0, true);
        assert_single_key(
            &screen,
            key(KeyCode::Char('y'), KeyModifiers::NONE),
            Msg::ConfirmDiscardCreateForm,
        );
        assert_single_key(
            &screen,
            key(KeyCode::Char('n'), KeyModifiers::NONE),
            Msg::CancelDiscardCreateForm,
        );
        assert_single_key(
            &screen,
            key(KeyCode::Esc, KeyModifiers::NONE),
            Msg::CancelDiscardCreateForm,
        );
        assert_single_key(
            &screen,
            key(KeyCode::Char('?'), KeyModifiers::NONE),
            Msg::OpenHelp,
        );
        assert_eq!(
            resolve_key(
                &screen,
                &InputState::default(),
                key(KeyCode::Tab, KeyModifiers::NONE)
            ),
            KeyResolution::Ignore
        );
    }

    #[test]
    fn help_screen_consumes_any_key_except_global_quit() {
        let screen = help_screen();
        assert_single_key(
            &screen,
            key(KeyCode::Char('x'), KeyModifiers::NONE),
            Msg::PopScreen,
        );
        assert_single_key(
            &screen,
            key(KeyCode::Char('?'), KeyModifiers::NONE),
            Msg::PopScreen,
        );
        assert_single_key(
            &screen,
            key(KeyCode::Char('c'), KeyModifiers::CONTROL),
            Msg::Quit,
        );
    }
}

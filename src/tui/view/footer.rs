#![allow(dead_code)]

use crate::tui::{Model, Screen, theme};
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::prelude::{Line, Span};
use ratatui::widgets::Paragraph;

pub fn draw(frame: &mut Frame<'_>, area: Rect, model: &Model) {
    if area.height == 0 || area.width == 0 {
        return;
    }

    frame.render_widget(Paragraph::new(footer_line(model)), area);
}

fn footer_line(model: &Model) -> Line<'static> {
    match model.screens.last() {
        Some(Screen::Board(_)) => hints(&[
            ("h/j/k/l", "move"),
            ("enter", "open"),
            ("c", "create"),
            ("r", "refresh"),
            ("?", "help"),
            ("q", "quit"),
        ]),
        Some(Screen::IssueDetail(_)) => hints(&[
            ("j/k", "scroll"),
            ("e", "edit"),
            ("s", "status"),
            ("?", "help"),
            ("q", "back"),
        ]),
        Some(Screen::StatusPicker(_)) | Some(Screen::PriorityPicker(_)) => hints(&[
            ("j/k", "move"),
            ("enter", "choose"),
            ("esc", "cancel"),
            ("?", "help"),
        ]),
        Some(Screen::CreateForm(state)) if state.pending_cancel => hints(&[
            ("y", "discard"),
            ("n", "keep editing"),
            ("esc", "keep editing"),
        ]),
        Some(Screen::CreateForm(_)) => hints(&[
            ("tab", "next field"),
            ("S-tab", "prev field"),
            ("C-s", "save"),
            ("C-e", "save + edit"),
            ("esc", "cancel"),
            ("?", "help"),
        ]),
        Some(Screen::Help(_)) => hints(&[("any key", "close")]),
        None => hints(&[("q", "quit")]),
    }
}

fn hints(items: &[(&str, &str)]) -> Line<'static> {
    let mut spans = Vec::new();

    for (index, (key, description)) in items.iter().enumerate() {
        if index > 0 {
            spans.push(Span::raw("  "));
        }
        spans.push(Span::styled((*key).to_string(), theme::footer_key()));
        spans.push(Span::raw(" "));
        spans.push(Span::styled(
            (*description).to_string(),
            theme::footer_desc(),
        ));
    }

    Line::from(spans)
}

#[cfg(test)]
mod tests {
    use super::footer_line;
    use crate::test_support::tui::model_with_board;
    use crate::tui::{CreateFormState, HelpState, Screen};

    #[test]
    fn footer_hints_follow_active_screen() {
        let mut model = model_with_board(vec![]);
        assert!(footer_line(&model).to_string().contains("open"));

        model.screens.push(Screen::Help(HelpState));
        assert_eq!(footer_line(&model).to_string(), "any key close");
    }

    #[test]
    fn discard_confirmation_footer_prefers_y_n_actions() {
        let mut model = model_with_board(vec![]);
        let mut state = CreateFormState::new(&model.config);
        state.pending_cancel = true;
        model.screens.push(Screen::CreateForm(state));

        assert_eq!(
            footer_line(&model).to_string(),
            "y discard  n keep editing  esc keep editing"
        );
    }
}

#![allow(dead_code)]

use crate::tui::view::{picker_modal, priority_label};
use crate::tui::{Model, PriorityPickerState, theme};
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::prelude::{Line, Span};
use ratatui::style::{Modifier, Style};

pub fn draw(frame: &mut Frame<'_>, area: Rect, model: &Model, state: &PriorityPickerState) {
    picker_modal::draw(frame, area, "Set priority", option_lines(model, state));
}

fn option_lines(model: &Model, state: &PriorityPickerState) -> Vec<Line<'static>> {
    state
        .options
        .iter()
        .enumerate()
        .map(|(index, priority)| {
            let selected = index == state.selected;
            Line::from(vec![
                Span::styled(
                    if selected { "› " } else { "  " },
                    if selected {
                        Style::default()
                            .fg(ratatui::style::Color::Cyan)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().add_modifier(Modifier::DIM)
                    },
                ),
                Span::styled(
                    priority_label(*priority).to_string(),
                    theme::priority_style(&model.config, *priority),
                ),
            ])
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::option_lines;
    use crate::test_support::tui::{IshBuilder, model_with_board};
    use crate::tui::{DetailState, Priority, PriorityPickerState, Screen, view};
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;

    #[test]
    fn option_lines_render_all_priorities_and_highlight_selected_row() {
        let model = model_with_board(vec![]);
        let state = PriorityPickerState {
            issue_id: "ish-detail".to_string(),
            options: Priority::ALL.to_vec(),
            selected: 1,
        };

        let rendered = option_lines(&model, &state)
            .iter()
            .map(|line| line.to_string())
            .collect::<Vec<_>>();

        assert_eq!(rendered[0], "  Critical");
        assert_eq!(rendered[1], "› High");
        assert_eq!(rendered[4], "  Deferred");
    }

    #[test]
    fn priority_picker_renders_on_top_of_detail_screen_without_panicking() {
        let issue = IshBuilder::new("detail")
            .title("Detail view")
            .status("todo")
            .priority(Priority::High)
            .body("Body")
            .build();
        let mut model = model_with_board(vec![issue]);
        model.screens = vec![
            Screen::IssueDetail(DetailState {
                id: "ish-detail".to_string(),
                scroll: 0,
            }),
            Screen::PriorityPicker(PriorityPickerState {
                issue_id: "ish-detail".to_string(),
                options: Priority::ALL.to_vec(),
                selected: 3,
            }),
        ];

        let backend = TestBackend::new(100, 30);
        let mut terminal = Terminal::new(backend).expect("test terminal should initialize");

        terminal
            .draw(|frame| view::draw(frame, &model))
            .expect("priority picker should render");
    }
}

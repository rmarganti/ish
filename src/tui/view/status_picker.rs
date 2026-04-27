#![allow(dead_code)]

use crate::tui::view::status_label;
use crate::tui::{Model, PickerState, theme};
use ratatui::Frame;
use ratatui::layout::{Constraint, Flex, Layout, Rect};
use ratatui::prelude::{Line, Span};
use ratatui::style::{Modifier, Style};
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Wrap};

const MODAL_WIDTH: u16 = 40;
const MODAL_HEIGHT: u16 = 8;

pub fn draw(frame: &mut Frame<'_>, area: Rect, model: &Model, state: &PickerState) {
    let modal_area = centered_rect(
        MODAL_WIDTH.min(area.width.saturating_sub(2)).max(24),
        MODAL_HEIGHT.min(area.height.saturating_sub(2)).max(6),
        area,
    );

    frame.render_widget(Clear, modal_area);
    frame.render_widget(
        Paragraph::new(option_lines(model, state))
            .block(Block::default().borders(Borders::ALL).title(" Set status "))
            .wrap(Wrap { trim: false }),
        modal_area,
    );
}

fn option_lines(model: &Model, state: &PickerState) -> Vec<Line<'static>> {
    state
        .options
        .iter()
        .enumerate()
        .map(|(index, status)| {
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
                    status_label(*status).to_string(),
                    theme::status_style(&model.config, *status),
                ),
            ])
        })
        .collect()
}

fn centered_rect(width: u16, height: u16, area: Rect) -> Rect {
    let [vertical] = Layout::vertical([Constraint::Length(height)])
        .flex(Flex::Center)
        .areas(area);
    let [horizontal] = Layout::horizontal([Constraint::Length(width)])
        .flex(Flex::Center)
        .areas(vertical);
    horizontal
}

#[cfg(test)]
mod tests {
    use super::option_lines;
    use crate::test_support::tui::{IshBuilder, model_with_board};
    use crate::tui::{DetailState, PickerState, Screen, Status, view};
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;

    #[test]
    fn option_lines_render_all_statuses_and_highlight_selected_row() {
        let model = model_with_board(vec![]);
        let state = PickerState {
            issue_id: "ish-detail".to_string(),
            options: Status::ALL.to_vec(),
            selected: 2,
        };

        let rendered = option_lines(&model, &state)
            .iter()
            .map(|line| line.to_string())
            .collect::<Vec<_>>();

        assert_eq!(rendered[0], "  Draft");
        assert_eq!(rendered[2], "› In Progress");
        assert_eq!(rendered[4], "  Scrapped");
    }

    #[test]
    fn status_picker_renders_on_top_of_detail_screen_without_panicking() {
        let issue = IshBuilder::new("detail")
            .title("Detail view")
            .status("todo")
            .body("Body")
            .build();
        let mut model = model_with_board(vec![issue]);
        model.screens = vec![
            Screen::IssueDetail(DetailState {
                id: "ish-detail".to_string(),
                scroll: 0,
            }),
            Screen::StatusPicker(PickerState {
                issue_id: "ish-detail".to_string(),
                options: Status::ALL.to_vec(),
                selected: 1,
            }),
        ];

        let backend = TestBackend::new(100, 30);
        let mut terminal = Terminal::new(backend).expect("test terminal should initialize");

        terminal
            .draw(|frame| view::draw(frame, &model))
            .expect("status picker should render");
    }
}

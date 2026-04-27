#![allow(dead_code)]

use crate::tui::{CreateFormState, Model, theme};
use ratatui::Frame;
use ratatui::layout::{Constraint, Flex, Layout, Rect};
use ratatui::prelude::{Line, Span};
use ratatui::style::{Modifier, Style};
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Wrap};

const PANEL_WIDTH: u16 = 72;
const PANEL_HEIGHT: u16 = 12;
const CANCEL_MODAL_WIDTH: u16 = 34;
const CANCEL_MODAL_HEIGHT: u16 = 5;

pub fn draw(frame: &mut Frame<'_>, area: Rect, model: &Model, state: &CreateFormState) {
    let panel_area = centered_rect(
        PANEL_WIDTH.min(area.width.saturating_sub(2)).max(24),
        PANEL_HEIGHT.min(area.height.saturating_sub(2)).max(8),
        area,
    );

    frame.render_widget(Clear, panel_area);
    frame.render_widget(
        Paragraph::new(form_lines(model, state))
            .block(Block::default().borders(Borders::ALL).title(" New issue "))
            .wrap(Wrap { trim: false }),
        panel_area,
    );

    if state.pending_cancel {
        draw_pending_cancel_modal(frame, panel_area);
    }
}

fn form_lines(model: &Model, state: &CreateFormState) -> Vec<Line<'static>> {
    vec![
        form_row(
            "Title",
            Line::from(field_text(&state.title, "(required)")),
            state.focused_field == 0,
        ),
        form_row(
            "Type",
            cycle_line(
                state.ish_type.as_str(),
                theme::type_style(&model.config, state.ish_type),
            ),
            state.focused_field == 1,
        ),
        form_row(
            "Priority",
            cycle_line(
                state.priority.as_str(),
                theme::priority_style(&model.config, state.priority),
            ),
            state.focused_field == 2,
        ),
        form_row(
            "Tags",
            Line::from(field_text(&state.tags, "comma,separated,tags")),
            state.focused_field == 3,
        ),
        Line::default(),
        form_row(
            "Save",
            Line::from(vec![
                Span::styled("[", Style::default().add_modifier(Modifier::DIM)),
                Span::styled(
                    " Create issue ",
                    Style::default().add_modifier(Modifier::BOLD),
                ),
                Span::styled("]", Style::default().add_modifier(Modifier::DIM)),
            ]),
            state.focused_field == 4,
        ),
    ]
}

fn form_row(label: &str, value: Line<'static>, focused: bool) -> Line<'static> {
    let label_style = if focused {
        Style::default()
            .fg(ratatui::style::Color::Cyan)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default()
            .fg(ratatui::style::Color::DarkGray)
            .add_modifier(Modifier::BOLD)
    };

    let mut spans = vec![Span::styled(format!("{label:<10} "), label_style)];
    spans.extend(value.spans);

    Line::from(spans)
}

fn field_text(value: &str, placeholder: &str) -> Vec<Span<'static>> {
    if value.is_empty() {
        vec![Span::styled(
            placeholder.to_string(),
            Style::default().add_modifier(Modifier::DIM),
        )]
    } else {
        vec![Span::raw(value.to_string())]
    }
}

fn cycle_line(value: &str, value_style: Style) -> Line<'static> {
    Line::from(vec![
        Span::styled("< ", Style::default().add_modifier(Modifier::DIM)),
        Span::styled(value.to_string(), value_style),
        Span::styled(" >", Style::default().add_modifier(Modifier::DIM)),
    ])
}

fn draw_pending_cancel_modal(frame: &mut Frame<'_>, area: Rect) {
    let modal_area = centered_rect(
        CANCEL_MODAL_WIDTH.min(area.width.saturating_sub(2)).max(20),
        CANCEL_MODAL_HEIGHT
            .min(area.height.saturating_sub(2))
            .max(5),
        area,
    );

    frame.render_widget(Clear, modal_area);
    frame.render_widget(
        Paragraph::new(vec![
            Line::from("Discard new issue?"),
            Line::from(vec![
                Span::styled("y", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" discard  "),
                Span::styled("n", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" keep editing"),
            ]),
        ])
        .block(Block::default().borders(Borders::ALL).title(" Confirm "))
        .wrap(Wrap { trim: true }),
        modal_area,
    );
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
    use super::{cycle_line, draw_pending_cancel_modal, form_lines};
    use crate::test_support::tui::model_with_board;
    use crate::tui::{CreateFormState, IshType, Priority, Screen, view};
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;
    use ratatui::style::Modifier;

    #[test]
    fn form_lines_show_placeholders_and_cycle_widgets() {
        let model = model_with_board(vec![]);
        let state = CreateFormState::new(&model.config);

        let rendered = form_lines(&model, &state)
            .iter()
            .map(|line| line.to_string())
            .collect::<Vec<_>>();

        assert!(rendered[0].contains("Title"));
        assert!(rendered[0].contains("(required)"));
        assert!(rendered[1].contains("< task >"));
        assert!(rendered[2].contains("< normal >"));
        assert!(rendered[5].contains("Create issue"));
    }

    #[test]
    fn cycle_lines_keep_value_in_the_middle() {
        let line = cycle_line(
            IshType::Feature.as_str(),
            crate::tui::theme::type_style(&crate::config::Config::default(), IshType::Feature),
        );

        assert_eq!(line.to_string(), "< feature >");
        assert!(line.spans[1].style.add_modifier.contains(Modifier::BOLD));
    }

    #[test]
    fn discard_modal_shows_literal_y_n_actions() {
        let backend = TestBackend::new(40, 10);
        let mut terminal = Terminal::new(backend).expect("test terminal should initialize");

        terminal
            .draw(|frame| draw_pending_cancel_modal(frame, frame.area()))
            .expect("discard modal should render");

        let rendered = terminal
            .backend()
            .buffer()
            .content()
            .iter()
            .map(|cell| cell.symbol())
            .collect::<String>();

        assert!(rendered.contains("Discard new issue?"));
        assert!(rendered.contains("y discard"));
        assert!(rendered.contains("n keep editing"));
    }

    #[test]
    fn create_form_screen_renders_registered_view_without_panicking() {
        let mut model = model_with_board(vec![]);
        let mut state = CreateFormState::new(&model.config);
        state.title = "Ship create form".to_string();
        state.ish_type = IshType::Bug;
        state.priority = Priority::High;
        state.tags = "tui,kanban".to_string();
        state.pending_cancel = true;
        model.screens = vec![Screen::CreateForm(state)];

        let backend = TestBackend::new(100, 30);
        let mut terminal = Terminal::new(backend).expect("test terminal should initialize");

        terminal
            .draw(|frame| view::draw(frame, &model))
            .expect("create form should render");
    }
}

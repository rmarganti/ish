#![allow(dead_code)]

use crate::tui::{BoardState, IshType, Model, Priority, Screen, Status, theme};
use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::prelude::{Line, Span};
use ratatui::style::{Modifier, Style};
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Wrap};

mod board;
mod create_form;
mod footer;
mod help;
mod issue_detail;
mod status_line;
mod status_picker;

pub fn draw(frame: &mut Frame<'_>, model: &Model) {
    let area = frame.area();

    if model.term_too_small {
        draw_terminal_too_small(frame, area);
        return;
    }

    let status_height = if model.status_line.is_some() { 1 } else { 0 };
    let sections = Layout::vertical([
        Constraint::Min(1),
        Constraint::Length(status_height),
        Constraint::Length(1),
    ])
    .split(area);

    draw_main(frame, sections[0], model);

    if status_height > 0 {
        status_line::draw(frame, sections[1], model);
    }

    footer::draw(frame, sections[2], model);
}

fn draw_main(frame: &mut Frame<'_>, area: Rect, model: &Model) {
    match model.screens.as_slice() {
        [
            ..,
            Screen::IssueDetail(detail),
            Screen::StatusPicker(picker),
        ] => {
            issue_detail::draw(frame, area, model, detail);
            status_picker::draw(frame, area, model, picker);
        }
        [.., screen] => draw_screen(frame, area, model, screen),
        [] => board::draw(frame, area, model, &BoardState::default()),
    }
}

fn draw_screen(frame: &mut Frame<'_>, area: Rect, model: &Model, screen: &Screen) {
    match screen {
        Screen::Board(state) => board::draw(frame, area, model, state),
        Screen::IssueDetail(state) => issue_detail::draw(frame, area, model, state),
        Screen::StatusPicker(state) => status_picker::draw(frame, area, model, state),
        Screen::PriorityPicker(_) => draw_placeholder(frame, area),
        Screen::CreateForm(state) => create_form::draw(frame, area, model, state),
        Screen::Help(_) => help::draw(frame, area),
    }
}

fn draw_placeholder(frame: &mut Frame<'_>, area: Rect) {
    let block = Block::default().borders(Borders::ALL).title("ish tui");
    let inner = block.inner(area);
    frame.render_widget(Clear, area);
    frame.render_widget(block, area);
    frame.render_widget(
        Paragraph::new("This TUI screen is not implemented yet.")
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true }),
        inner,
    );
}

fn draw_terminal_too_small(frame: &mut Frame<'_>, area: Rect) {
    frame.render_widget(Clear, area);
    frame.render_widget(
        Paragraph::new("Terminal too small (minimum 80×20)")
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true }),
        area,
    );
}

fn status_label(status: Status) -> &'static str {
    match status {
        Status::Draft => "Draft",
        Status::Todo => "Todo",
        Status::InProgress => "In Progress",
        Status::Completed => "Completed",
        Status::Scrapped => "Scrapped",
    }
}

fn status_from_ish(ish_status: &str) -> Option<Status> {
    Status::from_str(ish_status)
}

fn priority_from_ish(priority: Option<&str>) -> Priority {
    priority
        .and_then(Priority::from_str)
        .unwrap_or(Priority::Normal)
}

fn type_from_ish(ish_type: &str) -> IshType {
    IshType::from_str(ish_type).unwrap_or(IshType::Task)
}

fn truncate_with_ellipsis(text: &str, width: usize) -> String {
    if width == 0 {
        return String::new();
    }

    let chars = text.chars().collect::<Vec<_>>();
    if chars.len() <= width {
        return text.to_string();
    }

    if width == 1 {
        return "…".to_string();
    }

    chars[..width - 1].iter().collect::<String>() + "…"
}

fn card_title_line(title: &str, width: u16, selected: bool) -> Line<'static> {
    let style = if selected {
        Style::default().add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    };

    Line::from(Span::styled(
        truncate_with_ellipsis(title, width as usize),
        style,
    ))
}

fn card_meta_line(
    model: &Model,
    id: &str,
    priority: Priority,
    ish_type: IshType,
    tag: Option<&str>,
) -> Line<'static> {
    let mut spans = vec![
        Span::raw(id.to_string()),
        Span::raw("  ! "),
        Span::styled(
            priority.as_str().to_string(),
            theme::priority_style(&model.config, priority),
        ),
        Span::raw("  ⊙ "),
        Span::styled(
            ish_type.as_str().to_string(),
            theme::type_style(&model.config, ish_type),
        ),
    ];

    if let Some(tag) = tag.filter(|tag| !tag.is_empty()) {
        spans.push(Span::raw("  #"));
        spans.push(Span::styled(
            tag.to_string(),
            Style::default().fg(ratatui::style::Color::Cyan),
        ));
    }

    Line::from(spans)
}

#[cfg(test)]
mod tests {
    use super::{draw, status_label, truncate_with_ellipsis};
    use crate::test_support::tui::model_with_board;
    use crate::tui::{Severity, Status, StatusLine};
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;

    #[test]
    fn truncates_card_titles_with_ellipsis() {
        assert_eq!(truncate_with_ellipsis("abcdef", 4), "abc…");
        assert_eq!(truncate_with_ellipsis("abc", 4), "abc");
        assert_eq!(truncate_with_ellipsis("abc", 1), "…");
    }

    #[test]
    fn formats_status_labels_for_column_headers() {
        assert_eq!(status_label(Status::Draft), "Draft");
        assert_eq!(status_label(Status::InProgress), "In Progress");
    }

    #[test]
    fn terminal_too_small_message_replaces_normal_chrome() {
        let backend = TestBackend::new(40, 10);
        let mut terminal = Terminal::new(backend).expect("test terminal should initialize");
        let mut model = model_with_board(vec![]);
        model.term_too_small = true;
        model.status_line = Some(StatusLine {
            text: "saved".to_string(),
            severity: Severity::Success,
        });

        terminal
            .draw(|frame| draw(frame, &model))
            .expect("drawing should succeed");

        let buffer = terminal.backend().buffer().clone();
        let rendered = buffer
            .content()
            .iter()
            .map(|cell| cell.symbol())
            .collect::<String>();

        assert!(rendered.contains("Terminal too small (minimum 80×20)"));
        assert!(!rendered.contains("saved"));
        assert!(!rendered.contains("? help"));
    }
}

#![allow(dead_code)]

use crate::tui::{BoardState, IshType, Model, Priority, Screen, Status, theme};
use ratatui::Frame;
use ratatui::layout::{Alignment, Rect};
use ratatui::prelude::{Line, Span};
use ratatui::style::{Modifier, Style};
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Wrap};

mod board;
mod issue_detail;

pub fn draw(frame: &mut Frame<'_>, model: &Model) {
    let area = frame.area();

    match model.screens.last() {
        Some(Screen::Board(state)) => board::draw(frame, area, model, state),
        Some(Screen::IssueDetail(state)) => issue_detail::draw(frame, area, model, state),
        Some(_) => draw_placeholder(frame, area),
        None => board::draw(frame, area, model, &BoardState::default()),
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
    use super::{status_label, truncate_with_ellipsis};
    use crate::tui::Status;

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
}

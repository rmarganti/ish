use ratatui::Frame;
use ratatui::layout::{Constraint, Flex, Layout, Rect};
use ratatui::prelude::Line;
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Wrap};

const MODAL_WIDTH: u16 = 40;
const MODAL_HEIGHT: u16 = 8;

pub fn draw(frame: &mut Frame<'_>, area: Rect, title: &str, lines: Vec<Line<'static>>) {
    let modal_area = centered_rect(
        MODAL_WIDTH.min(area.width.saturating_sub(2)).max(24),
        MODAL_HEIGHT.min(area.height.saturating_sub(2)).max(6),
        area,
    );

    frame.render_widget(Clear, modal_area);
    frame.render_widget(
        Paragraph::new(lines)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(format!(" {title} ")),
            )
            .wrap(Wrap { trim: false }),
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

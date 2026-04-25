#![allow(dead_code)]

use crate::tui::{Model, theme};
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::prelude::{Line, Span};
use ratatui::widgets::Paragraph;

pub fn draw(frame: &mut Frame<'_>, area: Rect, model: &Model) {
    if area.height == 0 || area.width == 0 {
        return;
    }

    let line = model
        .status_line
        .as_ref()
        .map(|status| {
            Line::from(Span::styled(
                status.text.clone(),
                theme::severity_style(status.severity),
            ))
        })
        .unwrap_or_default();

    frame.render_widget(Paragraph::new(line), area);
}

#[cfg(test)]
mod tests {
    use super::draw;
    use crate::test_support::tui::model_with_board;
    use crate::tui::{Severity, StatusLine};
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;

    #[test]
    fn status_line_renders_when_present() {
        let mut model = model_with_board(vec![]);
        model.status_line = Some(StatusLine {
            text: "Refreshed".to_string(),
            severity: Severity::Success,
        });

        let backend = TestBackend::new(40, 2);
        let mut terminal = Terminal::new(backend).expect("test terminal should initialize");
        terminal
            .draw(|frame| draw(frame, frame.area(), &model))
            .expect("status line should render");
    }
}

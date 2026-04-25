#![allow(dead_code)]

use crate::tui::theme;
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::prelude::{Line, Span};
use ratatui::style::{Modifier, Style};
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Wrap};

pub fn draw(frame: &mut Frame<'_>, area: Rect) {
    if area.height == 0 || area.width == 0 {
        return;
    }

    frame.render_widget(Clear, area);
    frame.render_widget(
        Paragraph::new(help_lines())
            .block(Block::default().borders(Borders::ALL).title(" Help "))
            .wrap(Wrap { trim: false }),
        area,
    );
}

fn help_lines() -> Vec<Line<'static>> {
    vec![
        section_title("Global"),
        binding_line("Ctrl-c", "quit immediately"),
        binding_line("?", "toggle help"),
        Line::default(),
        section_title("Board"),
        binding_line("h/j/k/l, arrows", "move between columns and cards"),
        binding_line("g / G", "jump to top / bottom"),
        binding_line("Ctrl-u / Ctrl-d", "half-page scroll"),
        binding_line("Enter / Space", "open selected issue"),
        binding_line("c", "create issue"),
        binding_line("r", "refresh from disk"),
        binding_line("q", "quit"),
        Line::default(),
        section_title("Issue detail"),
        binding_line("j / k, arrows", "scroll body"),
        binding_line("g / G", "jump to top / bottom"),
        binding_line("Ctrl-u / Ctrl-d", "half-page scroll"),
        binding_line("e", "edit in $EDITOR"),
        binding_line("s", "open status picker"),
        binding_line("q / Esc", "back to board"),
        Line::default(),
        section_title("Status picker"),
        binding_line("j / k, arrows", "move selection"),
        binding_line("Ctrl-n / Ctrl-p", "move selection"),
        binding_line("Enter", "save selected status"),
        binding_line("q / Esc", "cancel"),
        Line::default(),
        section_title("Create form"),
        binding_line("Tab / Shift-Tab", "next / previous field"),
        binding_line("Ctrl-n / Ctrl-p", "next / previous field"),
        binding_line("h / l, ← / →", "cycle type / priority"),
        binding_line("Ctrl-s", "create issue"),
        binding_line("Ctrl-e", "create issue and edit"),
        binding_line("Esc", "cancel or confirm discard"),
        Line::default(),
        Line::from(vec![
            Span::styled("Tip:", theme::footer_key()),
            Span::raw(" press any key to close this overlay."),
        ]),
    ]
}

fn section_title(title: &str) -> Line<'static> {
    Line::from(Span::styled(
        title.to_string(),
        Style::default().add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
    ))
}

fn binding_line(key: &str, description: &str) -> Line<'static> {
    Line::from(vec![
        Span::styled(format!("{key:<18}"), theme::footer_key()),
        Span::styled(description.to_string(), theme::footer_desc()),
    ])
}

#[cfg(test)]
mod tests {
    use super::help_lines;

    #[test]
    fn help_overlay_lists_all_sections() {
        let rendered = help_lines()
            .into_iter()
            .map(|line| line.to_string())
            .collect::<Vec<_>>()
            .join("\n");

        assert!(rendered.contains("Global"));
        assert!(rendered.contains("Board"));
        assert!(rendered.contains("Issue detail"));
        assert!(rendered.contains("Status picker"));
        assert!(rendered.contains("Create form"));
        assert!(rendered.contains("press any key to close"));
    }
}

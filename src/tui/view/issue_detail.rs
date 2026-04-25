#![allow(dead_code)]

use crate::model::ish::Ish;
use crate::tui::view::{priority_from_ish, status_from_ish, type_from_ish};
use crate::tui::{DetailState, Model, theme};
use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::prelude::{Line, Span};
use ratatui::style::{Modifier, Style};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

const FOOTER_HEIGHT: u16 = 3;

pub fn draw(frame: &mut Frame<'_>, area: Rect, model: &Model, state: &DetailState) {
    let Some(issue) = model.issues.iter().find(|issue| issue.id == state.id) else {
        draw_missing_issue(frame, area, &state.id);
        return;
    };

    let metadata_lines = metadata_lines(model, issue);
    let metadata_height = metadata_lines.len().saturating_add(2) as u16;
    let sections = Layout::vertical([
        Constraint::Length(metadata_height),
        Constraint::Min(1),
        Constraint::Length(FOOTER_HEIGHT),
    ])
    .split(area);

    frame.render_widget(
        Paragraph::new(metadata_lines)
            .block(Block::default().borders(Borders::ALL).title(" Details "))
            .wrap(Wrap { trim: false }),
        sections[0],
    );

    frame.render_widget(
        Paragraph::new(body_lines(issue))
            .block(Block::default().borders(Borders::ALL).title(" Body "))
            .scroll((state.scroll, 0))
            .wrap(Wrap { trim: false }),
        sections[1],
    );

    frame.render_widget(
        Paragraph::new(detail_footer())
            .block(Block::default().borders(Borders::ALL).title(" Keys "))
            .wrap(Wrap { trim: false }),
        sections[2],
    );
}

fn draw_missing_issue(frame: &mut Frame<'_>, area: Rect, id: &str) {
    frame.render_widget(
        Paragraph::new(format!(
            "Issue {id} is no longer available. Press q to go back."
        ))
        .block(Block::default().borders(Borders::ALL).title(" Details "))
        .wrap(Wrap { trim: true }),
        area,
    );
}

fn metadata_lines(model: &Model, issue: &Ish) -> Vec<Line<'static>> {
    let status = status_from_ish(&issue.status);
    let priority = priority_from_ish(issue.priority.as_deref());
    let ish_type = type_from_ish(&issue.ish_type);

    vec![
        Line::from(Span::styled(
            issue.title.clone(),
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Line::from(vec![
            meta_label("id"),
            Span::raw(issue.id.clone()),
            Span::raw("  "),
            meta_label("type"),
            Span::styled(
                ish_type.as_str().to_string(),
                theme::type_style(&model.config, ish_type),
            ),
            Span::raw("  "),
            meta_label("status"),
            Span::styled(
                issue.status.clone(),
                status
                    .map(|status| theme::status_style(&model.config, status))
                    .unwrap_or_default(),
            ),
            Span::raw("  "),
            meta_label("priority"),
            Span::styled(
                priority.as_str().to_string(),
                theme::priority_style(&model.config, priority),
            ),
        ]),
        Line::from(vec![
            meta_label("tags"),
            Span::raw(join_or_dash(&issue.tags)),
        ]),
        Line::from(vec![
            meta_label("parent"),
            Span::raw(issue.parent.as_deref().unwrap_or("—").to_string()),
        ]),
        Line::from(vec![
            meta_label("blocking"),
            Span::raw(join_or_dash(&issue.blocking)),
        ]),
        Line::from(vec![
            meta_label("blocked_by"),
            Span::raw(join_or_dash(&issue.blocked_by)),
        ]),
        Line::from(vec![
            meta_label("updated"),
            Span::raw(issue.updated_at.to_rfc3339()),
        ]),
    ]
}

fn body_lines(issue: &Ish) -> Vec<Line<'static>> {
    if issue.body.is_empty() {
        return vec![Line::from(Span::styled(
            "(empty body)".to_string(),
            Style::default().add_modifier(Modifier::DIM),
        ))];
    }

    issue
        .body
        .lines()
        .map(|line| {
            if let Some(stripped) = line.strip_prefix("#") {
                let level = line.chars().take_while(|ch| *ch == '#').count();
                let text = stripped.trim_start_matches('#').trim().to_string();
                return Line::from(Span::styled(
                    if text.is_empty() {
                        line.to_string()
                    } else {
                        text
                    },
                    Style::default()
                        .add_modifier(Modifier::BOLD)
                        .add_modifier(if level == 1 {
                            Modifier::UNDERLINED
                        } else {
                            Modifier::empty()
                        }),
                ));
            }

            if line.starts_with("```") {
                return Line::from(Span::styled(
                    line.to_string(),
                    Style::default().add_modifier(Modifier::DIM),
                ));
            }

            Line::from(line.to_string())
        })
        .collect()
}

fn detail_footer() -> Line<'static> {
    Line::from(vec![
        Span::styled("e", theme::footer_key()),
        Span::styled(" edit  ", theme::footer_desc()),
        Span::styled("s", theme::footer_key()),
        Span::styled(" status  ", theme::footer_desc()),
        Span::styled("q", theme::footer_key()),
        Span::styled(" back", theme::footer_desc()),
    ])
}

fn meta_label(label: &str) -> Span<'static> {
    Span::styled(
        format!("{label}: "),
        Style::default()
            .fg(ratatui::style::Color::DarkGray)
            .add_modifier(Modifier::BOLD),
    )
}

fn join_or_dash(values: &[String]) -> String {
    if values.is_empty() {
        "—".to_string()
    } else {
        values.join(", ")
    }
}

#[cfg(test)]
mod tests {
    use super::{body_lines, join_or_dash, metadata_lines};
    use crate::test_support::tui::{IshBuilder, model_with_board};
    use crate::tui::{DetailState, Screen, view};
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;

    #[test]
    fn metadata_lines_include_relationships_and_tags() {
        let issue = IshBuilder::new("detail")
            .title("Detail view")
            .status("in-progress")
            .tags(&["tui", "kanban"])
            .parent("ish-parent")
            .blocking(&["ish-blocking"])
            .blocked_by(&["ish-blocker"])
            .build();
        let model = model_with_board(vec![issue]);
        let issue = &model.issues[0];

        let lines = metadata_lines(&model, issue);
        let rendered = lines
            .iter()
            .map(|line| line.to_string())
            .collect::<Vec<_>>()
            .join("\n");

        assert!(rendered.contains("Detail view"));
        assert!(rendered.contains("tags: tui, kanban"));
        assert!(rendered.contains("parent: ish-parent"));
        assert!(rendered.contains("blocking: ish-blocking"));
        assert!(rendered.contains("blocked_by: ish-blocker"));
    }

    #[test]
    fn body_lines_handle_empty_and_basic_markdown_styling() {
        let empty = IshBuilder::new("empty").build();
        let filled = IshBuilder::new("filled")
            .body("# Heading\n\nparagraph\n```rust")
            .build();

        assert_eq!(body_lines(&empty)[0].to_string(), "(empty body)");
        let body = body_lines(&filled)
            .iter()
            .map(|line| line.to_string())
            .collect::<Vec<_>>();
        assert_eq!(body[0], "Heading");
        assert_eq!(body[2], "paragraph");
        assert_eq!(body[3], "```rust");
        assert_eq!(join_or_dash(&[]), "—");
    }

    #[test]
    fn detail_screen_renders_registered_view_without_panicking() {
        let issue = IshBuilder::new("detail")
            .title("Detail view")
            .status("in-progress")
            .body("# Heading\n\nBody")
            .build();
        let mut model = model_with_board(vec![issue]);
        model.screens = vec![Screen::IssueDetail(DetailState {
            id: "ish-detail".to_string(),
            scroll: 1,
        })];

        let backend = TestBackend::new(100, 30);
        let mut terminal = Terminal::new(backend).expect("test terminal should initialize");

        terminal
            .draw(|frame| view::draw(frame, &model))
            .expect("detail screen should render");
    }
}

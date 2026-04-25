#![allow(dead_code)]

use crate::config::Config;
use crate::output::color_name_to_ratatui;
use crate::tui::{IshType, Priority, Severity, Status};
use ratatui::style::{Color, Modifier, Style};

fn themed_color(color_name: &str) -> Color {
    color_name_to_ratatui(color_name).unwrap_or(Color::Reset)
}

fn color_from_status(config: &Config, status: Status) -> Color {
    config
        .get_status(status.as_str())
        .map(|status| themed_color(status.color))
        .unwrap_or(Color::Reset)
}

fn color_from_type(config: &Config, ish_type: IshType) -> Color {
    config
        .get_type(ish_type.as_str())
        .map(|ish_type| themed_color(ish_type.color))
        .unwrap_or(Color::Reset)
}

fn color_from_priority(config: &Config, priority: Priority) -> Color {
    config
        .get_priority(priority.as_str())
        .map(|priority| themed_color(priority.color))
        .unwrap_or(Color::Reset)
}

pub fn status_color(config: &Config, status: Status) -> Color {
    color_from_status(config, status)
}

pub fn type_color(config: &Config, ish_type: IshType) -> Color {
    color_from_type(config, ish_type)
}

pub fn priority_color(config: &Config, priority: Priority) -> Color {
    color_from_priority(config, priority)
}

pub fn severity_style(severity: Severity) -> Style {
    match severity {
        Severity::Info => Style::default()
            .fg(Color::DarkGray)
            .add_modifier(Modifier::DIM),
        Severity::Success => Style::default()
            .fg(Color::Green)
            .add_modifier(Modifier::BOLD),
        Severity::Error => Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
    }
}

pub fn status_style(config: &Config, status: Status) -> Style {
    let mut style = Style::default()
        .fg(status_color(config, status))
        .add_modifier(Modifier::BOLD);

    if matches!(status, Status::Completed | Status::Scrapped) {
        style = style.add_modifier(Modifier::DIM);
    }

    style
}

pub fn type_style(config: &Config, ish_type: IshType) -> Style {
    Style::default()
        .fg(type_color(config, ish_type))
        .add_modifier(Modifier::BOLD)
}

pub fn priority_style(config: &Config, priority: Priority) -> Style {
    Style::default()
        .fg(priority_color(config, priority))
        .add_modifier(Modifier::BOLD)
}

pub fn card_border(selected: bool, focused_column: bool) -> Style {
    match (selected, focused_column) {
        (true, true) => Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
        (true, false) => Style::default().fg(Color::Cyan),
        (false, true) => Style::default().fg(Color::DarkGray),
        (false, false) => Style::default()
            .fg(Color::DarkGray)
            .add_modifier(Modifier::DIM),
    }
}

pub fn column_header(active: bool) -> Style {
    if active {
        Style::default()
            .fg(Color::White)
            .bg(Color::DarkGray)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default()
            .fg(Color::DarkGray)
            .add_modifier(Modifier::BOLD)
    }
}

pub fn footer_key() -> Style {
    Style::default()
        .fg(Color::Cyan)
        .add_modifier(Modifier::BOLD)
}

pub fn footer_desc() -> Style {
    Style::default().fg(Color::Gray)
}

#[cfg(test)]
mod tests {
    use super::{
        card_border, column_header, footer_desc, footer_key, priority_color, priority_style,
        severity_style, status_color, status_style, type_color, type_style,
    };
    use crate::config::Config;
    use crate::output::color_name_to_ratatui;
    use crate::tui::{IshType, Priority, Severity, Status};
    use ratatui::style::{Color, Modifier};

    #[test]
    fn kanban_column_colors_match_cli_palette() {
        let config = Config::default();

        for status in [
            Status::Draft,
            Status::Todo,
            Status::InProgress,
            Status::Completed,
        ] {
            let expected = color_name_to_ratatui(
                config
                    .get_status(status.as_str())
                    .expect("status should exist")
                    .color,
            )
            .expect("status color should be supported");
            let actual = status_color(&config, status);

            assert_eq!(actual, expected);
            assert_ne!(actual, Color::Reset);
        }
    }

    #[test]
    fn type_and_priority_colors_match_cli_palette() {
        let config = Config::default();

        assert_eq!(
            type_color(&config, IshType::Epic),
            color_name_to_ratatui(config.get_type("epic").expect("type should exist").color)
                .expect("type color should be supported")
        );
        assert_eq!(
            priority_color(&config, Priority::High),
            color_name_to_ratatui(
                config
                    .get_priority("high")
                    .expect("priority should exist")
                    .color
            )
            .expect("priority color should be supported")
        );
    }

    #[test]
    fn status_and_metadata_styles_apply_expected_modifiers() {
        let config = Config::default();

        assert!(
            status_style(&config, Status::Todo)
                .add_modifier
                .contains(Modifier::BOLD)
        );
        assert!(
            status_style(&config, Status::Completed)
                .add_modifier
                .contains(Modifier::DIM)
        );
        assert!(
            type_style(&config, IshType::Task)
                .add_modifier
                .contains(Modifier::BOLD)
        );
        assert!(
            priority_style(&config, Priority::Critical)
                .add_modifier
                .contains(Modifier::BOLD)
        );
    }

    #[test]
    fn severity_and_shared_widget_styles_are_stable() {
        assert_eq!(severity_style(Severity::Info).fg, Some(Color::DarkGray));
        assert_eq!(severity_style(Severity::Success).fg, Some(Color::Green));
        assert_eq!(severity_style(Severity::Error).fg, Some(Color::Red));

        assert!(
            card_border(true, true)
                .add_modifier
                .contains(Modifier::BOLD)
        );
        assert_eq!(column_header(true).bg, Some(Color::DarkGray));
        assert_eq!(footer_key().fg, Some(Color::Cyan));
        assert_eq!(footer_desc().fg, Some(Color::Gray));
    }
}

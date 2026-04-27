#![allow(dead_code)]

use crate::tui::view::{
    card_meta_line, card_title_line, priority_from_ish, status_from_ish, status_label,
    type_from_ish,
};
use crate::tui::{BOARD_COLUMNS, BoardState, Model, theme};
use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::prelude::{Alignment, Line};
use ratatui::style::{Modifier, Style};
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Wrap};

const CARD_HEIGHT: u16 = 4;
const MIN_HEADER_HEIGHT: u16 = 3;

pub fn draw(frame: &mut Frame<'_>, area: Rect, model: &Model, state: &BoardState) {
    let columns =
        Layout::horizontal(vec![Constraint::Ratio(1, 4); BOARD_COLUMNS.len()]).split(area);

    for (index, (status, column_area)) in BOARD_COLUMNS.iter().zip(columns.iter()).enumerate() {
        draw_column(frame, *column_area, model, state, index, *status);
    }
}

fn draw_column(
    frame: &mut Frame<'_>,
    area: Rect,
    model: &Model,
    state: &BoardState,
    column_index: usize,
    status: crate::tui::Status,
) {
    let active = state.selected_column == column_index;
    let bucket = model.bucket_for_status(status);

    let outer = Block::default()
        .borders(Borders::ALL)
        .border_style(theme::card_border(false, active))
        .title(Line::styled(
            format!(" {} ({}) ", status_label(status), bucket.len()),
            theme::column_header(active),
        ));

    let inner = outer.inner(area);
    frame.render_widget(outer, area);

    if inner.height == 0 || inner.width == 0 {
        return;
    }

    if bucket.is_empty() {
        let placeholder = Paragraph::new("(empty)")
            .alignment(Alignment::Center)
            .style(Style::default().add_modifier(Modifier::DIM));
        frame.render_widget(placeholder, inner);
        return;
    }

    let start = state.column_offsets[column_index].min(bucket.len().saturating_sub(1));
    let visible_count = ((inner.height.max(MIN_HEADER_HEIGHT)) / CARD_HEIGHT).max(1) as usize;
    let end = (start + visible_count).min(bucket.len());

    let card_areas =
        Layout::vertical(vec![Constraint::Length(CARD_HEIGHT); end - start]).split(inner);

    for (visible_index, ish) in bucket[start..end].iter().enumerate() {
        let absolute_index = start + visible_index;
        let selected = state.column_cursors[column_index] == Some(absolute_index);
        draw_card(
            frame,
            card_areas[visible_index],
            model,
            ish,
            selected,
            active,
        );
    }
}

fn draw_card(
    frame: &mut Frame<'_>,
    area: Rect,
    model: &Model,
    ish: &crate::model::ish::Ish,
    selected: bool,
    focused_column: bool,
) {
    if area.height == 0 || area.width == 0 {
        return;
    }

    let visually_selected = selected && focused_column;
    let border_style = theme::card_border(visually_selected, focused_column);
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(border_style)
        .style(if visually_selected {
            Style::default().add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        });
    let inner = block.inner(area);
    frame.render_widget(Clear, area);
    frame.render_widget(block, area);

    if inner.height == 0 || inner.width == 0 {
        return;
    }

    let priority = priority_from_ish(ish.priority.as_deref());
    let ish_type = type_from_ish(&ish.ish_type);
    let first_tag = ish.tags.first().map(String::as_str);

    let title_status = status_from_ish(&ish.status)
        .map(|status| theme::status_style(&model.config, status))
        .unwrap_or_default();

    let title =
        card_title_line(&ish.title, inner.width, visually_selected).patch_style(title_status);
    let meta = card_meta_line(model, &ish.id, priority, ish_type, first_tag);
    let lines = vec![title, meta];

    frame.render_widget(Paragraph::new(lines).wrap(Wrap { trim: false }), inner);
}

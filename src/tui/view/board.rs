#![allow(dead_code)]

use crate::tui::model::BoardRow;
use crate::tui::view::{
    card_meta_line, card_title_line, priority_from_ish, status_from_ish, status_label,
    type_from_ish,
};
use crate::tui::{BOARD_COLUMNS, BoardState, Model, theme};
use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::prelude::{Alignment, Line};
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Wrap};

const CARD_HEIGHT: u16 = 4;
const MIN_HEADER_HEIGHT: u16 = 3;
/// Width of one level of tree indentation in the board column gutter.
/// Horizontal space in board columns is precious, so each level only takes a
/// single cell. The connector glyph itself is rendered as a one-cell `└` /
/// `├` to fit.
const INDENT_PER_LEVEL: u16 = 1;
/// Minimum card content width we will preserve before falling back to drawing
/// a child card without an indent gutter.
const MIN_CARD_WIDTH: u16 = 12;

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

    for (visible_index, row) in bucket[start..end].iter().enumerate() {
        let absolute_index = start + visible_index;
        let selected = state.column_cursors[column_index] == Some(absolute_index);
        draw_card(
            frame,
            card_areas[visible_index],
            model,
            row,
            selected,
            active,
        );
    }
}

fn draw_card(
    frame: &mut Frame<'_>,
    area: Rect,
    model: &Model,
    row: &BoardRow<'_>,
    selected: bool,
    focused_column: bool,
) {
    if area.height == 0 || area.width == 0 {
        return;
    }

    let depth = row.depth() as u16;
    let (card_area, gutter_area) = split_indent(area, depth);

    if let Some(gutter) = gutter_area {
        draw_tree_gutter(frame, gutter, &row.ancestors_have_more, row.is_last);
    }

    let area = card_area;
    let ish = row.ish;

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

/// Reserve a left gutter for parent–child indentation. Returns the (card,
/// gutter) areas. Falls back to no gutter when the column is too narrow to
/// preserve a usable card width.
fn split_indent(area: Rect, depth: u16) -> (Rect, Option<Rect>) {
    if depth == 0 {
        return (area, None);
    }
    let gutter_width = depth.saturating_mul(INDENT_PER_LEVEL);
    if gutter_width == 0 || area.width <= gutter_width.saturating_add(MIN_CARD_WIDTH) {
        return (area, None);
    }
    let parts = Layout::horizontal([
        Constraint::Length(gutter_width),
        Constraint::Min(MIN_CARD_WIDTH),
    ])
    .split(area);
    (parts[1], Some(parts[0]))
}

/// Draw the dimmed tree connectors and ancestor lines into the gutter.
fn draw_tree_gutter(
    frame: &mut Frame<'_>,
    area: Rect,
    ancestors_have_more: &[bool],
    is_last: bool,
) {
    if area.width == 0 || area.height == 0 {
        return;
    }
    let depth = ancestors_have_more.len();
    if depth == 0 {
        return;
    }

    let style = Style::default()
        .fg(Color::DarkGray)
        .add_modifier(Modifier::DIM);
    let buf = frame.buffer_mut();
    let right_edge = area.x + area.width;

    // Vertical bars for every ancestor whose chain is still continuing. The
    // last entry of `ancestors_have_more` corresponds to the current node's
    // immediate parent — that one is encoded by `is_last` and rendered as the
    // connector glyph below, so we skip it here (matches CLI semantics).
    for (i, &has_more) in ancestors_have_more
        .iter()
        .take(depth.saturating_sub(1))
        .enumerate()
    {
        if !has_more {
            continue;
        }
        let x = area.x + (i as u16) * INDENT_PER_LEVEL;
        if x >= right_edge {
            break;
        }
        for dy in 0..area.height {
            buf.set_string(x, area.y + dy, "│", style);
        }
    }

    // Current node connector at column (depth - 1) * INDENT_PER_LEVEL.
    let connector_x = area.x + ((depth - 1) as u16).saturating_mul(INDENT_PER_LEVEL);
    if connector_x >= right_edge {
        return;
    }

    // Cards are 4 rows tall: 0 = top border, 1 = title, 2 = meta, 3 = bottom
    // border. Place the connector glyph on the title row to align with the
    // ish title.
    let title_row = 1u16.min(area.height.saturating_sub(1));
    for dy in 0..area.height {
        let row_y = area.y + dy;
        if dy == title_row {
            let glyph = if is_last { "└" } else { "├" };
            buf.set_string(connector_x, row_y, glyph, style);
        } else if dy < title_row || !is_last {
            // Above the title we always extend a vertical bar upward; below
            // the title we only continue when this row is not the last among
            // its siblings (so the bar reaches the next sibling card).
            buf.set_string(connector_x, row_y, "│", style);
        }
    }
}

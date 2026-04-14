use crate::app::App;
use crate::theme::{Theme, ThemeEditColumn, ThemeField};
use crate::typing::CharState;

use super::common::{base_style, line_from_typing};

use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Modifier};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Cell, Paragraph, Row, Table, Wrap};
use ratatui::Frame;
use strum::IntoEnumIterator;

pub(super) fn draw_config(frame: &mut Frame, theme: &Theme, app: &App) {
    let area = frame.area();
    let block = Block::default()
        .borders(Borders::ALL)
        .title("Config")
        .style(base_style(theme));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(5),
            Constraint::Length(9),
            Constraint::Length(2),
        ])
        .split(inner);

    let fields: Vec<ThemeField> = ThemeField::iter().collect();
    let highlight_style = base_style(theme).add_modifier(Modifier::REVERSED);
    let normal_style = base_style(theme);

    let rows: Vec<Row> = fields
        .iter()
        .enumerate()
        .map(|(i, &field)| {
            let row_sel = app.theme_edit_row == i;
            let field_st = if row_sel {
                base_style(theme).add_modifier(Modifier::BOLD)
            } else {
                normal_style
            };
            let pal_st = if row_sel && app.theme_edit_col == ThemeEditColumn::Palette {
                highlight_style
            } else {
                normal_style
            };
            let shade_st = if row_sel && app.theme_edit_col == ThemeEditColumn::Shade {
                highlight_style
            } else {
                normal_style
            };
            Row::new(vec![
                Cell::from(field.label()).style(field_st),
                Cell::from(theme.palette_label(field)).style(pal_st),
                Cell::from(theme.shade_label(field)).style(shade_st),
            ])
        })
        .collect();

    let widths = [
        Constraint::Percentage(52),
        Constraint::Percentage(24),
        Constraint::Percentage(24),
    ];

    let table = Table::new(rows, widths)
        .header(
            Row::new(vec![
                Cell::from("Theme Field"),
                Cell::from("Palette"),
                Cell::from("Shade"),
            ])
            .style(base_style(theme).add_modifier(Modifier::BOLD))
            .bottom_margin(1),
        )
        .column_spacing(1)
        .style(base_style(theme));

    frame.render_widget(table, chunks[0]);

    let preview_line1 = line_from_typing(
        theme,
        "The quick brown fox".chars().enumerate(),
        |i| {
            let state = match i {
                0..=15 => CharState::Correct,
                16 => CharState::Current,
                _ => CharState::Untyped,
            };
            (state, false)
        },
        Some(8),
    );
    let preview_line2 = line_from_typing(
        theme,
        "jumped over the lazy dog".chars().enumerate(),
        |i| match i {
            0..=1 => (CharState::Correct, false),
            2 => (CharState::Incorrect, false),
            3 => (CharState::Current, true),
            _ => (CharState::Untyped, false),
        },
        Some(0),
    );

    let preview_block = Block::default()
        .borders(Borders::ALL)
        .title("Preview")
        .style(base_style(theme));
    let preview_inner = preview_block.inner(chunks[1]);
    frame.render_widget(preview_block, chunks[1]);
    frame.render_widget(
        Paragraph::new(vec![preview_line1, preview_line2])
            .wrap(Wrap { trim: false })
            .style(base_style(theme)),
        preview_inner,
    );

    let hints = vec![
        Line::from(""),
        Line::from(Span::styled(
            "Use arrow keys to navigate  Tab/ShiftTab: cycle palette or shade   R: reset   Q: save & back",
            base_style(theme).fg(Color::DarkGray),
        )),
    ];
    frame.render_widget(
        Paragraph::new(hints)
            .wrap(Wrap { trim: false })
            .style(base_style(theme)),
        chunks[2],
    );
}

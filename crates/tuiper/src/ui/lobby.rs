use crate::app::{App, RaceMode};
use crate::theme::{ThemeField, ThemePaint};

use super::common::{base_style, default_block, default_paragraph};

use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Modifier};
use ratatui::text::{Line, Span};
use ratatui::Frame;

pub(super) fn draw_lobby(frame: &mut Frame, paint: &ThemePaint<'_>, app: &App) {
    let area = frame.area();
    let block = default_block("Tuiper", paint);
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(3), Constraint::Length(2)])
        .split(inner);

    let len = app.lobby_value();
    let mode_line = Line::from(Span::styled(
        match app.mode {
            RaceMode::Time => "Mode: Time",
            RaceMode::Words => "Mode: Words",
        },
        base_style(paint).add_modifier(Modifier::BOLD),
    ));
    let length_line = Line::from(match app.mode {
        RaceMode::Time => format!("{}s", len),
        RaceMode::Words => format!("{} words", len),
    });

    let account_line = match &app.account {
        Some(acc) => Line::from(Span::styled(
            format!("{} (elo: {})", acc.username, acc.elo),
            base_style(paint)
                .fg(paint.get(ThemeField::TypedCorrect))
                .add_modifier(Modifier::BOLD),
        )),
        None => Line::from(Span::styled(
            "Playing as a guest",
            base_style(paint).fg(paint.get(ThemeField::TypedCorrect)),
        )),
    };

    let text = vec![
        account_line,
        Line::from(""),
        mode_line,
        length_line,
        Line::from(""),
        Line::from("S: start a race"),
        Line::from("F: find an opponent (Time: 30s)"),
        Line::from("C: customize"),
        Line::from("Esc: Quit"),
    ];
    frame.render_widget(default_paragraph(text, paint), chunks[0]);

    let hint = vec![
        Line::from(""),
        Line::from(Span::styled(
            "Tab: mode    Left/Right: length",
            base_style(paint).fg(paint.resolve(Color::DarkGray)),
        )),
    ];
    frame.render_widget(default_paragraph(hint, paint), chunks[1]);
}

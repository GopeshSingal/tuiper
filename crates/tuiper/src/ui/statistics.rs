use crate::app::App;
use crate::theme::ThemeField;

use super::common::{base_style, default_block, default_paragraph};

use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier};
use ratatui::text::{Line, Span};
use ratatui::Frame;

fn format_played_at(played_at: &str) -> String {
    if played_at.len() >= 10 {
        played_at[..10].to_string()
    } else {
        played_at.to_string()
    }
}

fn outcome_label(won: Option<bool>) -> &'static str {
    match won {
        Some(true) => "W",
        Some(false) => "L",
        None => "T",
    }
}

pub(super) fn draw_statistics(frame: &mut Frame, area: Rect, app: &mut App) {
    let theme = &app.theme;
    let block = default_block("Statistics", theme);
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(6), Constraint::Min(3), Constraint::Length(2)])
        .split(inner);

    let mut header_lines: Vec<Line> = Vec::new();

    if let Some(err) = &app.race_history_error {
        header_lines.push(Line::from(Span::styled(
            err.clone(),
            base_style(theme).fg(theme.get(ThemeField::TypedIncorrect)),
        )));
    } else if let Some(history) = &app.race_history {
        let summary = &history.summary;
        header_lines.push(Line::from(Span::styled(
            format!("{} online races", summary.total_races),
            base_style(theme).add_modifier(Modifier::BOLD),
        )));
        header_lines.push(Line::from(format!(
            "Avg WPM: {:.1}    Best WPM: {:.0}",
            summary.avg_wpm, summary.best_wpm
        )));
        match summary.avg_accuracy {
            Some(acc) => header_lines.push(Line::from(format!("Avg Accuracy: {:.1}%", acc))),
            None => header_lines.push(Line::from(Span::styled(
                "Avg Accuracy: —",
                base_style(theme).fg(Color::DarkGray),
            ))),
        }
    } else {
        header_lines.push(Line::from(Span::styled(
            "No data loaded.",
            base_style(theme).fg(Color::DarkGray),
        )));
    }

    frame.render_widget(default_paragraph(header_lines, theme), chunks[0]);

    let mut race_lines: Vec<Line> = Vec::new();
    if let Some(history) = &app.race_history {
        if history.races.is_empty() && app.race_history_error.is_none() {
            race_lines.push(Line::from(Span::styled(
                "No online races yet. Press F in the lobby to find an opponent.",
                base_style(theme).fg(Color::DarkGray),
            )));
        } else {
            for race in &history.races {
                let acc = match race.accuracy {
                    Some(a) => format!("{a:.1}%"),
                    None => "—".to_string(),
                };
                let opponent = race
                    .opponent_username
                    .as_deref()
                    .unwrap_or("guest");
                let opp_wpm = race
                    .opponent_wpm
                    .map(|w| format!("{w:.0}"))
                    .unwrap_or_else(|| "—".to_string());
                race_lines.push(Line::from(format!(
                    "{}  {:.0} wpm  {}  {}  vs {} ({})",
                    format_played_at(&race.played_at),
                    race.wpm,
                    acc,
                    outcome_label(race.won),
                    opponent,
                    opp_wpm,
                )));
            }
        }
    }

    let visible_height = chunks[1].height as usize;
    let max_offset = race_lines.len().saturating_sub(visible_height.max(1));
    if app.stats_scroll_offset > max_offset {
        app.stats_scroll_offset = max_offset;
    }
    let offset = app.stats_scroll_offset;
    let visible = if race_lines.is_empty() {
        race_lines
    } else {
        race_lines
            .into_iter()
            .skip(offset)
            .take(visible_height)
            .collect()
    };

    frame.render_widget(default_paragraph(visible, theme), chunks[1]);

    let scroll_hint = if max_offset > 0 {
        format!("Up/Down: scroll ({}/{})", offset + 1, max_offset + 1)
    } else {
        "Up/Down: scroll".to_string()
    };
    let hint = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled(format!("{scroll_hint}    "), base_style(theme).fg(Color::DarkGray)),
            Span::styled("Esc: quit", base_style(theme).fg(Color::DarkGray)),
        ]),
    ];
    frame.render_widget(default_paragraph(hint, theme), chunks[2]);
}

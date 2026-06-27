use crate::app::App;
use crate::theme::Theme;

use super::common::{base_style, default_block, default_paragraph};

use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier};
use ratatui::text::{Line, Span};
use ratatui::Frame;

pub(super) fn draw_leaderboard(frame: &mut Frame, area: Rect, theme: &Theme, app: &App) {
    let block = default_block("Leaderboard", theme);
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(3), Constraint::Length(2)])
        .split(inner);

    let mut lines: Vec<Line> = Vec::new();

    match &app.leaderboard {
        Some(lb) => {
            lines.push(Line::from(Span::styled(
                "Top ELO",
                base_style(theme).add_modifier(Modifier::BOLD),
            )));
            if lb.top_elo.is_empty() {
                lines.push(Line::from(Span::styled(
                    "(no accounts yet)",
                    base_style(theme).fg(Color::DarkGray),
                )));
            } else {
                for (i, e) in lb.top_elo.iter().enumerate() {
                    lines.push(Line::from(format!(
                        "{}. {} — {}",
                        i + 1,
                        e.username,
                        e.elo
                    )));
                }
            }

            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                "Top Daily WPM (today)",
                base_style(theme).add_modifier(Modifier::BOLD),
            )));
            if lb.top_daily_wpm.is_empty() {
                lines.push(Line::from(Span::styled(
                    "(no matches today)",
                    base_style(theme).fg(Color::DarkGray),
                )));
            } else {
                for (i, e) in lb.top_daily_wpm.iter().enumerate() {
                    lines.push(Line::from(format!(
                        "{}. {} — {:.1} wpm",
                        i + 1,
                        e.username,
                        e.wpm
                    )));
                }
            }
        }
        None => {
            lines.push(Line::from(Span::styled(
                "No data loaded.",
                base_style(theme).fg(Color::DarkGray),
            )));
        }
    }

    frame.render_widget(default_paragraph(lines, theme), chunks[0]);

    let hint = vec![
        Line::from(""),
        Line::from(Span::styled(
            "Q: back to lobby    Esc: quit",
            base_style(theme).fg(Color::DarkGray),
        )),
    ];
    frame.render_widget(default_paragraph(hint, theme), chunks[1]);
}

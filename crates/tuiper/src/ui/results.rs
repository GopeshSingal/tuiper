use crate::app::App;
use crate::theme::Theme;

use super::common::{base_style, default_block, default_paragraph};

use ratatui::style::Color;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Paragraph, Wrap};
use ratatui::Frame;

pub(super) fn draw_results(frame: &mut Frame, theme: &Theme, app: &App) {
    let area = frame.area();
    let block = default_block("Results", theme);
    let inner = block.inner(area);
    frame.render_widget(block, area);

    if let Some(ref res) = app.race_results {
        let winner_str = match &res.winner {
            Some(protocols::Winner::Me) => "You won :)",
            Some(protocols::Winner::Opponent) => "You lost :(",
            None => "",
        };
        let text = vec![
            Line::from(""),
            Line::from(Span::styled(winner_str, base_style(theme).fg(Color::Green))),
            Line::from(""),
            Line::from(vec![
                Span::styled("You: ", base_style(theme).fg(Color::Cyan)),
                Span::styled(
                    format!("{:.0} WPM  {:.1}% acc", res.me.wpm, res.me.accuracy),
                    base_style(theme),
                ),
            ]),
            Line::from(vec![
                Span::styled("Opponent: ", base_style(theme).fg(Color::Cyan)),
                Span::styled(
                    format!(
                        "{:.0} WPM  {:.1}% acc",
                        res.opponent.wpm, res.opponent.accuracy
                    ),
                    base_style(theme),
                ),
            ]),
            Line::from(""),
            Line::from(Span::styled(
                "Tab or Enter: race again  Q: lobby",
                base_style(theme),
            )),
        ];
        frame.render_widget(default_paragraph(text, theme), inner);
    } else if let Some(r) = app.result() {
        let text = vec![
            Line::from(""),
            Line::from(vec![
                Span::styled("WPM: ", base_style(theme).fg(Color::Cyan)),
                Span::styled(format!("{:.0}", r.wpm), base_style(theme)),
            ]),
            Line::from(vec![
                Span::styled("Raw WPM: ", base_style(theme).fg(Color::Cyan)),
                Span::styled(format!("{:.0}", r.raw_wpm), base_style(theme)),
            ]),
            Line::from(vec![
                Span::styled("Accuracy: ", base_style(theme).fg(Color::Cyan)),
                Span::styled(format!("{:.1}%", r.accuracy), base_style(theme)),
            ]),
            Line::from(vec![
                Span::styled("Consistency: ", base_style(theme).fg(Color::Cyan)),
                Span::styled(format!("{:.0}%", r.consistency), base_style(theme)),
            ]),
            Line::from(vec![
                Span::styled("Time: ", base_style(theme).fg(Color::Cyan)),
                Span::styled(format!("{:.1}", r.duration_secs), base_style(theme)),
            ]),
            Line::from(""),
            Line::from(Span::styled(
                "Tab / Enter: try again    Q: lobby",
                base_style(theme),
            )),
        ];
        frame.render_widget(
            Paragraph::new(text)
                .wrap(Wrap { trim: false })
                .style(base_style(theme)),
            inner,
        );
    } else {
        frame.render_widget(
            Paragraph::new("No results to display!").style(base_style(theme)),
            inner,
        );
    }
}

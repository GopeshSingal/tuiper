use crate::app::App;
use crate::theme::{Theme, ThemeField};

use super::common::{base_style, default_block, default_paragraph};

use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::Color;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Axis, Chart, Dataset, GraphType, Paragraph, Wrap};
use ratatui::Frame;

fn draw_wpm_chart(
    frame: &mut Frame,
    theme: &Theme,
    area: ratatui::layout::Rect,
    local: Option<&crate::typing::TypingStats>,
    opponent_history: Option<&[(f64, f64)]>,
) {
    let local_hist = local
        .filter(|r| !r.wpm_history.is_empty())
        .map(|r| r.wpm_history.as_slice());
    let opp_hist = opponent_history.filter(|h| !h.is_empty());

    if local_hist.is_none() && opp_hist.is_none() {
        frame.render_widget(
            Paragraph::new("Not enough typing data to plot WPM graph.")
                .style(base_style(theme))
                .block(default_block("WPM Over Time", theme)),
            area,
        );
        return;
    }

    let mut x_data_max = 1.0_f64;
    if let Some(h) = local_hist {
        x_data_max = x_data_max.max(h.iter().map(|(secs, _)| *secs).fold(0.0_f64, f64::max));
    }
    if let Some(h) = opp_hist {
        x_data_max = x_data_max.max(h.iter().map(|(secs, _)| *secs).fold(0.0_f64, f64::max));
    }
    x_data_max = x_data_max.max(1.0);

    let rounded_15 = (x_data_max / 15.0).round() * 15.0;
    let snap_to_15 = rounded_15 >= 15.0 && (x_data_max - rounded_15).abs() <= 1.5;
    let x_axis_max = if snap_to_15 { rounded_15 } else { x_data_max };

    let x_axis_min = if x_axis_max > 0.5 { 0.5 } else { 0.0 };

    let x_labels = if snap_to_15 {
        vec![
            Span::styled(
                format!("{:.1}", x_axis_min),
                base_style(theme).fg(theme.get(ThemeField::Untyped)),
            ),
            Span::styled(
                format!("{:.0}", x_axis_max),
                base_style(theme).fg(theme.get(ThemeField::Untyped)),
            ),
        ]
    } else {
        vec![
            Span::styled(
                format!("{:.1}", x_axis_min),
                base_style(theme).fg(theme.get(ThemeField::Untyped)),
            ),
            Span::styled(
                format!("{:.1}", x_data_max),
                base_style(theme).fg(theme.get(ThemeField::Untyped)),
            ),
        ]
    };

    let mut max_wpm = 0.0_f64;
    let mut min_non_zero_wpm = f64::INFINITY;
    if let Some(h) = local_hist {
        for (_, wpm) in h.iter() {
            max_wpm = max_wpm.max(*wpm);
            if *wpm > 0.0 {
                min_non_zero_wpm = min_non_zero_wpm.min(*wpm);
            }
        }
    }
    if let Some(h) = opp_hist {
        for (_, wpm) in h.iter() {
            max_wpm = max_wpm.max(*wpm);
            if *wpm > 0.0 {
                min_non_zero_wpm = min_non_zero_wpm.min(*wpm);
            }
        }
    }
    let y_min = if min_non_zero_wpm.is_finite() {
        (min_non_zero_wpm - 5.0).max(0.0)
    } else {
        0.0
    };
    let mut y_max = (max_wpm + 5.0).max(10.0);
    if y_max <= y_min {
        y_max = y_min + 10.0;
    }
    let y_axis_min = (y_min / 10.0).floor() * 10.0;
    let mut y_axis_max = (y_max / 10.0).ceil() * 10.0;
    if y_axis_max <= y_axis_min {
        y_axis_max = y_axis_min + 10.0;
    }

    let mut y_label_values = Vec::new();
    let mut current = y_axis_max;
    while current >= y_axis_min {
        y_label_values.push(current);
        current -= 10.0;
    }
    let y_labels = y_label_values
        .into_iter()
        .rev()
        .map(|v| {
            Span::styled(
                format!("{:.0}", v),
                base_style(theme).fg(theme.get(ThemeField::Untyped)),
            )
        })
        .collect::<Vec<_>>();

    let local_plot = local_hist.map(|h| {
        let mut pts = h.to_vec();
        if pts.last().is_some() {
            if let Some(last) = pts.last_mut() {
                last.0 = x_axis_max;
            }
        }
        pts
    });
    let opp_plot = opp_hist.map(|h| h.to_vec());

    let title = if opp_hist.is_some() {
        "WPM Over Time (you vs opponent)"
    } else {
        "WPM Over Time"
    };

    let mut datasets = Vec::new();
    if let Some(ref pts) = local_plot {
        datasets.push(
            Dataset::default()
                .marker(ratatui::symbols::Marker::Braille)
                .graph_type(GraphType::Line)
                .style(base_style(theme).fg(theme.get(ThemeField::TypedCorrect)))
                .data(pts.as_slice()),
        );
    }
    if let Some(ref pts) = opp_plot {
        datasets.push(
            Dataset::default()
                .marker(ratatui::symbols::Marker::Braille)
                .graph_type(GraphType::Line)
                .style(base_style(theme).fg(theme.get(ThemeField::TypedIncorrect)))
                .data(pts.as_slice()),
        );
    }

    let chart = Chart::new(datasets)
        .block(default_block(title, theme))
        .style(base_style(theme))
        .x_axis(
            Axis::default()
                .style(base_style(theme).fg(theme.get(ThemeField::Untyped)))
                .bounds([x_axis_min, x_axis_max])
                .labels(x_labels),
        )
        .y_axis(
            Axis::default()
                .style(base_style(theme).fg(theme.get(ThemeField::Untyped)))
                .bounds([y_axis_min, y_axis_max])
                .labels(y_labels),
        );

    frame.render_widget(chart, area);
}

pub(super) fn draw_results(frame: &mut Frame, theme: &Theme, app: &App) {
    let area = frame.area();
    let block = default_block("Results", theme);
    let inner = block.inner(area);
    frame.render_widget(block, area);

    if let Some(ref res) = app.race_results {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(7),
                Constraint::Min(8),
                Constraint::Length(1),
            ])
            .split(inner);

        let winner_span = match &res.winner {
            Some(protocols::Winner::Me) => Span::styled("You won :)", base_style(theme).fg(theme.get(ThemeField::TypedCorrect))),
            Some(protocols::Winner::Opponent) => Span::styled("You lost :(", base_style(theme).fg(theme.get(ThemeField::TypedIncorrect))),
            None => Span::styled("", base_style(theme).fg(Color::Cyan))
        };

        let text = vec![
            Line::from(""),
            Line::from(winner_span),
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
        ];
        frame.render_widget(default_paragraph(text, theme), chunks[0]);

        if app.result().is_some() || !app.opponent_wpm_history.is_empty() {
            draw_wpm_chart(
                frame,
                theme,
                chunks[1],
                app.result(),
                Some(app.opponent_wpm_history.as_slice()),
            );
        } else {
            frame.render_widget(
                Paragraph::new("No local WPM history available.")
                    .style(base_style(theme))
                    .block(default_block("WPM Over Time", theme)),
                chunks[1],
            );
        }

        frame.render_widget(
            Paragraph::new("Tab or Enter: race again  Q: lobby")
                .style(base_style(theme).fg(Color::DarkGray)),
            chunks[2],
        );
    } else if let Some(r) = app.result() {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(7),
                Constraint::Min(8),
                Constraint::Length(1),
            ])
            .split(inner);

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
        ];
        frame.render_widget(
            Paragraph::new(text)
                .wrap(Wrap { trim: false })
                .style(base_style(theme)),
            chunks[0],
        );

        draw_wpm_chart(frame, theme, chunks[1], Some(r), None);
        frame.render_widget(
            Paragraph::new("Tab / Enter: try again    Q: lobby")
                .style(base_style(theme).fg(Color::DarkGray)),
            chunks[2],
        );
    } else {
        frame.render_widget(
            Paragraph::new("No results to display!").style(base_style(theme)),
            inner,
        );
    }
}

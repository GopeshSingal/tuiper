use crate::app::{App, Screen};
use crate::theme::{Theme, ThemeEditColumn, ThemeField};
use crate::typing::CharState;

use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Cell, Paragraph, Row, Table, Wrap};
use ratatui::Frame;
use strum::IntoEnumIterator;

const APP_BG: Color = Color::Rgb(16, 16, 16);

fn base_style(theme: &Theme) -> Style {
    Style::default().bg(theme.get(ThemeField::WindowBg))
}

pub fn draw(frame: &mut Frame, app: &App) {
    let area = frame.area();
    let theme = &app.theme;
    frame.render_widget(Block::default().style(base_style(theme)), area);

    match app.screen {
        Screen::Lobby => draw_lobby(frame, theme),
        Screen::Queue => draw_queue(frame, theme),
        Screen::Race => draw_race(frame, theme, app),
        Screen::Results => draw_results(frame, theme, app),
        Screen::Config => draw_config(frame, theme, app),
    }
}

fn draw_lobby(frame: &mut Frame, theme: &Theme) {
    let area = frame.area();
    let block = Block::default()
        .borders(Borders::ALL)
        .title("Tuiper")
        .style(base_style(theme));
    let inner = block.inner(area);
    frame.render_widget(block, area);
    let text = vec![
        Line::from(""),
        Line::from("S: start a race"),
        Line::from("F: find an opponent"),
        Line::from("C: customize"),
        Line::from("Esc: Quit"),
    ];
    frame.render_widget(
        Paragraph::new(text)
            .wrap(Wrap { trim: false })
            .style(base_style(theme)),
        inner,
    );
}

fn draw_queue(frame: &mut Frame, theme: &Theme) {
    let area = frame.area();
    let block = Block::default()
        .borders(Borders::ALL)
        .title("Finding opponent")
        .style(base_style(theme));
    let inner = block.inner(area);
    frame.render_widget(block, area);
    let text = vec![
        Line::from(""),
        Line::from("Q: leave queue"),
        Line::from("Esc: Quit"),
    ];
    frame.render_widget(
        Paragraph::new(text)
            .wrap(Wrap { trim: false })
            .style(base_style(theme)),
        inner,
    );
}

fn draw_race(frame: &mut Frame, theme: &Theme, app: &App) {
    let area = frame.area();
    let typing = app.typing();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2),
            Constraint::Length(2),
            Constraint::Min(5),
            Constraint::Length(3),
        ])
        .split(area);

    if let Some(t) = typing {
        let header_str = format!("Time: {}s | {:.0}s elapsed", t.value(), t.elapsed_secs());
        let stats = format!(
            "WPM: {:.0} Raw: {:.0} Acc: {:.1}% Consistency: {:.0}% | {}",
            t.wpm(),
            t.raw_wpm(),
            t.accuracy(),
            t.consistency(),
            header_str
        );
        frame.render_widget(
            Paragraph::new(stats).style(base_style(theme).fg(Color::Cyan)),
            chunks[0],
        );

        if app.is_waiting_for_multiplayer_start() {
            let countdown = app.multiplayer_countdown_secs().unwrap_or(0);
            let waiting = format!("Starting in {}s...", countdown);
            frame.render_widget(
                Paragraph::new(waiting).style(base_style(theme).fg(Color::Yellow)),
                chunks[1],
            );
        } else if app.is_multi() {
            let opponent_stats = format!("Opponent WPM: {:.0}, Opponent Chars: {}", app.opponent_wpm, app.opponent_chars);
            frame.render_widget(
                Paragraph::new(opponent_stats).style(base_style(theme).fg(Color::Yellow)),
                chunks[1],
            );
        } else {
            frame.render_widget(Paragraph::new("").style(base_style(theme)), chunks[1]);
        }

        let states = t.char_states();
        let pending_error = t.has_unfixed_error();
        let text_chars: Vec<char> = t.text().chars().collect();
        let opponent_cursor_idx =
            if app.is_multi() && !app.is_waiting_for_multiplayer_start() {
                let oc = app.opponent_chars as usize;
                if oc < text_chars.len() {
                    Some(oc)
                } else {
                    None
                }
            } else {
                None
            };
        let mut spans: Vec<Span> = Vec::new();
        for (i, &c) in text_chars.iter().enumerate() {
            let state = states.get(i).copied().unwrap_or(CharState::Untyped);
            let mut style = match state {
                CharState::Correct => base_style(theme).fg(theme.get(ThemeField::TypedCorrect)),
                CharState::Incorrect => base_style(theme)
                    .fg(theme.get(ThemeField::TypedIncorrect))
                    .add_modifier(Modifier::UNDERLINED),
                CharState::Current if pending_error => base_style(theme).bg(theme.get(ThemeField::CursorBg)).fg(theme.get(ThemeField::TypedIncorrect)),
                CharState::Current => base_style(theme).bg(theme.get(ThemeField::CursorBg)).fg(theme.get(ThemeField::CursorFg)),
                CharState::Untyped => base_style(theme).fg(theme.get(ThemeField::Untyped)),
            };
            if opponent_cursor_idx == Some(i) {
                if matches!(state, CharState::Current) {
                    style = if pending_error {
                        base_style(theme).bg(theme.get(ThemeField::OppCursorBg)).fg(theme.get(ThemeField::TypedIncorrect))
                    } else {
                        base_style(theme).bg(theme.get(ThemeField::OppCursorBg)).fg(theme.get(ThemeField::OppCursorFg))
                    };
                } else {
                    style = style.bg(Color::Magenta);
                }
            }
            spans.push(Span::styled(c.to_string(), style));
        }
        let line = Line::from(spans);
        let block = Block::default()
            .borders(Borders::ALL)
            .title("Type the given text!")
            .style(base_style(theme));
        let inner = block.inner(chunks[2]);
        frame.render_widget(block, chunks[2]);
        frame.render_widget(
            Paragraph::new(line)
                .wrap(Wrap { trim: false })
                .style(base_style(theme)),
            inner,
        );
    } else {
        let msg = Paragraph::new("Loading...").block(
            Block::default()
                .borders(Borders::ALL)
                .style(base_style(theme)),
        );
        frame.render_widget(msg, chunks[2]);
    }

    let hint = if app.is_waiting_for_multiplayer_start() {
        "Waiting for race to start"
    } else if app.is_multi() {
        "Race is in progress"
    } else {
        "Tab: restart"
    };
    frame.render_widget(
        Paragraph::new(hint).style(base_style(theme).fg(Color::DarkGray)),
        chunks[3],
    );
}

fn draw_results(frame: &mut Frame, theme: &Theme, app: &App) {
    let area = frame.area();
    let block = Block::default()
        .borders(Borders::ALL)
        .title("Results ")
        .style(base_style(theme));
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
                    format!("{:.0} WPM  {:.1}% acc", res.opponent.wpm, res.opponent.accuracy),
                    base_style(theme),
                ),
            ]),
            Line::from(""),
            Line::from(Span::styled(
                "Tab or Enter: race again  Q: lobby",
                base_style(theme),
            )),
        ];
        frame.render_widget(
            Paragraph::new(text)
                .wrap(Wrap { trim: false })
                .style(base_style(theme)),
            inner,
        );
    } else if let Some(ref r) = app.result() {
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
                "Tab or Enter: try again    Q: lobby",
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

fn draw_config(frame: &mut Frame, theme: &Theme, app: &App) {
    let area = frame.area();
    let block = Block::default()
        .borders(Borders::ALL)
        .title("Config")
        .style(base_style(theme));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(5), Constraint::Length(2)])
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
                Cell::from("Setting"),
                Cell::from("Palette"),
                Cell::from("Shade"),
            ])
            .style(base_style(theme).add_modifier(Modifier::BOLD))
            .bottom_margin(1),
        )
        .column_spacing(1)
        .style(base_style(theme));

    frame.render_widget(table, chunks[0]);

    let hints = vec![
        Line::from(""),
        Line::from(Span::styled(
            "Use arrow keys to navigate  Tab: cycle palette or shade   Q: save & back",
            base_style(theme).fg(Color::DarkGray),
        )),
    ];
    frame.render_widget(
        Paragraph::new(hints)
            .wrap(Wrap { trim: false })
            .style(base_style(theme)),
        chunks[1],
    );
}

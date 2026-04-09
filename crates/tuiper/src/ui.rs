use crate::app::{App, RaceMode, Screen};
use crate::theme::{Theme, ThemeEditColumn, ThemeField};
use crate::typing::CharState;

use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Cell, Paragraph, Row, Table, Wrap};
use ratatui::Frame;
use strum::IntoEnumIterator;

fn base_style(theme: &Theme) -> Style {
    Style::default().bg(theme.get(ThemeField::WindowBg))
}

fn typing_char_style(
    theme: &Theme,
    state: CharState,
    pending_error: bool,
    opponent_cursor_idx: Option<usize>,
    i: usize,
) -> Style {
    let opp = opponent_cursor_idx == Some(i);
    if opp && matches!(state, CharState::Current) {
        return if pending_error {
            base_style(theme)
                .bg(theme.get(ThemeField::OppCursorBg))
                .fg(theme.get(ThemeField::TypedIncorrect))
        } else {
            base_style(theme)
                .bg(theme.get(ThemeField::OppCursorBg))
                .fg(theme.get(ThemeField::OppCursorFg))
        };
    }

    let mut style = match state {
        CharState::Correct => base_style(theme).fg(theme.get(ThemeField::TypedCorrect)),
        CharState::Incorrect => base_style(theme)
            .fg(theme.get(ThemeField::TypedIncorrect))
            .add_modifier(Modifier::UNDERLINED),
        CharState::Current if pending_error => base_style(theme)
            .bg(theme.get(ThemeField::CursorBg))
            .fg(theme.get(ThemeField::TypedIncorrect)),
        CharState::Current => base_style(theme)
            .bg(theme.get(ThemeField::CursorBg))
            .fg(theme.get(ThemeField::CursorFg)),
        CharState::Untyped => base_style(theme).fg(theme.get(ThemeField::Untyped)),
    };

    if opp {
        style = style
            .bg(theme.get(ThemeField::OppCursorBg))
            .fg(theme.get(ThemeField::OppCursorFg));
    }
    style
}

fn race_progress_line(theme: &Theme, width: u16, ratio: f64) -> Line<'static> {
    const DOT_TRACK: &str = "·";
    const DOT_FILL: &str = "•";
    const DOT_FILL_CURRENT: &str = "●";

    let w = width as usize;
    if w < 2 {
        return Line::from("");
    }
    let inner = w - 2;
    let ratio = ratio.clamp(0.0, 1.0);
    let filled_end = ratio * inner as f64;
    let filled_count = (filled_end.floor() as usize).min(inner);

    let bracket_style = base_style(theme).fg(theme.get(ThemeField::Untyped));
    let track_style = base_style(theme).fg(theme.get(ThemeField::Untyped));
    let fill_style = base_style(theme).fg(theme.get(ThemeField::TypedCorrect));
    let fill_current_style = base_style(theme)
        .fg(theme.get(ThemeField::TypedCorrect))
        .add_modifier(Modifier::BOLD);

    let mut spans = Vec::with_capacity(inner + 2);
    spans.push(Span::styled("[", bracket_style));

    for i in 0..inner {
        if i >= filled_count {
            spans.push(Span::styled(DOT_TRACK, track_style));
        } else if i == filled_count - 1 {
            spans.push(Span::styled(DOT_FILL_CURRENT, fill_current_style));
        } else {
            spans.push(Span::styled(DOT_FILL, fill_style));
        }
    }

    spans.push(Span::styled("]", bracket_style));
    Line::from(spans)
}

fn line_from_typing(
    theme: &Theme,
    indexed_chars: impl Iterator<Item = (usize, char)>,
    at: impl Fn(usize) -> (CharState, bool),
    opponent_cursor_idx: Option<usize>,
) -> Line<'_> {
    Line::from(
        indexed_chars
            .map(|(i, c)| {
                let (state, pe) = at(i);
                Span::styled(
                    c.to_string(),
                    typing_char_style(theme, state, pe, opponent_cursor_idx, i),
                )
            })
            .collect::<Vec<Span>>(),
    )
}

pub fn draw(frame: &mut Frame, app: &App) {
    let area = frame.area();
    let theme = &app.theme;
    frame.render_widget(Block::default().style(base_style(theme)), area);

    match app.screen {
        Screen::Lobby => draw_lobby(frame, theme, app),
        Screen::Queue => draw_queue(frame, theme),
        Screen::Race => draw_race(frame, theme, app),
        Screen::Results => draw_results(frame, theme, app),
        Screen::Config => draw_config(frame, theme, app),
    }
}

fn draw_lobby(frame: &mut Frame, theme: &Theme, app: &App) {
    let area = frame.area();
    let block = Block::default()
        .borders(Borders::ALL)
        .title("Tuiper")
        .style(base_style(theme));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let mode_line = match app.mode {
        RaceMode::Time => "Mode: Time",
        RaceMode::Words => "Mode: Words",
    };

    let text = vec![
        Line::from(""),
        Line::from(mode_line),
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
    let middle_h = if typing.is_some()
        && (app.is_waiting_for_multiplayer_start() || app.is_multi())
    {
        1u16
    } else {
        0
    };
    let header_h = if typing.is_some() { 4u16 } else { 2u16 };
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(header_h),
            Constraint::Length(middle_h),
            Constraint::Min(5),
            Constraint::Length(3),
        ])
        .split(area);

    if let Some(t) = typing {
        let header_rows = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Length(1)])
            .split(chunks[0]);

        let finish_mode = if app.is_multi() {
            RaceMode::Time
        } else {
            app.mode
        };
        let ratio = t.progress_ratio(finish_mode);

        let stats = format!(
            "WPM: {:.0} Raw: {:.0} Acc: {:.1}% Consistency: {:.0}%",
            t.wpm(),
            t.raw_wpm(),
            t.accuracy(),
            t.consistency(),
        );
        let stats_block = Block::default()
            .borders(Borders::ALL)
            .title("Stats")
            .style(base_style(theme));
        let stats_inner = stats_block.inner(header_rows[0]);
        frame.render_widget(stats_block, header_rows[0]);
        frame.render_widget(
            Paragraph::new(stats).style(base_style(theme).fg(Color::Cyan)),
            stats_inner,
        );

        let bar = race_progress_line(theme, header_rows[1].width, ratio);
        frame.render_widget(
            Paragraph::new(bar).style(base_style(theme)),
            header_rows[1],
        );

        if app.is_waiting_for_multiplayer_start() {
            let countdown = app.multiplayer_countdown_secs().unwrap_or(0);
            let waiting = format!("Starting in {}s...", countdown);
            frame.render_widget(
                Paragraph::new(waiting).style(base_style(theme).fg(Color::Yellow)),
                chunks[1],
            );
        } else if app.is_multi() {
            let opponent_stats = format!("Opponent WPM: {:.0}", app.opponent_wpm);
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
        let line = line_from_typing(
            theme,
            text_chars.iter().cloned().enumerate(),
            |i| (states.get(i).copied().unwrap_or(CharState::Untyped), pending_error),
            opponent_cursor_idx,
        );
        let block = Block::default()
            .borders(Borders::ALL)
            .title("Type here")
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
        "Tab / Enter: restart"
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
        |i| {
            match i {
                0..=1 => (CharState::Correct, false),
                2 => (CharState::Incorrect, false),
                3 => (CharState::Current, true),
                _ => (CharState::Untyped, false),
            }
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
